//! V2：无线链路的恶劣条件语义验证（无硬件，纯软件仿真）。
//!
//! 目的：在买任何无线模块之前，先证明"无线传输会带来的四种坏情况"内核都能按设计处置：
//! 重复投递、序号倒退、同序号不同内容（碰撞）、节点重启（新 `boot_id`）。
//!
//! 同时验证一条**链路约束**：内核按"严格递增序号"接收，因此**链路必须保序**。
//! BLE 的连接层本身保序，所以无线方案成立；若将来换成无序传输，必须先在链路层排序。

use ailive_gun_spirit_contracts::{
    BootId, Capability, Contact, ContractVersion, GripBinding, MotionClass, NodeUid, ObservationId,
    Permille, PrimaryControlId, SensorObservation, SensorObservationKind,
};
use ailive_gun_spirit_perception_core::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    PerceptionReducer, ReductionDisposition, ReducerError,
};

fn permille(value: u16) -> Permille {
    Permille::new(value).expect("permille in range")
}

struct Node {
    uid: NodeUid,
    boot: BootId,
    sequence: u64,
}

impl Node {
    fn new(name: &str, boot: u32) -> Self {
        Self {
            uid: NodeUid::try_new(name).expect("node uid"),
            boot: BootId::try_new(format!("{name}-boot-{boot}")).expect("boot id"),
            sequence: 0,
        }
    }

    fn reboot(&mut self) {
        self.sequence = 0;
        self.boot = BootId::try_new(format!("{}-boot-2", self.uid.as_str())).expect("boot id");
    }

    fn observe(&mut self, millis: u64, kind: SensorObservationKind) -> SensorObservation {
        self.sequence += 1;
        SensorObservation {
            contract_version: ContractVersion::V0_3,
            observation_id: ObservationId::try_new(format!(
                "{}:{}:{}",
                self.uid.as_str(),
                self.boot.as_str(),
                self.sequence
            ))
            .expect("observation id"),
            node_uid: self.uid.clone(),
            boot_id: self.boot.clone(),
            sequence: self.sequence,
            node_monotonic_ms: millis,
            observation: kind,
        }
    }
}

/// 五模块形态：每个节点只绑定自己的那一项能力（同一能力只能一个 owner）。
fn five_node_reducer() -> PerceptionReducer {
    let nodes = [
        ("node-front", vec![Capability::FrontGrip, Capability::Motion]),
        ("node-rear", vec![Capability::RearGrip]),
        ("node-trigger", vec![Capability::PrimaryControl]),
        ("node-imu", vec![Capability::Orientation]),
        ("node-shock", vec![Capability::ShockDiagnostic]),
    ];
    let mut entries = Vec::new();
    for (name, capabilities) in nodes {
        entries.push((
            NodeUid::try_new(name).expect("node uid"),
            CapabilitySet::try_new(capabilities).expect("capability set"),
        ));
    }
    let freshness =
        FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 3_500, 200).expect("freshness");
    let orientation = OrientationCalibration::try_new([1_000, 0, 0], [0, 0, 1_000], permille(500))
        .expect("orientation");
    let confidence =
        ConfidenceProfile::try_new(permille(900), permille(700), permille(900)).expect("confidence");
    let profile = PerceptionProfile::try_new(entries, freshness, Some(orientation), confidence)
        .expect("profile");
    PerceptionReducer::new(
        BootId::try_new("host-boot-1").expect("host boot"),
        profile,
        0,
    )
}

fn grip(grip: GripBinding, contact: Contact, strength: u16) -> SensorObservationKind {
    SensorObservationKind::GripContact {
        grip,
        contact,
        strength_milli: permille(strength),
    }
}

#[test]
fn duplicates_never_advance_semantics_twice() {
    let mut reducer = five_node_reducer();
    let mut front = Node::new("node-front", 1);
    let mut rear = Node::new("node-rear", 1);

    let first = front.observe(0, grip(GripBinding::Front, Contact::Engaged, 700));
    assert!(reducer.ingest(first, 0).is_ok());

    let rear_engaged = rear.observe(10, grip(GripBinding::Rear, Contact::Engaged, 650));
    let advanced = reducer.ingest(rear_engaged.clone(), 10).expect("ingest rear");
    assert_eq!(
        advanced.disposition,
        ReductionDisposition::SemanticStateAdvanced,
        "前握+后握同时接触应产生语义推进（held_entered）"
    );

    // 链路重传：同一条观测再投递一次。
    let duplicate = reducer.ingest(rear_engaged, 20).expect("ingest duplicate");
    assert_eq!(
        duplicate.disposition,
        ReductionDisposition::DuplicateIgnored,
        "重复投递必须被忽略，不得再次推进语义"
    );
}

#[test]
fn sequence_regression_is_rejected() {
    let mut reducer = five_node_reducer();
    let mut front = Node::new("node-front", 1);
    let first = front.observe(0, grip(GripBinding::Front, Contact::Engaged, 700));
    let second = front.observe(10, grip(GripBinding::Front, Contact::Released, 0));
    assert!(reducer.ingest(first.clone(), 0).is_ok());
    assert!(reducer.ingest(second, 10).is_ok());

    // 链路乱序：先到的旧包被迟到投递。
    let error = reducer.ingest(first, 20).expect_err("倒退序号必须被拒绝");
    assert!(
        matches!(error, ReducerError::SequenceRegressed { .. }),
        "期望 SequenceRegressed，实际 {error:?}"
    );
}

#[test]
fn same_sequence_with_different_payload_is_collision() {
    let mut reducer = five_node_reducer();
    let mut front = Node::new("node-front", 1);
    let original = front.observe(0, grip(GripBinding::Front, Contact::Engaged, 700));
    assert!(reducer.ingest(original, 0).is_ok());

    // 同一序号、不同内容（例如设备端 bug 或链路损坏）不得被当成"重传"。
    let mut forged = front.observe(5, grip(GripBinding::Front, Contact::Released, 0));
    forged.sequence = 1;
    let error = reducer.ingest(forged, 5).expect_err("同号异内容必须被拒绝");
    assert!(
        matches!(error, ReducerError::SequenceCollision { .. }),
        "期望 SequenceCollision，实际 {error:?}"
    );
}

#[test]
fn reboot_starts_a_fresh_session() {
    let mut reducer = five_node_reducer();
    let mut rear = Node::new("node-rear", 1);
    for step in 0..3u64 {
        let observation = rear.observe(step * 10, grip(GripBinding::Rear, Contact::Engaged, 600));
        assert!(reducer.ingest(observation, step * 10).is_ok());
    }
    rear.reboot();
    // 重启后序号从 1 重新开始，必须被接受（新 boot_id = 新会话）。
    let after_reboot = rear.observe(100, grip(GripBinding::Rear, Contact::Engaged, 600));
    let output = reducer
        .ingest(after_reboot, 100)
        .expect("新 boot 会话应被接受");
    assert!(matches!(
        output.disposition,
        ReductionDisposition::SnapshotRefreshed | ReductionDisposition::SemanticStateAdvanced
    ));
}

#[test]
fn five_modules_all_report_and_stay_healthy() {
    let mut reducer = five_node_reducer();
    let mut front = Node::new("node-front", 1);
    let mut rear = Node::new("node-rear", 1);
    let mut trigger = Node::new("node-trigger", 1);
    let mut imu = Node::new("node-imu", 1);
    let mut shock = Node::new("node-shock", 1);

    let samples = [
        front.observe(0, grip(GripBinding::Front, Contact::Engaged, 700)),
        rear.observe(5, grip(GripBinding::Rear, Contact::Engaged, 650)),
        trigger.observe(
            10,
            SensorObservationKind::AuxControlContact {
                control_id: PrimaryControlId::PrimaryTrigger,
                contact: Contact::Engaged,
            },
        ),
        imu.observe(
            15,
            SensorObservationKind::OrientationEstimate {
                gravity_mg_x: 1_000,
                gravity_mg_y: 0,
                gravity_mg_z: 0,
                quality_milli: permille(900),
            },
        ),
        shock.observe(
            20,
            SensorObservationKind::ShockObserved {
                severity_milli: permille(250),
            },
        ),
        front.observe(25, SensorObservationKind::MotionClass { motion: MotionClass::Idle }),
    ];
    for (index, sample) in samples.into_iter().enumerate() {
        reducer
            .ingest(sample, (index as u64) * 10)
            .expect("五个模块的观测都应被接受");
    }
    let state = reducer.current_state();
    // 每个节点都有健康记录（`PerceptionState::node_health`）。
    assert_eq!(state.node_health.len(), 5, "应有五个节点健康记录");
}

#[test]
fn link_must_preserve_order() {
    // 这条不是"希望成立"，而是记录一条**链路约束**：
    // 内核按严格递增序号接收，乱序投递会被拒绝，所以链路层必须保序。
    let mut reducer = five_node_reducer();
    let mut rear = Node::new("node-rear", 1);
    let first = rear.observe(0, grip(GripBinding::Rear, Contact::Engaged, 600));
    let second = rear.observe(10, grip(GripBinding::Rear, Contact::Released, 0));
    assert!(reducer.ingest(first, 0).is_ok());
    let error = reducer.ingest(second.clone(), 10).expect("顺序投递应成功");
    assert!(matches!(
        error.disposition,
        ReductionDisposition::SnapshotRefreshed | ReductionDisposition::SemanticStateAdvanced
    ));
}

