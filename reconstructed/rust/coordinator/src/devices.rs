use crate::{
    accounts::{now, random_secret, token_hash},
    files,
};
use garlic_relay_session::SessionError;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Default, Deserialize, Serialize)]
struct Devices {
    leases: BTreeMap<Uuid, Lease>,
}

#[derive(Deserialize, Serialize)]
struct Lease {
    device_id: String,
    generation: u64,
    token_hash: String,
    expires_at: u64,
}

#[derive(Serialize)]
pub struct Claimed {
    pub device_id: String,
    pub generation: u64,
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug)]
pub enum DeviceError {
    Invalid,
    InUse,
}
impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "device_authorization_failed",
            Self::InUse => "device_in_use",
        })
    }
}
impl Error for DeviceError {}

pub struct DeviceStore {
    path: PathBuf,
}
impl DeviceStore {
    pub fn for_accounts(path: &Path) -> Self {
        Self {
            path: path.with_extension("devices.json"),
        }
    }

    pub fn claim(&self, user: Uuid, device: &str, takeover: bool) -> Result<Claimed, SessionError> {
        let device = Uuid::parse_str(device)
            .ok()
            .filter(|id| !id.is_nil())
            .ok_or(DeviceError::Invalid)?
            .to_string();
        files::edit_or_create(
            &self.path,
            || Ok(Devices::default()),
            |state| {
                let time = now();
                let previous = state.leases.get(&user);
                if previous.is_some_and(|l| l.expires_at > time) && !takeover {
                    return Err(DeviceError::InUse.into());
                }
                let generation = previous.map_or(1, |l| l.generation.saturating_add(1));
                let token = random_secret("")?;
                let expires_at = time + 60;
                state.leases.insert(
                    user,
                    Lease {
                        device_id: device.clone(),
                        generation,
                        token_hash: token_hash(&token),
                        expires_at,
                    },
                );
                Ok(Claimed {
                    device_id: device,
                    generation,
                    token,
                    expires_at,
                })
            },
        )
    }

    pub fn valid(&self, user: Uuid, token: &str) -> Result<(), SessionError> {
        let state: Devices = match files::read(&self.path) {
            Err(err)
                if err
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                return Err(DeviceError::InUse.into())
            }
            other => other?,
        };
        let hash = token_hash(token);
        if state
            .leases
            .get(&user)
            .is_some_and(|l| l.token_hash == hash && l.expires_at > now())
        {
            Ok(())
        } else {
            Err(DeviceError::InUse.into())
        }
    }

    pub fn update(&self, user: Uuid, token: &str, release: bool) -> Result<(), SessionError> {
        let hash = token_hash(token);
        files::edit_or_create(
            &self.path,
            || Ok(Devices::default()),
            |state| {
                let time = now();
                let lease = state
                    .leases
                    .get_mut(&user)
                    .filter(|l| l.expires_at > time && l.token_hash == hash)
                    .ok_or(DeviceError::InUse)?;
                lease.expires_at = if release { 0 } else { time + 60 };
                Ok(())
            },
        )
    }
}
