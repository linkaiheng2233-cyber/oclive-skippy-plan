//! Hardware-free SensorObservation -> PerceptionState -> DeviceEvent -> screen simulation.

use ailive_gun_spirit_contracts::{
    BootId, Capability, Contact, ContractVersion, DeviceEvent, DeviceEventKind, EventId,
    GripBinding, MotionClass, NodeUid, ObservationId, PerceptionState, Permille, Pose,
    PrimaryControlId, SensorObservation, SensorObservationKind,
};
use ailive_gun_spirit_host::{
    InputScheduler, OutputArbiter, RoleCue, RoleCueSource, SensorTurnRequest, SystemCue,
};
use ailive_gun_spirit_perception_core::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer, PerceptionReplay, ReductionOutput, ReplayDisposition,
};
use serde_json::json;

const FRONT_NODE_UID: &str = "mock-front-node";
const REAR_NODE_UID: &str = "mock-rear-node";
const FACT_TTL_MS: u64 = 1_000;
const NODE_OFFLINE_TTL_MS: u64 = 1_500;
const STANDBY_DWELL_MS: u64 = 100;
/// One observation cadence: used by the `tick` command to advance the Host clock
/// without delivering anything new.
const TICK_STEP_MS: u64 = 10;
const DEFAULT_COMMANDS: &[&str] = &[
    "baseline",
    "held",
    "ready",
    "control-on",
    "control-off",
    "reboot",
    "lower",
    "standby",
    "unknown",
];
/// Duplicate-input coverage. A repeated value command re-observes the same
/// semantic value with a fresh sequence (`SnapshotRefreshed`, same revision);
/// a `duplicate` command re-delivers the exact same `SensorObservation`
/// (`DuplicateIgnored`, same revision); a `tick` publishes the unchanged
/// snapshot again (`SnapshotResent`). All three must stay `resend` frames.
const REPEAT_COMMANDS: &[&str] = &[
    "baseline",
    "held",
    "held",
    "duplicate",
    "tick",
    "ready",
    "duplicate",
    "control-on",
    "control-on",
    "duplicate",
    "control-off",
];
/// TTL coverage. `stale` jumps past the fact TTL while both nodes are still
/// inside the node-offline TTL, so facts degrade to `Stale` and the carrier
/// becomes `Unknown(Stale)` instead of guessing a release. The following
/// `held` proves a fresh observation recovers, and `unknown` crosses the
/// node-offline TTL to show `NodeOffline` taking precedence.
const STALE_COMMANDS: &[&str] = &["baseline", "ready", "stale", "held", "duplicate", "unknown"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Default,
    Repeat,
    Stale,
}

impl Scenario {
    fn commands(self) -> &'static [&'static str] {
        match self {
            Self::Default => DEFAULT_COMMANDS,
            Self::Repeat => REPEAT_COMMANDS,
            Self::Stale => STALE_COMMANDS,
        }
    }
}

fn scenario_from_name(name: &str) -> Result<Scenario, String> {
    match name {
        "default" => Ok(Scenario::Default),
        "repeat" => Ok(Scenario::Repeat),
        "stale" => Ok(Scenario::Stale),
        other => Err(format!(
            "unknown scenario {other:?}; expected default, repeat or stale"
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockPhysicalState {
    front_grip: Contact,
    rear_grip: Contact,
    motion: MotionClass,
    pose: Pose,
    primary_control: Contact,
}

impl Default for MockPhysicalState {
    fn default() -> Self {
        Self {
            front_grip: Contact::Released,
            rear_grip: Contact::Released,
            motion: MotionClass::Idle,
            pose: Pose::Lowered,
            primary_control: Contact::Released,
        }
    }
}

#[derive(Debug, Clone)]
struct MockNode {
    uid: NodeUid,
    boot_id: BootId,
    sequence: u64,
}

impl MockNode {
    fn new(uid: &str) -> Result<Self, String> {
        Ok(Self {
            uid: NodeUid::try_new(uid).map_err(|error| error.to_string())?,
            boot_id: BootId::try_new(format!("{uid}-boot-1")).map_err(|error| error.to_string())?,
            sequence: 0,
        })
    }

    fn observe(
        &mut self,
        node_monotonic_ms: u64,
        observation: SensorObservationKind,
    ) -> Result<SensorObservation, String> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| "mock node sequence overflow".to_owned())?;
        Ok(SensorObservation {
            contract_version: ContractVersion::V0_4,
            observation_id: ObservationId::try_new(format!(
                "{}:{}:{}",
                self.uid.as_str(),
                self.boot_id.as_str(),
                self.sequence
            ))
            .map_err(|error| error.to_string())?,
            node_uid: self.uid.clone(),
            boot_id: self.boot_id.clone(),
            sequence: self.sequence,
            node_monotonic_ms,
            observation,
        })
    }
}

struct DesktopFixture {
    host_boot_number: u64,
    now_ms: u64,
    physical: MockPhysicalState,
    front: MockNode,
    rear: MockNode,
    reducer: PerceptionReducer,
    replay: PerceptionReplay,
    event_sequence: u64,
    /// Exact last consumed observation, used by the `duplicate` command to
    /// replay the same `observation_id`/`sequence` at a later Host time.
    last_observation: Option<SensorObservation>,
}

impl DesktopFixture {
    fn new() -> Result<Self, String> {
        let front = MockNode::new(FRONT_NODE_UID)?;
        let rear = MockNode::new(REAR_NODE_UID)?;
        let reducer = new_reducer(1, 0, &front.uid, &rear.uid)?;
        Ok(Self {
            host_boot_number: 1,
            now_ms: 0,
            physical: MockPhysicalState::default(),
            front,
            rear,
            reducer,
            replay: PerceptionReplay::new(),
            event_sequence: 0,
            last_observation: None,
        })
    }

    fn run_command(&mut self, command: &str) -> Result<serde_json::Value, String> {
        let mut observation_count = 0_usize;
        let mut semantic_advance = false;
        let mut event_kinds = Vec::new();
        let disposition = match command {
            "baseline" => {
                self.physical = MockPhysicalState::default();
                observation_count += self.seed_current_physical()?;
                self.establish_quiet_baseline()?;
                "baseline"
            }
            "reboot" => {
                self.host_boot_number = self
                    .host_boot_number
                    .checked_add(1)
                    .ok_or_else(|| "mock host boot counter overflow".to_owned())?;
                self.reducer = new_reducer(
                    self.host_boot_number,
                    self.now_ms,
                    &self.front.uid,
                    &self.rear.uid,
                )?;
                observation_count += self.seed_current_physical()?;
                self.establish_quiet_baseline()?;
                "baseline"
            }
            "held" => {
                self.physical.rear_grip = Contact::Engaged;
                let observation = self.rear_observation(SensorObservationKind::GripContact {
                    grip: GripBinding::Rear,
                    contact: Contact::Engaged,
                    strength_milli: permille(900)?,
                })?;
                self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                observation_count += 1;
                disposition_name(semantic_advance)
            }
            "ready" => {
                self.physical.front_grip = Contact::Engaged;
                self.physical.rear_grip = Contact::Engaged;
                self.physical.motion = MotionClass::Moving;
                self.physical.pose = Pose::Raised;
                let observations = vec![
                    self.front_observation(SensorObservationKind::GripContact {
                        grip: GripBinding::Front,
                        contact: Contact::Engaged,
                        strength_milli: permille(900)?,
                    })?,
                    self.front_observation(SensorObservationKind::MotionClass {
                        motion: MotionClass::Moving,
                    })?,
                    self.front_observation(orientation(Pose::Raised)?)?,
                    self.rear_observation(SensorObservationKind::GripContact {
                        grip: GripBinding::Rear,
                        contact: Contact::Engaged,
                        strength_milli: permille(900)?,
                    })?,
                ];
                observation_count += observations.len();
                for observation in observations {
                    self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                }
                disposition_name(semantic_advance)
            }
            "control-on" | "control-off" => {
                let contact = if command == "control-on" {
                    Contact::Engaged
                } else {
                    Contact::Released
                };
                self.physical.primary_control = contact;
                let observation =
                    self.rear_observation(SensorObservationKind::AuxControlContact {
                        control_id: PrimaryControlId::PrimaryTrigger,
                        contact,
                    })?;
                self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                observation_count += 1;
                disposition_name(semantic_advance)
            }
            "lower" => {
                self.physical.pose = Pose::Lowered;
                let observation = self.front_observation(orientation(Pose::Lowered)?)?;
                self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                observation_count += 1;
                disposition_name(semantic_advance)
            }
            "standby" => {
                self.physical.front_grip = Contact::Released;
                self.physical.rear_grip = Contact::Released;
                self.physical.motion = MotionClass::Idle;
                let observations = vec![
                    self.front_observation(SensorObservationKind::GripContact {
                        grip: GripBinding::Front,
                        contact: Contact::Released,
                        strength_milli: permille(100)?,
                    })?,
                    self.rear_observation(SensorObservationKind::GripContact {
                        grip: GripBinding::Rear,
                        contact: Contact::Released,
                        strength_milli: permille(100)?,
                    })?,
                    self.front_observation(SensorObservationKind::MotionClass {
                        motion: MotionClass::Idle,
                    })?,
                ];
                observation_count += observations.len();
                for observation in observations {
                    self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                }
                self.now_ms = self.now_ms.saturating_add(STANDBY_DWELL_MS);
                let output = self
                    .reducer
                    .tick(self.now_ms)
                    .map_err(|error| format!("standby tick failed: {error:?}"))?;
                self.consume_output(output, &mut semantic_advance, &mut event_kinds)?;
                disposition_name(semantic_advance)
            }
            "unknown" => {
                self.now_ms = self.now_ms.saturating_add(NODE_OFFLINE_TTL_MS + 1);
                let output = self
                    .reducer
                    .tick(self.now_ms)
                    .map_err(|error| format!("offline tick failed: {error:?}"))?;
                self.consume_output(output, &mut semantic_advance, &mut event_kinds)?;
                disposition_name(semantic_advance)
            }
            "duplicate" => {
                let observation = self.last_observation.clone().ok_or_else(|| {
                    "duplicate requires at least one previously delivered observation".to_owned()
                })?;
                self.consume(observation, &mut semantic_advance, &mut event_kinds)?;
                observation_count += 1;
                disposition_name(semantic_advance)
            }
            "tick" => {
                self.now_ms = self.now_ms.saturating_add(TICK_STEP_MS);
                let output = self
                    .reducer
                    .tick(self.now_ms)
                    .map_err(|error| format!("idle tick failed: {error:?}"))?;
                self.consume_output(output, &mut semantic_advance, &mut event_kinds)?;
                disposition_name(semantic_advance)
            }
            "stale" => {
                // Cross every fact TTL by one millisecond while staying below
                // the node-offline TTL: facts become `Stale`, nodes stay healthy.
                self.now_ms = self.now_ms.saturating_add(FACT_TTL_MS + 1);
                let output = self
                    .reducer
                    .tick(self.now_ms)
                    .map_err(|error| format!("stale tick failed: {error:?}"))?;
                self.consume_output(output, &mut semantic_advance, &mut event_kinds)?;
                disposition_name(semantic_advance)
            }
            unknown => return Err(format!("unknown command {unknown:?}")),
        };

        let snapshot = self.reducer.current_state().clone();
        let device_events = self.materialize_events(&snapshot, &event_kinds)?;
        let sensor_turn = InputScheduler::schedule_sensor_turn(&snapshot, &device_events)
            .map_err(|error| format!("input scheduling failed: {error:?}"))?;
        let role_cue = sensor_turn.as_ref().and_then(desktop_fixture_role_cue);
        let system_cue = offline_system_cue(&snapshot);
        let screen = OutputArbiter::render(&snapshot, role_cue.as_ref(), system_cue.as_ref());

        Ok(json!({
            "command": command,
            "producer_boot_id": snapshot.producer_boot_id.as_str(),
            "revision": snapshot.revision,
            "disposition": disposition,
            "observation_count": observation_count,
            "events": event_kinds,
            "device_events": device_events,
            "perception_state": snapshot,
            "role_bridge": {
                "source": "desktop_fixture",
                "oclive_dispatched": false,
                "sensor_turn": sensor_turn,
                "role_cue": role_cue,
            },
            "screen_view_model": screen,
        }))
    }

    fn seed_current_physical(&mut self) -> Result<usize, String> {
        let observations = vec![
            self.front_observation(SensorObservationKind::GripContact {
                grip: GripBinding::Front,
                contact: self.physical.front_grip,
                strength_milli: contact_strength(self.physical.front_grip)?,
            })?,
            self.front_observation(SensorObservationKind::MotionClass {
                motion: self.physical.motion,
            })?,
            self.front_observation(orientation(self.physical.pose)?)?,
            self.rear_observation(SensorObservationKind::GripContact {
                grip: GripBinding::Rear,
                contact: self.physical.rear_grip,
                strength_milli: contact_strength(self.physical.rear_grip)?,
            })?,
            self.rear_observation(SensorObservationKind::AuxControlContact {
                control_id: PrimaryControlId::PrimaryTrigger,
                contact: self.physical.primary_control,
            })?,
        ];
        let observation_count = observations.len();
        for observation in observations {
            self.now_ms = self.now_ms.saturating_add(10);
            self.reducer
                .ingest(observation.clone(), self.now_ms)
                .map_err(|error| format!("baseline observation failed: {error:?}"))?;
            self.last_observation = Some(observation);
        }
        if self.physical == MockPhysicalState::default() {
            self.now_ms = self.now_ms.saturating_add(STANDBY_DWELL_MS);
            self.reducer
                .tick(self.now_ms)
                .map_err(|error| format!("baseline dwell failed: {error:?}"))?;
        }
        Ok(observation_count)
    }

    fn establish_quiet_baseline(&mut self) -> Result<(), String> {
        self.replay = PerceptionReplay::new();
        let disposition = self
            .replay
            .push(self.reducer.current_state().clone())
            .map_err(|error| format!("cannot establish baseline: {error:?}"))?;
        if disposition != ReplayDisposition::BaselineEstablished {
            return Err("quiet baseline gate returned a non-baseline disposition".to_owned());
        }
        Ok(())
    }

    fn consume(
        &mut self,
        observation: SensorObservation,
        semantic_advance: &mut bool,
        event_kinds: &mut Vec<DeviceEventKind>,
    ) -> Result<(), String> {
        self.now_ms = self.now_ms.saturating_add(10);
        let output = self
            .reducer
            .ingest(observation.clone(), self.now_ms)
            .map_err(|error| format!("observation reduction failed: {error:?}"))?;
        self.last_observation = Some(observation);
        self.consume_output(output, semantic_advance, event_kinds)
    }

    fn consume_output(
        &mut self,
        output: ReductionOutput,
        semantic_advance: &mut bool,
        event_kinds: &mut Vec<DeviceEventKind>,
    ) -> Result<(), String> {
        let replay = self
            .replay
            .push(output.snapshot)
            .map_err(|error| format!("perception replay failed: {error:?}"))?;
        match replay {
            ReplayDisposition::BaselineEstablished | ReplayDisposition::SnapshotResent => {}
            ReplayDisposition::Advanced(events) => {
                *semantic_advance = true;
                event_kinds.extend(events);
            }
        }
        Ok(())
    }

    fn front_observation(
        &mut self,
        observation: SensorObservationKind,
    ) -> Result<SensorObservation, String> {
        self.front.observe(self.now_ms, observation)
    }

    fn rear_observation(
        &mut self,
        observation: SensorObservationKind,
    ) -> Result<SensorObservation, String> {
        self.rear.observe(self.now_ms, observation)
    }

    fn materialize_events(
        &mut self,
        snapshot: &PerceptionState,
        event_kinds: &[DeviceEventKind],
    ) -> Result<Vec<DeviceEvent>, String> {
        event_kinds
            .iter()
            .map(|kind| {
                self.event_sequence = self
                    .event_sequence
                    .checked_add(1)
                    .ok_or_else(|| "event sequence overflow".to_owned())?;
                Ok(DeviceEvent {
                    contract_version: ContractVersion::V0_4,
                    event_id: EventId::try_new(format!("desktop-event-{}", self.event_sequence))
                        .map_err(|error| error.to_string())?,
                    producer_boot_id: snapshot.producer_boot_id.clone(),
                    occurred_at_monotonic_ms: snapshot.produced_at_monotonic_ms,
                    caused_by_perception_revision: snapshot.revision,
                    kind: *kind,
                })
            })
            .collect()
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match parse_commands(&arguments) {
        Ok(Some(commands)) => {
            if let Err(message) = run_commands(&commands) {
                eprintln!("[AILIVE_GUN_SPIRIT_SIM_INVALID] {message}");
                std::process::exit(2);
            }
        }
        Ok(None) => {}
        Err(message) => {
            eprintln!("[AILIVE_GUN_SPIRIT_SIM_INVALID] {message}");
            std::process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: ailive-gun-spirit-sim [--scenario default|repeat|stale] [--] [command ...]\n\
         \n\
         Scenarios:\n\
         \x20 default  Standard 9-frame hardware-free lifecycle (default).\n\
         \x20 repeat   Duplicate observation and same-revision resend coverage.\n\
         \x20 stale    Fact TTL expiry, recovery and node-offline coverage.\n\
         \n\
         Commands: baseline held ready control-on control-off lower standby unknown reboot duplicate tick stale\n\
         With no positional commands, the selected scenario preset runs. Output is JSON Lines."
    );
}

/// Parses the optional scenario selector and the optional explicit command list.
///
/// Explicit positional commands replace the scenario preset; `--scenario` only
/// chooses which preset runs when no command is given. `Ok(None)` means help was
/// printed and the process should exit successfully.
fn parse_commands(arguments: &[String]) -> Result<Option<Vec<String>>, String> {
    let mut scenario = Scenario::Default;
    let mut scenario_given = false;
    let mut commands = Vec::new();
    let mut positional_only = false;
    let mut index = 0_usize;
    while index < arguments.len() {
        let argument = arguments[index].as_str();
        if !positional_only {
            match argument {
                "-h" | "--help" => {
                    print_usage();
                    return Ok(None);
                }
                "--" => {
                    positional_only = true;
                    index += 1;
                    continue;
                }
                "--scenario" => {
                    let value = arguments
                        .get(index + 1)
                        .ok_or_else(|| "--scenario requires a value".to_owned())?;
                    if scenario_given {
                        return Err("--scenario may only be given once".to_owned());
                    }
                    scenario = scenario_from_name(value)?;
                    scenario_given = true;
                    index += 2;
                    continue;
                }
                _ => {}
            }
            if let Some(value) = argument.strip_prefix("--scenario=") {
                if scenario_given {
                    return Err("--scenario may only be given once".to_owned());
                }
                scenario = scenario_from_name(value)?;
                scenario_given = true;
                index += 1;
                continue;
            }
            if argument.starts_with("--") {
                return Err(format!("unknown option {argument:?}"));
            }
        }
        commands.push(argument.to_owned());
        index += 1;
    }

    if commands.is_empty() {
        commands = scenario
            .commands()
            .iter()
            .map(|command| command.to_string())
            .collect();
    }
    Ok(Some(commands))
}

fn run_commands(commands: &[String]) -> Result<(), String> {
    let mut fixture = DesktopFixture::new()?;
    for command in commands {
        let frame = fixture.run_command(command)?;
        let line = serde_json::to_string(&frame)
            .map_err(|error| format!("cannot serialize simulator output: {error}"))?;
        println!("{line}");
    }
    Ok(())
}

fn new_reducer(
    host_boot_number: u64,
    now_ms: u64,
    front_node_uid: &NodeUid,
    rear_node_uid: &NodeUid,
) -> Result<PerceptionReducer, String> {
    let freshness = FreshnessProfile::try_new(
        FACT_TTL_MS,
        FACT_TTL_MS,
        FACT_TTL_MS,
        FACT_TTL_MS,
        NODE_OFFLINE_TTL_MS,
        STANDBY_DWELL_MS,
    )
    .map_err(|error| error.to_string())?;
    let orientation = OrientationCalibration::try_new([1_000, 0, 0], [0, 0, 1_000], permille(500)?)
        .map_err(|error| error.to_string())?;
    let confidence = ConfidenceProfile::try_new(permille(900)?, permille(700)?, permille(900)?)
        .map_err(|error| error.to_string())?;
    // 双板拓扑的能力划分，与泛化前`(role, observation)`白名单逐项等价。
    let front_capabilities = CapabilitySet::try_new([
        Capability::FrontGrip,
        Capability::Motion,
        Capability::Orientation,
        Capability::ShockDiagnostic,
    ])
    .map_err(|error| error.to_string())?;
    let rear_capabilities =
        CapabilitySet::try_new([Capability::RearGrip, Capability::PrimaryControl])
            .map_err(|error| error.to_string())?;
    let profile = PerceptionProfile::try_new(
        [
            (front_node_uid.clone(), front_capabilities),
            (rear_node_uid.clone(), rear_capabilities),
        ],
        freshness,
        Some(orientation),
        confidence,
    )
    .map_err(|error| error.to_string())?;
    Ok(PerceptionReducer::new(
        BootId::try_new(format!("mock-host-boot-{host_boot_number}"))
            .map_err(|error| error.to_string())?,
        profile,
        now_ms,
    ))
}

fn orientation(pose: Pose) -> Result<SensorObservationKind, String> {
    let gravity = match pose {
        Pose::Raised => [1_000, 0, 0],
        Pose::Lowered => [0, 0, 1_000],
    };
    Ok(SensorObservationKind::OrientationEstimate {
        gravity_mg_x: gravity[0],
        gravity_mg_y: gravity[1],
        gravity_mg_z: gravity[2],
        quality_milli: permille(950)?,
    })
}

fn contact_strength(contact: Contact) -> Result<Permille, String> {
    permille(if contact == Contact::Engaged {
        900
    } else {
        100
    })
}

fn permille(value: u16) -> Result<Permille, String> {
    Permille::new(value).map_err(|error| error.to_string())
}

fn disposition_name(semantic_advance: bool) -> &'static str {
    if semantic_advance {
        "advanced"
    } else {
        "resend"
    }
}

fn desktop_fixture_role_cue(request: &SensorTurnRequest) -> Option<RoleCue> {
    let last_event = request.events.last()?;
    let (visual_state_id, text) = match last_event.kind {
        DeviceEventKind::CarrierHeldEntered => ("fixture.held", "桌面夹具：检测到设备被拿起"),
        DeviceEventKind::CarrierHeldExited => ("fixture.standby", "桌面夹具：检测到设备进入待机"),
        DeviceEventKind::CarrierReadyEntered => ("fixture.ready", "桌面夹具：检测到准备姿态"),
        DeviceEventKind::CarrierReadyExited => ("fixture.held", "桌面夹具：离开准备姿态"),
        DeviceEventKind::ControlPrimaryEngaged => {
            ("fixture.control_engaged", "桌面夹具：辅助触点按下")
        }
        DeviceEventKind::ControlPrimaryReleased => {
            ("fixture.control_released", "桌面夹具：辅助触点释放")
        }
        DeviceEventKind::MagazineSeated => ("fixture.magazine_seated", "桌面夹具：检测到弹匣就位"),
    };
    Some(RoleCue {
        source: RoleCueSource::DesktopFixture,
        context_revision: request.device_context.perception_revision,
        source_event_ids: request
            .events
            .iter()
            .map(|event| event.event_id.clone())
            .collect(),
        visual_state_id: visual_state_id.to_owned(),
        text: text.to_owned(),
    })
}

fn offline_system_cue(snapshot: &PerceptionState) -> Option<SystemCue> {
    snapshot
        .node_health
        .iter()
        .any(|node| node.state == ailive_gun_spirit_contracts::NodeHealthState::Offline)
        .then(|| SystemCue {
            code: "node.offline".to_owned(),
            message: "传感节点离线；相关事实已降级为 Unknown".to_owned(),
        })
}
