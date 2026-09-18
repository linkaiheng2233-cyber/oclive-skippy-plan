//! Narrow adapter from neutral gun-spirit sensor turns to OCLive role turns.

use std::error::Error;
use std::fmt::{Display, Formatter};

use oclive_kernel_host::domain::chat_engine::process_message_with_origin;
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::{SendMessageRequest, TurnOrigin};
use serde::Serialize;

use crate::{RoleCue, RoleCueSource, SensorTurnRequest};

pub const OCLIVE_SENSOR_CONTEXT_SCHEMA: &str = "oclive.sensor_context.v1";

/// OCLive role/session selected by the device host.
///
/// `session_id` should be stable for one physical device installation. It keeps runtime lookup
/// separate from desktop chat even though sensor turns themselves are non-persistent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcliveSensorTarget {
    pub role_id: String,
    pub scene_id: Option<String>,
    pub session_id: String,
}

#[derive(Debug)]
pub enum OcliveSensorAdapterError {
    Serialize(serde_json::Error),
    Kernel(oclive_kernel_host::error::AppError),
}

impl Display for OcliveSensorAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "serialize sensor context: {error}"),
            Self::Kernel(error) => write!(formatter, "OCLive sensor turn: {error}"),
        }
    }
}

impl Error for OcliveSensorAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialize(error) => Some(error),
            Self::Kernel(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for OcliveSensorAdapterError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value)
    }
}

impl From<oclive_kernel_host::error::AppError> for OcliveSensorAdapterError {
    fn from(value: oclive_kernel_host::error::AppError) -> Self {
        Self::Kernel(value)
    }
}

#[derive(Debug, Serialize)]
struct SensorContextEnvelope<'a> {
    schema: &'static str,
    input_kind: &'static str,
    interpretation: &'static str,
    events: &'a [ailive_gun_spirit_contracts::DeviceEvent],
    device_context: &'a crate::DeviceContext,
}

/// The only production seam allowed to know both kernels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcliveSensorAdapter {
    target: OcliveSensorTarget,
}

impl OcliveSensorAdapter {
    #[must_use]
    pub fn new(target: OcliveSensorTarget) -> Self {
        Self { target }
    }

    /// Builds the bounded semantic body consumed by OCLive. Classification remains the typed
    /// [`TurnOrigin::Sensor`] argument used by [`Self::execute`], not a text prefix.
    pub fn build_message_request(
        &self,
        request: &SensorTurnRequest,
    ) -> Result<SendMessageRequest, OcliveSensorAdapterError> {
        let envelope = SensorContextEnvelope {
            schema: OCLIVE_SENSOR_CONTEXT_SCHEMA,
            input_kind: "sensor_context",
            interpretation: "neutral_device_facts_not_user_speech",
            events: request.events.as_slice(),
            device_context: &request.device_context,
        };
        Ok(SendMessageRequest {
            role_id: self.target.role_id.clone(),
            user_message: serde_json::to_string(&envelope)?,
            scene_id: self.target.scene_id.clone(),
            session_id: Some(self.target.session_id.clone()),
            ..Default::default()
        })
    }

    /// Runs one sensor-origin role turn and projects OCLive output into the host-owned role cue.
    pub async fn execute(
        &self,
        state: &AppState,
        request: &SensorTurnRequest,
    ) -> Result<RoleCue, OcliveSensorAdapterError> {
        let message = self.build_message_request(request)?;
        let response = process_message_with_origin(state, &message, TurnOrigin::Sensor).await?;
        let visual_state_id = response
            .visual_state_id
            .unwrap_or(response.portrait_emotion);
        Ok(RoleCue {
            source: RoleCueSource::Oclive,
            context_revision: request.device_context.perception_revision,
            source_event_ids: request
                .events
                .iter()
                .map(|event| event.event_id.clone())
                .collect(),
            visual_state_id,
            text: response.reply,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::{DeviceContext, InputOrigin, NodeIndicator, UiFact};
    use ailive_gun_spirit_contracts::{
        BootId, CarrierState, Contact, ContractVersion, DeviceEvent, DeviceEventKind, EventId,
    };

    fn request() -> SensorTurnRequest {
        SensorTurnRequest {
            origin: InputOrigin::Sensor,
            events: vec![DeviceEvent {
                contract_version: ContractVersion::V0_2,
                event_id: EventId::try_new("event-1").unwrap(),
                host_boot_id: BootId::try_new("boot-1").unwrap(),
                occurred_at_monotonic_ms: 100,
                caused_by_perception_revision: 7,
                kind: DeviceEventKind::CarrierHeldEntered,
            }],
            device_context: DeviceContext {
                perception_revision: 7,
                carrier_state: UiFact::Known {
                    value: CarrierState::Held,
                },
                primary_control: UiFact::Known {
                    value: Contact::Released,
                },
                node_health: Vec::<NodeIndicator>::new(),
                recent_event_kinds: vec![DeviceEventKind::CarrierHeldEntered],
            },
        }
    }

    #[test]
    fn adapter_emits_machine_readable_neutral_context() {
        let adapter = OcliveSensorAdapter::new(OcliveSensorTarget {
            role_id: "mumu".to_owned(),
            scene_id: Some("default".to_owned()),
            session_id: "device-radian-01".to_owned(),
        });
        let message = adapter.build_message_request(&request()).unwrap();
        let body: serde_json::Value = serde_json::from_str(&message.user_message).unwrap();

        assert_eq!(body["schema"], OCLIVE_SENSOR_CONTEXT_SCHEMA);
        assert_eq!(body["input_kind"], "sensor_context");
        assert_eq!(
            body["interpretation"],
            "neutral_device_facts_not_user_speech"
        );
        assert_eq!(body["device_context"]["perception_revision"], 7);
        assert_eq!(message.session_id.as_deref(), Some("device-radian-01"));
    }
}
