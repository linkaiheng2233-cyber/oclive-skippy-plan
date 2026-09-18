//! A.I.Live ai枪娘器灵跨组件契约。
//!
//! 本crate只包含纯值类型、语义校验和由同一Rust类型生成的JSON Schema。
//! 它不得依赖OCLive、BLE、Linux设备、数据库或async runtime。

use schemars::{schema_for, JsonSchema};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::fmt;

pub const CONTRACT_VERSION_V0_2: &str = "0.2";
pub const MAX_SOURCE_REFS: usize = 4;
pub const MAX_NODE_HEALTH_ENTRIES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ContractVersion {
    #[serde(rename = "0.2")]
    V0_2,
}

macro_rules! string_id {
    ($name:ident, $max:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(#[schemars(length(min = 1, max = $max))] String);

        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                validate_identifier_value(&value, $max)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            fn validate_at(&self, path: &str, errors: &mut Vec<ValidationError>) {
                validate_identifier(path, &self.0, $max, errors);
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_new(value).map_err(D::Error::custom)
            }
        }
    };
}

string_id!(ObservationId, 128);
string_id!(EventId, 128);
string_id!(NodeUid, 128);
string_id!(BootId, 128);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierError {
    Empty,
    TooLong {
        maximum_bytes: usize,
        actual_bytes: usize,
    },
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "identifier must not be empty"),
            Self::TooLong {
                maximum_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "identifier is {actual_bytes} UTF-8 bytes; maximum is {maximum_bytes}"
            ),
        }
    }
}

impl std::error::Error for IdentifierError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct Permille(#[schemars(range(max = 1000))] u16);

impl Permille {
    pub const MIN: u16 = 0;
    pub const MAX: u16 = 1000;

    pub fn new(value: u16) -> Result<Self, PermilleOutOfRange> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(PermilleOutOfRange(value))
        }
    }

    pub fn get(self) -> u16 {
        self.0
    }

    pub fn complement(self) -> Self {
        Self(Self::MAX - self.0)
    }

    pub fn minimum(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl<'de> Deserialize<'de> for Permille {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermilleOutOfRange(pub u16);

impl fmt::Display for PermilleOutOfRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "permille {} is outside 0..=1000", self.0)
    }
}

impl std::error::Error for PermilleOutOfRange {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Contact {
    Engaged,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MotionClass {
    Idle,
    Moving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Pose {
    Raised,
    Lowered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CarrierState {
    Standby,
    Held,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryControlId {
    PrimaryTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SensorObservationKind {
    GripContact {
        contact: Contact,
        strength_milli: Permille,
    },
    MotionClass {
        motion: MotionClass,
    },
    OrientationEstimate {
        /// Filtered gravity direction in milli-g. Exact wire bounds remain a measured DTO decision.
        gravity_mg_x: i32,
        gravity_mg_y: i32,
        gravity_mg_z: i32,
        quality_milli: Permille,
    },
    AuxControlContact {
        control_id: PrimaryControlId,
        contact: Contact,
    },
    ShockObserved {
        severity_milli: Permille,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SensorObservation {
    pub contract_version: ContractVersion,
    pub observation_id: ObservationId,
    pub node_uid: NodeUid,
    pub boot_id: BootId,
    pub sequence: u64,
    pub node_monotonic_ms: u64,
    pub observation: SensorObservationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReason {
    NeverObserved,
    NodeOffline,
    Stale,
    Uncalibrated,
    SensorFault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Contradicts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    FrontGrip,
    RearGrip,
    Motion,
    Orientation,
    PrimaryControl,
    ShockDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    pub observation_id: ObservationId,
    pub node_uid: NodeUid,
    pub capability: Capability,
    pub relation: EvidenceRelation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Fact<T> {
    Known {
        value: T,
        observed_at_monotonic_ms: u64,
        fresh_until_monotonic_ms: u64,
        confidence_milli: Permille,
        #[schemars(length(max = 4))]
        source_refs: Vec<SourceRef>,
    },
    Unknown {
        reason: UnknownReason,
        since_revision: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeHealthState {
    Healthy,
    Degraded,
    Offline,
    IncompatibleProtocol,
    ProtocolFault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NodeHealth {
    pub node_uid: NodeUid,
    pub state: NodeHealthState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PerceptionState {
    pub contract_version: ContractVersion,
    pub host_boot_id: BootId,
    pub revision: u64,
    pub produced_at_monotonic_ms: u64,
    pub carrier_state: Fact<CarrierState>,
    pub front_grip: Fact<Contact>,
    pub rear_grip: Fact<Contact>,
    pub motion: Fact<MotionClass>,
    pub pose: Fact<Pose>,
    pub primary_control: Fact<Contact>,
    #[schemars(length(max = 8))]
    pub node_health: Vec<NodeHealth>,
}

impl PerceptionState {
    /// Compare the revision-bearing semantics of two snapshots.
    ///
    /// A fresh observation may extend evidence timestamps without changing the classified value,
    /// unknown reason, node health, or revision. Those evidence-envelope updates are deliberately
    /// ignored here so a refreshed snapshot cannot create a semantic edge. Confidence and source
    /// references remain available to consumers, but are not themselves state-machine revisions.
    pub fn has_same_revision_payload(&self, other: &Self) -> bool {
        self.contract_version == other.contract_version
            && self.host_boot_id == other.host_boot_id
            && self.revision == other.revision
            && fact_has_same_semantics(&self.carrier_state, &other.carrier_state)
            && fact_has_same_semantics(&self.front_grip, &other.front_grip)
            && fact_has_same_semantics(&self.rear_grip, &other.rear_grip)
            && fact_has_same_semantics(&self.motion, &other.motion)
            && fact_has_same_semantics(&self.pose, &other.pose)
            && fact_has_same_semantics(&self.primary_control, &other.primary_control)
            && self.node_health == other.node_health
    }
}

fn fact_has_same_semantics<T: PartialEq>(left: &Fact<T>, right: &Fact<T>) -> bool {
    match (left, right) {
        (Fact::Known { value: left, .. }, Fact::Known { value: right, .. }) => left == right,
        (
            Fact::Unknown {
                reason: left_reason,
                ..
            },
            Fact::Unknown {
                reason: right_reason,
                ..
            },
        ) => left_reason == right_reason,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DeviceEventKind {
    #[serde(rename = "carrier.held_entered")]
    CarrierHeldEntered,
    #[serde(rename = "carrier.held_exited")]
    CarrierHeldExited,
    #[serde(rename = "carrier.ready_entered")]
    CarrierReadyEntered,
    #[serde(rename = "carrier.ready_exited")]
    CarrierReadyExited,
    #[serde(rename = "control.primary.engaged")]
    ControlPrimaryEngaged,
    #[serde(rename = "control.primary.released")]
    ControlPrimaryReleased,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceEvent {
    pub contract_version: ContractVersion,
    pub event_id: EventId,
    pub host_boot_id: BootId,
    pub occurred_at_monotonic_ms: u64,
    pub caused_by_perception_revision: u64,
    pub kind: DeviceEventKind,
}

pub trait ContractValidate {
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl ContractValidate for SensorObservation {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        self.observation_id
            .validate_at("observation_id", &mut errors);
        self.node_uid.validate_at("node_uid", &mut errors);
        self.boot_id.validate_at("boot_id", &mut errors);
        finish_validation(errors)
    }
}

impl<T> Fact<T> {
    fn validate_at(&self, path: &str, revision: u64, errors: &mut Vec<ValidationError>) {
        match self {
            Fact::Known {
                observed_at_monotonic_ms,
                fresh_until_monotonic_ms,
                source_refs,
                ..
            } => {
                if observed_at_monotonic_ms > fresh_until_monotonic_ms {
                    errors.push(ValidationError::new(
                        path,
                        "observed_at_monotonic_ms must not exceed fresh_until_monotonic_ms",
                    ));
                }
                if source_refs.len() > MAX_SOURCE_REFS {
                    errors.push(ValidationError::new(
                        format!("{path}.source_refs"),
                        format!("must contain at most {MAX_SOURCE_REFS} entries"),
                    ));
                }
                for (index, source) in source_refs.iter().enumerate() {
                    source.observation_id.validate_at(
                        &format!("{path}.source_refs[{index}].observation_id"),
                        errors,
                    );
                    source
                        .node_uid
                        .validate_at(&format!("{path}.source_refs[{index}].node_uid"), errors);
                }
            }
            Fact::Unknown { since_revision, .. } => {
                if *since_revision > revision {
                    errors.push(ValidationError::new(
                        format!("{path}.since_revision"),
                        "must not exceed the containing PerceptionState revision",
                    ));
                }
            }
        }
    }
}

impl ContractValidate for PerceptionState {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        self.host_boot_id.validate_at("host_boot_id", &mut errors);
        self.carrier_state
            .validate_at("carrier_state", self.revision, &mut errors);
        self.front_grip
            .validate_at("front_grip", self.revision, &mut errors);
        self.rear_grip
            .validate_at("rear_grip", self.revision, &mut errors);
        self.motion
            .validate_at("motion", self.revision, &mut errors);
        self.pose.validate_at("pose", self.revision, &mut errors);
        self.primary_control
            .validate_at("primary_control", self.revision, &mut errors);

        if self.node_health.len() > MAX_NODE_HEALTH_ENTRIES {
            errors.push(ValidationError::new(
                "node_health",
                format!("must contain at most {MAX_NODE_HEALTH_ENTRIES} entries"),
            ));
        }
        let mut node_uids = HashSet::new();
        for (index, health) in self.node_health.iter().enumerate() {
            health
                .node_uid
                .validate_at(&format!("node_health[{index}].node_uid"), &mut errors);
            if !node_uids.insert(health.node_uid.as_str()) {
                errors.push(ValidationError::new(
                    format!("node_health[{index}].node_uid"),
                    "duplicate node_uid",
                ));
            }
        }
        finish_validation(errors)
    }
}

impl ContractValidate for DeviceEvent {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        self.event_id.validate_at("event_id", &mut errors);
        self.host_boot_id.validate_at("host_boot_id", &mut errors);
        if self.caused_by_perception_revision == 0 {
            errors.push(ValidationError::new(
                "caused_by_perception_revision",
                "DeviceEvent cannot be caused by baseline revision 0",
            ));
        }
        finish_validation(errors)
    }
}

fn validate_identifier(
    path: &str,
    value: &str,
    maximum_length: usize,
    errors: &mut Vec<ValidationError>,
) {
    if value.is_empty() {
        errors.push(ValidationError::new(path, "must not be empty"));
    } else if value.len() > maximum_length {
        errors.push(ValidationError::new(
            path,
            format!("must not exceed {maximum_length} UTF-8 bytes"),
        ));
    }
}

fn validate_identifier_value(value: &str, maximum_length: usize) -> Result<(), IdentifierError> {
    if value.is_empty() {
        Err(IdentifierError::Empty)
    } else if value.len() > maximum_length {
        Err(IdentifierError::TooLong {
            maximum_bytes: maximum_length,
            actual_bytes: value.len(),
        })
    } else {
        Ok(())
    }
}

fn finish_validation(errors: Vec<ValidationError>) -> Result<(), Vec<ValidationError>> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedSchema {
    pub file_name: &'static str,
    pub json: String,
}

#[derive(Debug)]
pub enum SchemaGenerationError {
    Json(serde_json::Error),
    RootIsNotObject { file_name: &'static str },
}

impl fmt::Display for SchemaGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "cannot render JSON Schema: {error}"),
            Self::RootIsNotObject { file_name } => {
                write!(
                    formatter,
                    "generated schema {file_name} is not a JSON object"
                )
            }
        }
    }
}

impl std::error::Error for SchemaGenerationError {}

impl From<serde_json::Error> for SchemaGenerationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn generated_schema_documents() -> Result<Vec<GeneratedSchema>, SchemaGenerationError> {
    Ok(vec![
        render_schema::<SensorObservation>(
            "sensor-observation.v0.2.schema.json",
            "https://oclive.dev/schemas/ailive.gun-spirit/sensor-observation.v0.2.schema.json",
            "A.I.Live Gun Spirit SensorObservation v0.2",
        )?,
        render_schema::<PerceptionState>(
            "perception-state.v0.2.schema.json",
            "https://oclive.dev/schemas/ailive.gun-spirit/perception-state.v0.2.schema.json",
            "A.I.Live Gun Spirit PerceptionState v0.2",
        )?,
        render_schema::<DeviceEvent>(
            "device-event.v0.2.schema.json",
            "https://oclive.dev/schemas/ailive.gun-spirit/device-event.v0.2.schema.json",
            "A.I.Live Gun Spirit DeviceEvent v0.2",
        )?,
    ])
}

fn render_schema<T: JsonSchema>(
    file_name: &'static str,
    schema_id: &str,
    title: &str,
) -> Result<GeneratedSchema, SchemaGenerationError> {
    let mut schema = serde_json::to_value(schema_for!(T))?;
    let object = schema
        .as_object_mut()
        .ok_or(SchemaGenerationError::RootIsNotObject { file_name })?;
    object.insert("$id".to_owned(), schema_id.into());
    object.insert("title".to_owned(), title.into());
    let mut json = serde_json::to_string_pretty(&schema)?;
    json.push('\n');
    Ok(GeneratedSchema { file_name, json })
}
