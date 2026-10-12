#![allow(clippy::unwrap_used)]

use super::*;
use ailive_gun_spirit_contracts::{ObservationId, PrimaryControlId};

const FRONT: &str = "front-node";
const REAR: &str = "rear-node";

/// 双板拓扑，**能力划分与v0.2的白名单逐项等价**：
///
/// - front-node：前握 / 运动 / 姿态 / 冲击诊断
/// - rear-node：后握 / 主控
///
/// 旧实现由`(role, observation)`白名单隐含这个划分；现在它被显式声明出来。
/// 因此本文件的全部判据不受泛化影响——这正是硬约束2要锁的东西。
fn profile() -> PerceptionProfile {
    PerceptionProfile::try_new(
        [
            (
                NodeUid::try_new(FRONT).unwrap(),
                CapabilitySet::try_new([
                    Capability::FrontGrip,
                    Capability::Motion,
                    Capability::Orientation,
                    Capability::ShockDiagnostic,
                ])
                .unwrap(),
            ),
            (
                NodeUid::try_new(REAR).unwrap(),
                CapabilitySet::try_new([Capability::RearGrip, Capability::PrimaryControl]).unwrap(),
            ),
        ],
        FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 1_500, 100).unwrap(),
        Some(
            OrientationCalibration::try_new(
                [1_000, 0, 0],
                [0, 0, 1_000],
                Permille::new(500).unwrap(),
            )
            .unwrap(),
        ),
        ConfidenceProfile::try_new(
            Permille::new(900).unwrap(),
            Permille::new(700).unwrap(),
            Permille::new(900).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn observation(node: &str, sequence: u64, kind: SensorObservationKind) -> SensorObservation {
    SensorObservation {
        contract_version: ContractVersion::V0_3,
        observation_id: ObservationId::try_new(format!("{node}-{sequence}")).unwrap(),
        node_uid: NodeUid::try_new(node).unwrap(),
        boot_id: BootId::try_new(format!("{node}-boot-1")).unwrap(),
        sequence,
        node_monotonic_ms: sequence * 10,
        observation: kind,
    }
}

/// 判别字段由节点名派生，**保持v0.3之前的含义**（前节点报前握、后节点报后握）。
///
/// v0.3起判别来自载荷自身；这里在helper里做一次等价映射，
/// 好让下面的判据一行都不用改——它们检查的是融合结果，不是载荷形状。
fn grip_binding(node: &str) -> GripBinding {
    match node {
        FRONT => GripBinding::Front,
        REAR => GripBinding::Rear,
        other => panic!("test helper only knows the front and rear nodes, got {other}"),
    }
}

fn grip(node: &str, sequence: u64, contact: Contact) -> SensorObservation {
    observation(
        node,
        sequence,
        SensorObservationKind::GripContact {
            grip: grip_binding(node),
            contact,
            strength_milli: Permille::new(if contact == Contact::Engaged {
                900
            } else {
                100
            })
            .unwrap(),
        },
    )
}

fn motion(sequence: u64, value: MotionClass) -> SensorObservation {
    observation(
        FRONT,
        sequence,
        SensorObservationKind::MotionClass { motion: value },
    )
}

fn pose(sequence: u64, value: Pose) -> SensorObservation {
    let gravity = match value {
        Pose::Raised => [1_000, 0, 0],
        Pose::Lowered => [0, 0, 1_000],
    };
    observation(
        FRONT,
        sequence,
        SensorObservationKind::OrientationEstimate {
            gravity_mg_x: gravity[0],
            gravity_mg_y: gravity[1],
            gravity_mg_z: gravity[2],
            quality_milli: Permille::new(950).unwrap(),
        },
    )
}

fn control(sequence: u64, contact: Contact) -> SensorObservation {
    observation(
        REAR,
        sequence,
        SensorObservationKind::AuxControlContact {
            control_id: PrimaryControlId::PrimaryTrigger,
            contact,
        },
    )
}

fn known<T>(fact: &Fact<T>) -> Option<&T> {
    match fact {
        Fact::Known { value, .. } => Some(value),
        Fact::Unknown { .. } => None,
    }
}

fn reducer() -> PerceptionReducer {
    PerceptionReducer::new(BootId::try_new("host-boot").unwrap(), profile(), 0)
}

#[test]
fn rear_grip_enters_held_without_motion() {
    let mut reducer = reducer();
    let output = reducer.ingest(grip(REAR, 1, Contact::Engaged), 10).unwrap();
    assert_eq!(
        known(&output.snapshot.carrier_state),
        Some(&CarrierState::Held)
    );
}

#[test]
fn front_grip_requires_motion_to_enter_held() {
    let mut reducer = reducer();
    let first = reducer
        .ingest(grip(FRONT, 1, Contact::Engaged), 10)
        .unwrap();
    assert!(known(&first.snapshot.carrier_state).is_none());
    let second = reducer.ingest(motion(2, MotionClass::Moving), 20).unwrap();
    assert_eq!(
        known(&second.snapshot.carrier_state),
        Some(&CarrierState::Held)
    );
}

#[test]
fn ready_requires_rear_grip_and_raised_pose() {
    let mut reducer = reducer();
    reducer.ingest(grip(REAR, 1, Contact::Engaged), 10).unwrap();
    let ready = reducer.ingest(pose(1, Pose::Raised), 20).unwrap();
    assert_eq!(
        known(&ready.snapshot.carrier_state),
        Some(&CarrierState::Ready)
    );
    let lowered = reducer.ingest(pose(2, Pose::Lowered), 30).unwrap();
    assert_eq!(
        known(&lowered.snapshot.carrier_state),
        Some(&CarrierState::Held)
    );
}

#[test]
fn standby_waits_for_explicit_release_and_dwell() {
    let mut reducer = reducer();
    reducer.ingest(grip(REAR, 1, Contact::Engaged), 10).unwrap();
    reducer
        .ingest(grip(FRONT, 1, Contact::Released), 20)
        .unwrap();
    reducer.ingest(motion(2, MotionClass::Moving), 30).unwrap();
    reducer
        .ingest(grip(REAR, 2, Contact::Released), 40)
        .unwrap();
    reducer.ingest(motion(3, MotionClass::Idle), 50).unwrap();
    assert_eq!(
        known(&reducer.current.carrier_state),
        Some(&CarrierState::Held)
    );
    let standby = reducer.tick(150).unwrap();
    assert_eq!(
        known(&standby.snapshot.carrier_state),
        Some(&CarrierState::Standby)
    );
}

#[test]
fn ttl_expiry_becomes_unknown_without_guessing_release() {
    let mut reducer = reducer();
    reducer.ingest(grip(REAR, 1, Contact::Engaged), 10).unwrap();
    let expired = reducer.tick(1_011).unwrap();
    assert!(matches!(
        expired.snapshot.rear_grip,
        Fact::Unknown {
            reason: UnknownReason::Stale,
            ..
        }
    ));
    assert!(known(&expired.snapshot.carrier_state).is_none());
}

#[test]
fn same_semantic_observation_refreshes_without_advancing_revision() {
    let mut reducer = reducer();
    let first = reducer.ingest(grip(REAR, 1, Contact::Engaged), 10).unwrap();
    let second = reducer
        .ingest(grip(REAR, 2, Contact::Engaged), 100)
        .unwrap();
    assert_eq!(second.disposition, ReductionDisposition::SnapshotRefreshed);
    assert_eq!(first.snapshot.revision, second.snapshot.revision);
    assert!(matches!(
        second.snapshot.rear_grip,
        Fact::Known {
            observed_at_monotonic_ms: 100,
            ..
        }
    ));
}

#[test]
fn duplicate_and_sequence_errors_are_deterministic() {
    let mut reducer = reducer();
    let first = grip(REAR, 1, Contact::Engaged);
    reducer.ingest(first.clone(), 10).unwrap();
    let duplicate = reducer.ingest(first, 11).unwrap();
    assert_eq!(
        duplicate.disposition,
        ReductionDisposition::DuplicateIgnored
    );
    assert!(matches!(
        reducer.ingest(grip(REAR, 0, Contact::Released), 12),
        Err(ReducerError::SequenceRegressed { .. })
    ));
    assert!(matches!(
        reducer.ingest(grip(REAR, 1, Contact::Released), 12),
        Err(ReducerError::SequenceCollision { .. })
    ));
}

#[test]
fn node_boot_change_invalidates_old_capabilities() {
    let mut reducer = reducer();
    reducer.ingest(grip(REAR, 7, Contact::Engaged), 10).unwrap();
    reducer.ingest(control(8, Contact::Engaged), 20).unwrap();
    let mut rebooted = grip(REAR, 0, Contact::Released);
    rebooted.boot_id = BootId::try_new("rear-node-boot-2").unwrap();
    let output = reducer.ingest(rebooted, 30).unwrap();
    assert!(matches!(
        output.snapshot.primary_control,
        Fact::Unknown {
            reason: UnknownReason::NeverObserved,
            ..
        }
    ));
}

#[test]
fn missing_calibration_keeps_pose_uncalibrated() {
    let mut profile = profile();
    profile.orientation = None;
    let mut reducer = PerceptionReducer::new(BootId::try_new("host-boot").unwrap(), profile, 0);
    let output = reducer.ingest(pose(1, Pose::Raised), 10).unwrap();
    assert!(matches!(
        output.snapshot.pose,
        Fact::Unknown {
            reason: UnknownReason::Uncalibrated,
            ..
        }
    ));
}

// ---------------------------------------------------------------------------
// 节点模型泛化（GS-ARCH-001 / ADR-065）
//
// 上面那些测试锁的是**双板形态零语义变更**；下面这些锁的是**泛化本身成立**。
// ---------------------------------------------------------------------------

const SOLO: &str = "solo-node";

/// 单板形态：**一个节点声明全部六项能力**。
///
/// 这正是 P0-A 要的拓扑，也是泛化前**结构上无法构造**的（旧 `try_new`
/// 强制两个不同的 `NodeUid`，单节点直接撞 `DuplicateNodeBinding`）。
fn solo_profile() -> PerceptionProfile {
    let mut profile = profile();
    profile.nodes = vec![(
        NodeUid::try_new(SOLO).unwrap(),
        CapabilitySet::try_new([
            Capability::FrontGrip,
            Capability::RearGrip,
            Capability::Motion,
            Capability::Orientation,
            Capability::PrimaryControl,
            Capability::ShockDiagnostic,
        ])
        .unwrap(),
    )];
    profile
}

fn solo_observation(sequence: u64, kind: SensorObservationKind) -> SensorObservation {
    observation(SOLO, sequence, kind)
}

fn solo_reducer() -> PerceptionReducer {
    PerceptionReducer::new(BootId::try_new("host-boot").unwrap(), solo_profile(), 0)
}

#[test]
fn one_node_can_drive_the_full_carrier_lifecycle() {
    let mut reducer = solo_reducer();

    // 同一节点先报前握 + 运动 → Held
    let held = reducer
        .ingest(
            solo_observation(
                1,
                SensorObservationKind::GripContact {
                    grip: GripBinding::Front,
                    contact: Contact::Engaged,
                    strength_milli: Permille::new(900).unwrap(),
                },
            ),
            10,
        )
        .unwrap();
    let moving = reducer
        .ingest(
            solo_observation(
                2,
                SensorObservationKind::MotionClass {
                    motion: MotionClass::Moving,
                },
            ),
            20,
        )
        .unwrap();
    assert_eq!(known(&held.snapshot.front_grip), Some(&Contact::Engaged));
    assert_eq!(
        known(&moving.snapshot.carrier_state),
        Some(&CarrierState::Held)
    );

    // 同一节点再报后握 + 抬枪 → Ready。泛化前这两条会被白名单拒成
    // CapabilityNotBound（该节点已被当成"前节点"，不许报后握/姿态）。
    let rear = reducer
        .ingest(
            solo_observation(
                3,
                SensorObservationKind::GripContact {
                    grip: GripBinding::Rear,
                    contact: Contact::Engaged,
                    strength_milli: Permille::new(900).unwrap(),
                },
            ),
            30,
        )
        .unwrap();
    let raised = reducer
        .ingest(
            solo_observation(
                4,
                SensorObservationKind::OrientationEstimate {
                    gravity_mg_x: 1_000,
                    gravity_mg_y: 0,
                    gravity_mg_z: 0,
                    quality_milli: Permille::new(950).unwrap(),
                },
            ),
            40,
        )
        .unwrap();
    assert_eq!(known(&rear.snapshot.rear_grip), Some(&Contact::Engaged));
    assert_eq!(
        known(&raised.snapshot.carrier_state),
        Some(&CarrierState::Ready)
    );
    assert!(
        known(&raised.snapshot.primary_control).is_none(),
        "没报过主控，不得凭空出现"
    );
}

/// 同一节点报前后双握时，两个**语义槽位**必须分别落位，不能互相覆盖。
#[test]
fn one_node_keeps_front_and_rear_grip_separate() {
    let mut reducer = solo_reducer();
    let front = reducer
        .ingest(
            solo_observation(
                1,
                SensorObservationKind::GripContact {
                    grip: GripBinding::Front,
                    contact: Contact::Engaged,
                    strength_milli: Permille::new(900).unwrap(),
                },
            ),
            10,
        )
        .unwrap();
    let rear = reducer
        .ingest(
            solo_observation(
                2,
                SensorObservationKind::GripContact {
                    grip: GripBinding::Rear,
                    contact: Contact::Released,
                    strength_milli: Permille::new(100).unwrap(),
                },
            ),
            20,
        )
        .unwrap();

    assert_eq!(known(&front.snapshot.front_grip), Some(&Contact::Engaged));
    assert_eq!(known(&front.snapshot.rear_grip), None);
    assert_eq!(known(&rear.snapshot.front_grip), Some(&Contact::Engaged));
    assert_eq!(known(&rear.snapshot.rear_grip), Some(&Contact::Released));
}

/// 同一 `Capability` 被两个节点声明 → 构造期就拒绝。
/// 这是泛化后**唯一**的拓扑约束，它蕴含了旧的"前后必须不同 UID"。
#[test]
fn capability_claimed_twice_is_rejected() {
    let error = PerceptionProfile::try_new(
        [
            (
                NodeUid::try_new(FRONT).unwrap(),
                CapabilitySet::try_new([Capability::FrontGrip, Capability::Motion]).unwrap(),
            ),
            (
                NodeUid::try_new(REAR).unwrap(),
                CapabilitySet::try_new([Capability::FrontGrip]).unwrap(),
            ),
        ],
        FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 1_500, 100).unwrap(),
        Some(
            OrientationCalibration::try_new(
                [1_000, 0, 0],
                [0, 0, 1_000],
                Permille::new(500).unwrap(),
            )
            .unwrap(),
        ),
        ConfidenceProfile::try_new(
            Permille::new(900).unwrap(),
            Permille::new(700).unwrap(),
            Permille::new(900).unwrap(),
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProfileError::CapabilityOwnedTwice {
            capability: Capability::FrontGrip,
            ..
        }
    ));
}

#[test]
fn empty_capability_set_is_rejected() {
    assert!(matches!(
        CapabilitySet::try_new([]),
        Err(ProfileError::NodeWithoutCapability)
    ));
}

/// 节点**没声明**某能力却上报 → `CapabilityNotBound`（此前零覆盖）。
#[test]
fn undeclared_capability_is_rejected() {
    let mut reducer = reducer();
    // 后节点只声明了 RearGrip 与 PrimaryControl，没声明 Motion。
    let error = reducer
        .ingest(
            observation(
                REAR,
                1,
                SensorObservationKind::MotionClass {
                    motion: MotionClass::Moving,
                },
            ),
            10,
        )
        .unwrap_err();
    assert!(matches!(
        error,
        ReducerError::CapabilityNotBound {
            capability: Capability::Motion,
            ..
        }
    ));
}

/// 未知 `node_uid` → `UnknownNode`（此前零覆盖）。
#[test]
fn unknown_node_is_rejected() {
    let mut reducer = reducer();
    let error = reducer
        .ingest(
            observation(
                "unregistered-node",
                1,
                SensorObservationKind::MotionClass {
                    motion: MotionClass::Moving,
                },
            ),
            10,
        )
        .unwrap_err();
    assert!(matches!(error, ReducerError::UnknownNode(_)));
}

/// `node_health` 顺序由**字典序**排定，与声明顺序无关。
///
/// 这一条不能靠"两处既有 UID 恰好 front 在前"的巧合——那是巧合，不是保证。
/// 这里故意**倒序声明**，断言构造后与输出都归到 front 在前。
#[test]
fn node_health_follows_sorted_order_not_declaration_order() {
    // 先声明 rear、再声明 front，与 `profile()` 的声明顺序相反。
    let profile = PerceptionProfile::try_new(
        [
            (
                NodeUid::try_new(REAR).unwrap(),
                CapabilitySet::try_new([Capability::RearGrip, Capability::PrimaryControl]).unwrap(),
            ),
            (
                NodeUid::try_new(FRONT).unwrap(),
                CapabilitySet::try_new([
                    Capability::FrontGrip,
                    Capability::Motion,
                    Capability::Orientation,
                    Capability::ShockDiagnostic,
                ])
                .unwrap(),
            ),
        ],
        FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 1_500, 100).unwrap(),
        Some(
            OrientationCalibration::try_new(
                [1_000, 0, 0],
                [0, 0, 1_000],
                Permille::new(500).unwrap(),
            )
            .unwrap(),
        ),
        ConfidenceProfile::try_new(
            Permille::new(900).unwrap(),
            Permille::new(700).unwrap(),
            Permille::new(900).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        profile
            .nodes()
            .map(|(uid, _)| uid.as_str())
            .collect::<Vec<_>>(),
        vec![FRONT, REAR],
        "构造后必须按字典序排定，而不是保留声明顺序"
    );

    let reducer = PerceptionReducer::new(BootId::try_new("host-boot").unwrap(), profile, 0);
    assert_eq!(
        reducer
            .current_state()
            .node_health
            .iter()
            .map(|health| health.node_uid.as_str())
            .collect::<Vec<_>>(),
        vec![FRONT, REAR]
    );
}

/// 无人声明某能力时，对应事实必须是 `NeverObserved` ——
/// 感知内核的失败方向是"少知道"，不是"编造"。
#[test]
fn capability_without_owner_stays_never_observed() {
    // 没有任何节点声明 Orientation → pose 无 owner
    let profile = PerceptionProfile::try_new(
        [
            (
                NodeUid::try_new(FRONT).unwrap(),
                CapabilitySet::try_new([Capability::FrontGrip, Capability::Motion]).unwrap(),
            ),
            (
                NodeUid::try_new(REAR).unwrap(),
                CapabilitySet::try_new([Capability::RearGrip, Capability::PrimaryControl]).unwrap(),
            ),
        ],
        FreshnessProfile::try_new(1_000, 1_000, 1_000, 1_000, 1_500, 100).unwrap(),
        Some(
            OrientationCalibration::try_new(
                [1_000, 0, 0],
                [0, 0, 1_000],
                Permille::new(500).unwrap(),
            )
            .unwrap(),
        ),
        ConfidenceProfile::try_new(
            Permille::new(900).unwrap(),
            Permille::new(700).unwrap(),
            Permille::new(900).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut reducer = PerceptionReducer::new(BootId::try_new("host-boot").unwrap(), profile, 0);
    let output = reducer
        .ingest(
            observation(
                FRONT,
                1,
                SensorObservationKind::MotionClass {
                    motion: MotionClass::Moving,
                },
            ),
            10,
        )
        .unwrap();
    assert!(matches!(
        output.snapshot.pose,
        Fact::Unknown {
            reason: UnknownReason::NeverObserved,
            ..
        }
    ));
}
