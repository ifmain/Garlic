//! Observation and rollback rules from the deployed W two-stage watchdog.
pub mod health;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    Healthy,
    Offline,
    Inconclusive,
}

pub fn aggregate_checks(checks: &[Probe], require_all: bool) -> Probe {
    if checks.iter().all(|p| *p == Probe::Healthy)
        || (!require_all && checks.contains(&Probe::Healthy))
    {
        Probe::Healthy
    } else if checks.iter().all(|p| *p == Probe::Offline) {
        Probe::Offline
    } else {
        Probe::Inconclusive
    }
}

pub fn aggregate_paths(paths: &[Probe]) -> Probe {
    if paths.contains(&Probe::Offline) {
        Probe::Offline
    } else if paths.iter().all(|p| *p == Probe::Healthy) {
        Probe::Healthy
    } else {
        Probe::Inconclusive
    }
}

#[derive(Clone, Debug)]
pub struct Clock {
    pub boot: String,
    pub monotonic: f64,
    pub wall: f64,
}

pub struct Trial {
    pub previous_slot: String,
    pub start: Clock,
    pub last: Option<(String, f64)>,
    pub down_since: Option<f64>,
}

pub struct State {
    pub active_slot: String,
    pub trial: Option<Trial>,
    pub event: String,
}

impl State {
    pub fn sample(&mut self, probe: Probe, now: &Clock) {
        let Some(trial) = self.trial.as_mut() else {
            return;
        };
        let age = if now.boot == trial.start.boot {
            now.monotonic - trial.start.monotonic
        } else {
            now.wall - trial.start.wall
        };
        if age >= 3600.0 || age < 0.0 {
            self.event = "window-closed".into();
            self.trial = None;
            return;
        }
        if probe != Probe::Offline {
            trial.down_since = None;
        } else {
            let consecutive = trial.last.as_ref().is_some_and(|(boot, time)| {
                boot == &now.boot && (0.0..=35.0).contains(&(now.monotonic - time))
            });
            match trial.down_since {
                Some(since) if consecutive && now.monotonic - since >= 300.0 => {
                    self.active_slot = trial.previous_slot.clone();
                    self.event = "reverted-offline".into();
                    self.trial = None;
                    return;
                }
                Some(_) if consecutive => {}
                _ => trial.down_since = Some(now.monotonic),
            }
        }
        trial.last = Some((now.boot.clone(), now.monotonic));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn clock(t: f64) -> Clock {
        Clock {
            boot: "A".into(),
            monotonic: t,
            wall: t,
        }
    }
    fn promoted() -> State {
        State {
            active_slot: "green".into(),
            event: "promoted".into(),
            trial: Some(Trial {
                previous_slot: "blue".into(),
                start: clock(0.0),
                last: None,
                down_since: None,
            }),
        }
    }
    #[test]
    fn requires_continuous_outage() {
        let mut s = promoted();
        for t in (0..=270).step_by(30) {
            s.sample(Probe::Offline, &clock(t as f64));
        }
        assert_eq!(s.active_slot, "green");
        s.sample(Probe::Offline, &clock(300.0));
        assert_eq!(
            (s.active_slot.as_str(), s.event.as_str()),
            ("blue", "reverted-offline")
        );
        assert!(s.trial.is_none());
    }
    #[test]
    fn gaps_and_unknown_results_reset_outage() {
        let mut s = promoted();
        for t in [0., 35., 71.] {
            s.sample(Probe::Offline, &clock(t));
        }
        assert_eq!(s.trial.as_ref().unwrap().down_since, Some(71.));
        s.sample(Probe::Inconclusive, &clock(80.));
        assert!(s.trial.as_ref().unwrap().down_since.is_none());
        s.sample(
            Probe::Offline,
            &Clock {
                boot: "B".into(),
                monotonic: 1.,
                wall: 3600.,
            },
        );
        assert_eq!(s.event, "window-closed");
        assert_eq!(s.active_slot, "green");
    }
    #[test]
    fn mixed_probe_results() {
        use Probe::*;
        assert_eq!(aggregate_checks(&[Healthy, Offline], false), Healthy);
        assert_eq!(aggregate_checks(&[Healthy, Offline], true), Inconclusive);
        assert_eq!(
            aggregate_checks(&[Offline, Inconclusive], false),
            Inconclusive
        );
        assert_eq!(aggregate_paths(&[Offline, Inconclusive]), Offline);
    }
}
