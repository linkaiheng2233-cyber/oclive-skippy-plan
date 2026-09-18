#![allow(clippy::unwrap_used)]

use super::*;
use ailive_gun_spirit_contracts::{ObservationId, PrimaryControlId};

const FRONT: &str = "front-node";
const REAR: &str = "rear-node";

fn profile() -> PerceptionProfile {
    PerceptionProfile::try_new(
        NodeUid::try_new(FRONT).unwrap(),
        NodeUid::try_new(REAR).unwrap(),
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
        contract_version: ContractVersion::V0_2,
        observation_id: ObservationId::try_new(format!("{node}-{sequence}")).unwrap(),
        node_uid: NodeUid::try_new(node).unwrap(),
        boot_id: BootId::try_new(format!("{node}-boot-1")).unwrap(),
        sequence,
        node_monotonic_ms: sequence * 10,
        observation: kind,
    }
}

fn grip(node: &str, sequence: u64, contact: Contact) -> SensorObservation {
    observation(
        node,
        sequence,
        SensorObservationKind::GripContact {
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
