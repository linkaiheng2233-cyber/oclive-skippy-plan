use ailive_gun_spirit_contracts::{NodeUid, Permille};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreshnessProfile {
    pub(super) grip_ttl_ms: u64,
    pub(super) motion_ttl_ms: u64,
    pub(super) orientation_ttl_ms: u64,
    pub(super) control_ttl_ms: u64,
    pub(super) node_offline_ttl_ms: u64,
    pub(super) standby_dwell_ms: u64,
}

impl FreshnessProfile {
    pub fn try_new(
        grip_ttl_ms: u64,
        motion_ttl_ms: u64,
        orientation_ttl_ms: u64,
        control_ttl_ms: u64,
        node_offline_ttl_ms: u64,
        standby_dwell_ms: u64,
    ) -> Result<Self, ProfileError> {
        if [
            grip_ttl_ms,
            motion_ttl_ms,
            orientation_ttl_ms,
            control_ttl_ms,
            node_offline_ttl_ms,
        ]
        .contains(&0)
        {
            return Err(ProfileError::ZeroDuration);
        }
        let longest_fact_ttl = grip_ttl_ms
            .max(motion_ttl_ms)
            .max(orientation_ttl_ms)
            .max(control_ttl_ms);
        if node_offline_ttl_ms < longest_fact_ttl {
            return Err(ProfileError::NodeOfflineBeforeFactExpiry {
                node_offline_ttl_ms,
                longest_fact_ttl_ms: longest_fact_ttl,
            });
        }
        Ok(Self {
            grip_ttl_ms,
            motion_ttl_ms,
            orientation_ttl_ms,
            control_ttl_ms,
            node_offline_ttl_ms,
            standby_dwell_ms,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrientationCalibration {
    pub(super) raised_gravity_mg: [i32; 3],
    pub(super) lowered_gravity_mg: [i32; 3],
    pub(super) minimum_quality_milli: Permille,
}

impl OrientationCalibration {
    pub fn try_new(
        raised_gravity_mg: [i32; 3],
        lowered_gravity_mg: [i32; 3],
        minimum_quality_milli: Permille,
    ) -> Result<Self, ProfileError> {
        if raised_gravity_mg == lowered_gravity_mg {
            return Err(ProfileError::IdenticalOrientationAnchors);
        }
        Ok(Self {
            raised_gravity_mg,
            lowered_gravity_mg,
            minimum_quality_milli,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfidenceProfile {
    pub(super) discrete_observation: Permille,
    pub(super) ready_single_hand: Permille,
    pub(super) ready_two_hand: Permille,
}

impl ConfidenceProfile {
    pub fn try_new(
        discrete_observation: Permille,
        ready_single_hand: Permille,
        ready_two_hand: Permille,
    ) -> Result<Self, ProfileError> {
        if ready_two_hand < ready_single_hand {
            return Err(ProfileError::TwoHandReadyBelowSingleHand);
        }
        Ok(Self {
            discrete_observation,
            ready_single_hand,
            ready_two_hand,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerceptionProfile {
    pub(super) front_node_uid: NodeUid,
    pub(super) rear_node_uid: NodeUid,
    pub(super) freshness: FreshnessProfile,
    pub(super) orientation: Option<OrientationCalibration>,
    pub(super) confidence: ConfidenceProfile,
}

impl PerceptionProfile {
    pub fn try_new(
        front_node_uid: NodeUid,
        rear_node_uid: NodeUid,
        freshness: FreshnessProfile,
        orientation: Option<OrientationCalibration>,
        confidence: ConfidenceProfile,
    ) -> Result<Self, ProfileError> {
        if front_node_uid == rear_node_uid {
            return Err(ProfileError::DuplicateNodeBinding);
        }
        Ok(Self {
            front_node_uid,
            rear_node_uid,
            freshness,
            orientation,
            confidence,
        })
    }

    pub fn front_node_uid(&self) -> &NodeUid {
        &self.front_node_uid
    }

    pub fn rear_node_uid(&self) -> &NodeUid {
        &self.rear_node_uid
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    DuplicateNodeBinding,
    ZeroDuration,
    NodeOfflineBeforeFactExpiry {
        node_offline_ttl_ms: u64,
        longest_fact_ttl_ms: u64,
    },
    IdenticalOrientationAnchors,
    TwoHandReadyBelowSingleHand,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNodeBinding => write!(formatter, "front and rear nodes must differ"),
            Self::ZeroDuration => write!(formatter, "fact and node TTL values must be non-zero"),
            Self::NodeOfflineBeforeFactExpiry {
                node_offline_ttl_ms,
                longest_fact_ttl_ms,
            } => write!(
                formatter,
                "node offline TTL {node_offline_ttl_ms}ms is shorter than fact TTL {longest_fact_ttl_ms}ms"
            ),
            Self::IdenticalOrientationAnchors => {
                write!(formatter, "raised and lowered calibration anchors must differ")
            }
            Self::TwoHandReadyBelowSingleHand => write!(
                formatter,
                "two-hand Ready confidence must not be below single-hand confidence"
            ),
        }
    }
}

impl std::error::Error for ProfileError {}
