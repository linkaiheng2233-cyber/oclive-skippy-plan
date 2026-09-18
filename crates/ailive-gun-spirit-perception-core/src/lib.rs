//! Deterministic perception edge derivation.
//!
//! Hardware adapters own sampling and filtering. This crate consumes typed states and never knows
//! OCLive, BLE, GPIO, databases, Linux services, roles, cooldowns, or response probability.

mod reducer;

pub use reducer::{
    ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer, ProfileError, ReducerError, ReductionDisposition, ReductionOutput,
};

use ailive_gun_spirit_contracts::{
    CarrierState, Contact, ContractValidate, DeviceEventKind, Fact, PerceptionState,
    ValidationError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayDisposition {
    BaselineEstablished,
    SnapshotResent,
    Advanced(Vec<DeviceEventKind>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidState(Vec<ValidationError>),
    RevisionRegressed { previous: u64, current: u64 },
    SameRevisionChanged { revision: u64 },
}

/// Stateful replay gate used by the desktop simulator and later by the Host composition root.
///
/// A new Host boot always establishes a quiet baseline. Within one boot, revisions must increase
/// only when semantic content changes. A same-revision snapshot with the same authoritative
/// payload is a legal resend even when its publication timestamp changes.
#[derive(Debug, Default)]
pub struct PerceptionReplay {
    previous: Option<PerceptionState>,
}

impl PerceptionReplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, current: PerceptionState) -> Result<ReplayDisposition, ReplayError> {
        current.validate().map_err(ReplayError::InvalidState)?;

        let Some(previous) = self.previous.as_ref() else {
            self.previous = Some(current);
            return Ok(ReplayDisposition::BaselineEstablished);
        };

        if previous.host_boot_id != current.host_boot_id {
            self.previous = Some(current);
            return Ok(ReplayDisposition::BaselineEstablished);
        }

        if current.revision < previous.revision {
            return Err(ReplayError::RevisionRegressed {
                previous: previous.revision,
                current: current.revision,
            });
        }

        if current.revision == previous.revision {
            if previous.has_same_revision_payload(&current) {
                self.previous = Some(current);
                return Ok(ReplayDisposition::SnapshotResent);
            }
            return Err(ReplayError::SameRevisionChanged {
                revision: current.revision,
            });
        }

        let events = derive_event_kinds(previous, &current);
        self.previous = Some(current);
        Ok(ReplayDisposition::Advanced(events))
    }

    pub fn latest(&self) -> Option<&PerceptionState> {
        self.previous.as_ref()
    }
}

// The replay gate owns baseline and revision validation; this helper only compares accepted,
// strictly advancing snapshots and is intentionally not part of the public API.
fn derive_event_kinds(
    previous: &PerceptionState,
    current: &PerceptionState,
) -> Vec<DeviceEventKind> {
    let mut events = Vec::new();
    derive_carrier_edges(
        known_value(&previous.carrier_state),
        known_value(&current.carrier_state),
        &mut events,
    );
    derive_primary_control_edges(
        known_value(&previous.primary_control),
        known_value(&current.primary_control),
        &mut events,
    );
    events
}

fn known_value<T>(fact: &Fact<T>) -> Option<&T> {
    match fact {
        Fact::Known { value, .. } => Some(value),
        Fact::Unknown { .. } => None,
    }
}

fn derive_carrier_edges(
    previous: Option<&CarrierState>,
    current: Option<&CarrierState>,
    events: &mut Vec<DeviceEventKind>,
) {
    let (Some(previous), Some(current)) = (previous, current) else {
        return;
    };

    let previous_held = matches!(previous, CarrierState::Held | CarrierState::Ready);
    let current_held = matches!(current, CarrierState::Held | CarrierState::Ready);
    if !previous_held && current_held {
        events.push(DeviceEventKind::CarrierHeldEntered);
    } else if previous_held && !current_held {
        events.push(DeviceEventKind::CarrierHeldExited);
    }

    let previous_ready = *previous == CarrierState::Ready;
    let current_ready = *current == CarrierState::Ready;
    if !previous_ready && current_ready {
        events.push(DeviceEventKind::CarrierReadyEntered);
    } else if previous_ready && !current_ready {
        events.push(DeviceEventKind::CarrierReadyExited);
    }
}

fn derive_primary_control_edges(
    previous: Option<&Contact>,
    current: Option<&Contact>,
    events: &mut Vec<DeviceEventKind>,
) {
    match (previous, current) {
        (Some(Contact::Released), Some(Contact::Engaged)) => {
            events.push(DeviceEventKind::ControlPrimaryEngaged);
        }
        (Some(Contact::Engaged), Some(Contact::Released)) => {
            events.push(DeviceEventKind::ControlPrimaryReleased);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ailive_gun_spirit_contracts::{
        BootId, ContractVersion, MotionClass, NodeHealth, Permille, Pose, SourceRef, UnknownReason,
    };

    fn known<T>(value: T) -> Fact<T> {
        Fact::Known {
            value,
            observed_at_monotonic_ms: 100,
            fresh_until_monotonic_ms: 500,
            confidence_milli: Permille::new(900).unwrap(),
            source_refs: Vec::<SourceRef>::new(),
        }
    }

    fn unknown<T>() -> Fact<T> {
        Fact::Unknown {
            reason: UnknownReason::NeverObserved,
            since_revision: 0,
        }
    }

    fn state(
        revision: u64,
        carrier: Fact<CarrierState>,
        control: Fact<Contact>,
    ) -> PerceptionState {
        PerceptionState {
            contract_version: ContractVersion::V0_2,
            host_boot_id: BootId::try_new("host-boot-test").unwrap(),
            revision,
            produced_at_monotonic_ms: 100,
            carrier_state: carrier,
            front_grip: unknown(),
            rear_grip: unknown(),
            motion: Fact::<MotionClass>::Unknown {
                reason: UnknownReason::NeverObserved,
                since_revision: 0,
            },
            pose: Fact::<Pose>::Unknown {
                reason: UnknownReason::NeverObserved,
                since_revision: 0,
            },
            primary_control: control,
            node_health: Vec::<NodeHealth>::new(),
        }
    }

    #[test]
    fn baseline_never_generates_events() {
        let mut replay = PerceptionReplay::new();
        assert_eq!(
            replay
                .push(state(
                    7,
                    known(CarrierState::Ready),
                    known(Contact::Engaged),
                ))
                .unwrap(),
            ReplayDisposition::BaselineEstablished
        );
    }

    #[test]
    fn held_to_ready_only_enters_ready() {
        let previous = state(1, known(CarrierState::Held), known(Contact::Released));
        let current = state(2, known(CarrierState::Ready), known(Contact::Released));

        assert_eq!(
            derive_event_kinds(&previous, &current),
            vec![DeviceEventKind::CarrierReadyEntered]
        );
    }

    #[test]
    fn unknown_transitions_do_not_guess_edges() {
        let previous = state(1, known(CarrierState::Held), known(Contact::Engaged));
        let current = state(2, unknown(), unknown());

        assert!(derive_event_kinds(&previous, &current).is_empty());
    }

    #[test]
    fn primary_control_edges_stay_independent_from_carrier() {
        let previous = state(1, known(CarrierState::Ready), known(Contact::Released));
        let current = state(2, known(CarrierState::Ready), known(Contact::Engaged));

        assert_eq!(
            derive_event_kinds(&previous, &current),
            vec![DeviceEventKind::ControlPrimaryEngaged]
        );
    }

    #[test]
    fn replay_standard_lifecycle_is_deterministic() {
        let mut replay = PerceptionReplay::new();
        assert_eq!(
            replay
                .push(state(
                    0,
                    known(CarrierState::Standby),
                    known(Contact::Released)
                ))
                .unwrap(),
            ReplayDisposition::BaselineEstablished
        );
        assert_eq!(
            replay
                .push(state(
                    1,
                    known(CarrierState::Held),
                    known(Contact::Released)
                ))
                .unwrap(),
            ReplayDisposition::Advanced(vec![DeviceEventKind::CarrierHeldEntered])
        );
        assert_eq!(
            replay
                .push(state(
                    2,
                    known(CarrierState::Ready),
                    known(Contact::Engaged)
                ))
                .unwrap(),
            ReplayDisposition::Advanced(vec![
                DeviceEventKind::CarrierReadyEntered,
                DeviceEventKind::ControlPrimaryEngaged,
            ])
        );
    }

    #[test]
    fn replay_rejects_revision_regression_without_losing_latest_state() {
        let mut replay = PerceptionReplay::new();
        replay
            .push(state(
                3,
                known(CarrierState::Held),
                known(Contact::Released),
            ))
            .unwrap();

        assert_eq!(
            replay.push(state(
                2,
                known(CarrierState::Standby),
                known(Contact::Released),
            )),
            Err(ReplayError::RevisionRegressed {
                previous: 3,
                current: 2,
            })
        );
        assert_eq!(replay.latest().unwrap().revision, 3);
    }

    #[test]
    fn replay_allows_semantically_identical_same_revision_resends() {
        let mut replay = PerceptionReplay::new();
        let baseline = state(5, known(CarrierState::Held), known(Contact::Released));
        replay.push(baseline.clone()).unwrap();
        let mut republished = baseline;
        republished.produced_at_monotonic_ms += 250;
        if let Fact::Known {
            observed_at_monotonic_ms,
            fresh_until_monotonic_ms,
            ..
        } = &mut republished.carrier_state
        {
            *observed_at_monotonic_ms += 250;
            *fresh_until_monotonic_ms += 250;
        }
        assert_eq!(
            replay.push(republished),
            Ok(ReplayDisposition::SnapshotResent)
        );
        assert!(matches!(
            replay.latest().unwrap().carrier_state,
            Fact::Known {
                observed_at_monotonic_ms: 350,
                ..
            }
        ));
        assert_eq!(
            replay.push(state(
                5,
                known(CarrierState::Ready),
                known(Contact::Released),
            )),
            Err(ReplayError::SameRevisionChanged { revision: 5 })
        );
    }

    #[test]
    fn replay_rejects_invalid_state_without_establishing_a_baseline() {
        let mut replay = PerceptionReplay::new();
        let mut invalid = state(1, unknown(), unknown());
        invalid.carrier_state = Fact::Unknown {
            reason: UnknownReason::NeverObserved,
            since_revision: 2,
        };

        assert!(matches!(
            replay.push(invalid),
            Err(ReplayError::InvalidState(_))
        ));
        assert!(replay.latest().is_none());
    }

    #[test]
    fn replay_host_reboot_reestablishes_quiet_baseline() {
        let mut replay = PerceptionReplay::new();
        replay
            .push(state(
                9,
                known(CarrierState::Ready),
                known(Contact::Released),
            ))
            .unwrap();
        let mut after_reboot = state(0, known(CarrierState::Ready), known(Contact::Released));
        after_reboot.host_boot_id = BootId::try_new("host-boot-after-restart").unwrap();

        assert_eq!(
            replay.push(after_reboot),
            Ok(ReplayDisposition::BaselineEstablished)
        );
    }
}
