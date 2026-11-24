use super::Flow;
use garlic_relay_data::control::Authority;
use std::collections::{hash_map::Entry, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub(super) type Slot = Arc<Mutex<Assignment>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Attached(Uuid),
    Parked(Instant),
    Closed,
}

pub(super) struct Assignment {
    pub state: State,
    pub flow: Flow,
}

/// Relay connections for a single provisioned exit and authority share the
/// destination sockets. A different owner or epoch gets a separate pool.
#[derive(Clone)]
pub struct FlowPool {
    pub(super) authority: Authority,
    limit: usize,
    grace: Duration,
    slots: Arc<Mutex<HashMap<[u8; 16], Slot>>>,
}

impl FlowPool {
    pub fn new(authority: Authority, limit: usize, grace: Duration) -> Self {
        Self {
            authority,
            limit,
            grace,
            slots: Default::default(),
        }
    }

    pub(super) fn insert(
        &self,
        id: [u8; 16],
        worker: Uuid,
        flow: Flow,
    ) -> Result<Slot, garlic_relay_data::control::ErrorCode> {
        use garlic_relay_data::control::ErrorCode;
        let mut slots = self.slots.lock().unwrap();
        if slots.len() >= self.limit {
            return Err(ErrorCode::CapacityExhausted);
        }
        match slots.entry(id) {
            Entry::Occupied(_) => Err(ErrorCode::InvalidRequest),
            Entry::Vacant(entry) => Ok(entry
                .insert(Arc::new(Mutex::new(Assignment {
                    state: State::Attached(worker),
                    flow,
                })))
                .clone()),
        }
    }

    pub(super) fn adopt(&self, id: &[u8; 16], worker: Uuid) -> Option<Slot> {
        let slot = self.slots.lock().unwrap().get(id)?.clone();
        let mut assignment = slot.lock().unwrap();
        match assignment.state {
            State::Closed => return None,
            State::Parked(at) if at.elapsed() >= self.grace => return None,
            State::Attached(previous) if previous == worker => {}
            _ => {
                assignment.state = State::Attached(worker);
                assignment.flow.replay = true;
            }
        }
        drop(assignment);
        Some(slot)
    }

    /// Called even when attempts to reauthenticate fail. A disconnected peer
    /// cannot keep an abandoned destination socket open indefinitely.
    pub fn expire(&self) {
        self.slots
            .lock()
            .unwrap()
            .retain(|_, slot| match slot.lock().unwrap().state {
                State::Closed => false,
                State::Parked(at) => at.elapsed() < self.grace,
                State::Attached(_) => true,
            });
    }
}
