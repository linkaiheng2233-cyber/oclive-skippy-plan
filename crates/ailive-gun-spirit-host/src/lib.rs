//! Host-owned projections and output arbitration.
//!
//! This crate is the composition boundary. It may know both the neutral gun-spirit contracts and
//! OCLive adapters, while neither kernel is allowed to depend on the other.

mod oclive_adapter;

pub use oclive_adapter::{
    OcliveSensorAdapter, OcliveSensorAdapterError, OcliveSensorTarget, OCLIVE_SENSOR_CONTEXT_SCHEMA,
};

use ailive_gun_spirit_contracts::{
    CarrierState, Contact, DeviceEvent, DeviceEventKind, EventId, Fact, MotionClass,
    NodeHealthState, PerceptionState, Pose, UnknownReason,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UiFact<T> {
    Known { value: T },
    Unknown { reason: UnknownReason },
}

impl<T: Clone> From<&Fact<T>> for UiFact<T> {
    fn from(fact: &Fact<T>) -> Self {
        match fact {
            Fact::Known { value, .. } => Self::Known {
                value: value.clone(),
            },
            Fact::Unknown { reason, .. } => Self::Unknown { reason: *reason },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleCueSource {
    DesktopFixture,
    Oclive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleCue {
    pub source: RoleCueSource,
    pub context_revision: u64,
    pub source_event_ids: Vec<EventId>,
    pub visual_state_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputOrigin {
    Sensor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceContext {
    pub perception_revision: u64,
    pub carrier_state: UiFact<CarrierState>,
    pub primary_control: UiFact<Contact>,
    pub node_health: Vec<NodeIndicator>,
    pub recent_event_kinds: Vec<DeviceEventKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SensorTurnRequest {
    pub origin: InputOrigin,
    pub events: Vec<DeviceEvent>,
    pub device_context: DeviceContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputScheduleError {
    HostBootMismatch,
    TooManyEvents {
        count: usize,
        max: usize,
    },
    TooManyNodes {
        count: usize,
        max: usize,
    },
    FuturePerceptionRevision {
        event_revision: u64,
        current_revision: u64,
    },
}

/// Minimal desktop scheduling seam.
///
/// Probability, cooldown, voice merge and companion-pack rules intentionally remain outside this
/// fixture seam. The important invariant is already executable: a sensor turn has a typed origin
/// and bounded neutral context, never a fake user-text prefix.
#[derive(Debug, Default)]
pub struct InputScheduler;

impl InputScheduler {
    pub const MAX_EVENTS_PER_TURN: usize = 8;
    pub const MAX_NODES_PER_TURN: usize = 8;

    pub fn schedule_sensor_turn(
        perception: &PerceptionState,
        events: &[DeviceEvent],
    ) -> Result<Option<SensorTurnRequest>, InputScheduleError> {
        if events.is_empty() {
            return Ok(None);
        }
        if events.len() > Self::MAX_EVENTS_PER_TURN {
            return Err(InputScheduleError::TooManyEvents {
                count: events.len(),
                max: Self::MAX_EVENTS_PER_TURN,
            });
        }
        if perception.node_health.len() > Self::MAX_NODES_PER_TURN {
            return Err(InputScheduleError::TooManyNodes {
                count: perception.node_health.len(),
                max: Self::MAX_NODES_PER_TURN,
            });
        }
        for event in events {
            if event.host_boot_id != perception.host_boot_id {
                return Err(InputScheduleError::HostBootMismatch);
            }
            if event.caused_by_perception_revision > perception.revision {
                return Err(InputScheduleError::FuturePerceptionRevision {
                    event_revision: event.caused_by_perception_revision,
                    current_revision: perception.revision,
                });
            }
        }

        Ok(Some(SensorTurnRequest {
            origin: InputOrigin::Sensor,
            events: events.to_vec(),
            device_context: DeviceContext {
                perception_revision: perception.revision,
                carrier_state: UiFact::from(&perception.carrier_state),
                primary_control: UiFact::from(&perception.primary_control),
                node_health: node_indicators(perception),
                recent_event_kinds: events.iter().map(|event| event.kind).collect(),
            },
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SystemCue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrimaryScreenLayer {
    Status,
    Role {
        source: RoleCueSource,
        visual_state_id: String,
        text: String,
    },
    System {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeIndicator {
    pub node_uid: String,
    pub state: NodeHealthState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScreenViewModel {
    pub perception_revision: u64,
    pub carrier_state: UiFact<CarrierState>,
    pub front_grip: UiFact<Contact>,
    pub rear_grip: UiFact<Contact>,
    pub motion: UiFact<MotionClass>,
    pub pose: UiFact<Pose>,
    pub primary_control: UiFact<Contact>,
    pub nodes: Vec<NodeIndicator>,
    pub primary_layer: PrimaryScreenLayer,
}

/// The only place that decides what the screen presents as its primary layer.
///
/// System cues preempt role cues. A role cue is accepted only for the exact perception revision it
/// was generated from; later schedulers may add a bounded revision/TTL policy without changing the
/// perception kernel.
#[derive(Debug, Default)]
pub struct OutputArbiter;

impl OutputArbiter {
    pub fn render(
        perception: &PerceptionState,
        role_cue: Option<&RoleCue>,
        system_cue: Option<&SystemCue>,
    ) -> ScreenViewModel {
        let primary_layer = if let Some(system_cue) = system_cue {
            PrimaryScreenLayer::System {
                code: system_cue.code.clone(),
                message: system_cue.message.clone(),
            }
        } else if let Some(role_cue) =
            role_cue.filter(|cue| cue.context_revision == perception.revision)
        {
            PrimaryScreenLayer::Role {
                source: role_cue.source,
                visual_state_id: role_cue.visual_state_id.clone(),
                text: role_cue.text.clone(),
            }
        } else {
            PrimaryScreenLayer::Status
        };

        ScreenViewModel {
            perception_revision: perception.revision,
            carrier_state: UiFact::from(&perception.carrier_state),
            front_grip: UiFact::from(&perception.front_grip),
            rear_grip: UiFact::from(&perception.rear_grip),
            motion: UiFact::from(&perception.motion),
            pose: UiFact::from(&perception.pose),
            primary_control: UiFact::from(&perception.primary_control),
            nodes: node_indicators(perception),
            primary_layer,
        }
    }
}

fn node_indicators(perception: &PerceptionState) -> Vec<NodeIndicator> {
    perception
        .node_health
        .iter()
        .map(|node| NodeIndicator {
            node_uid: node.node_uid.as_str().to_owned(),
            state: node.state,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ailive_gun_spirit_contracts::{
        BootId, ContractVersion, DeviceEvent, DeviceEventKind, EventId, NodeHealth, Permille,
        SourceRef,
    };

    fn known<T>(value: T) -> Fact<T> {
        Fact::Known {
            value,
            observed_at_monotonic_ms: 10,
            fresh_until_monotonic_ms: 100,
            confidence_milli: Permille::new(900).unwrap(),
            source_refs: Vec::<SourceRef>::new(),
        }
    }

    fn state(revision: u64) -> PerceptionState {
        PerceptionState {
            contract_version: ContractVersion::V0_2,
            host_boot_id: BootId::try_new("host-boot").unwrap(),
            revision,
            produced_at_monotonic_ms: 10,
            carrier_state: known(CarrierState::Held),
            front_grip: known(Contact::Released),
            rear_grip: known(Contact::Engaged),
            motion: known(MotionClass::Idle),
            pose: known(Pose::Lowered),
            primary_control: known(Contact::Released),
            node_health: Vec::<NodeHealth>::new(),
        }
    }

    fn role_cue(revision: u64) -> RoleCue {
        RoleCue {
            source: RoleCueSource::DesktopFixture,
            context_revision: revision,
            source_event_ids: Vec::new(),
            visual_state_id: "fixture.held".to_owned(),
            text: "桌面夹具回应".to_owned(),
        }
    }

    fn event(revision: u64) -> DeviceEvent {
        DeviceEvent {
            contract_version: ContractVersion::V0_2,
            event_id: EventId::try_new("event-1").unwrap(),
            host_boot_id: BootId::try_new("host-boot").unwrap(),
            occurred_at_monotonic_ms: 10,
            caused_by_perception_revision: revision,
            kind: DeviceEventKind::CarrierHeldEntered,
        }
    }

    #[test]
    fn sensor_scheduler_keeps_origin_typed_and_context_bounded() {
        let request = InputScheduler::schedule_sensor_turn(&state(2), &[event(2)])
            .unwrap()
            .unwrap();
        assert_eq!(request.origin, InputOrigin::Sensor);
        assert_eq!(request.device_context.perception_revision, 2);
        assert_eq!(
            request.device_context.recent_event_kinds,
            vec![DeviceEventKind::CarrierHeldEntered]
        );
    }

    #[test]
    fn no_event_does_not_open_a_sensor_turn() {
        assert_eq!(
            InputScheduler::schedule_sensor_turn(&state(2), &[]).unwrap(),
            None
        );
    }

    #[test]
    fn current_role_cue_controls_primary_layer() {
        let view = OutputArbiter::render(&state(2), Some(&role_cue(2)), None);
        assert!(matches!(
            view.primary_layer,
            PrimaryScreenLayer::Role { .. }
        ));
    }

    #[test]
    fn stale_role_cue_cannot_overwrite_newer_state() {
        let view = OutputArbiter::render(&state(3), Some(&role_cue(2)), None);
        assert_eq!(view.primary_layer, PrimaryScreenLayer::Status);
    }

    #[test]
    fn system_cue_preempts_role_cue() {
        let system = SystemCue {
            code: "node.offline".to_owned(),
            message: "节点离线".to_owned(),
        };
        let view = OutputArbiter::render(&state(2), Some(&role_cue(2)), Some(&system));
        assert!(matches!(
            view.primary_layer,
            PrimaryScreenLayer::System { .. }
        ));
    }
}
