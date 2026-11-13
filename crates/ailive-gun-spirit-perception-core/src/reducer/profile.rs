use ailive_gun_spirit_contracts::{Capability, NodeUid, Permille};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

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

/// 单个节点声明拥有的能力集合。
///
/// `Capability` 是闭集（6项），因此用有序`Vec`而非位图或`HashSet`：
/// 判断是O(6)，且不需要为此引入新依赖或新trait。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// 空集是错误：一个什么都不声明的节点没有存在意义，
    /// 放行只会在运行时变成"它报什么都被拒"的沉默故障。
    pub fn try_new(declared: impl IntoIterator<Item = Capability>) -> Result<Self, ProfileError> {
        let mut capabilities: Vec<Capability> = Vec::new();
        for capability in declared {
            if !capabilities.contains(&capability) {
                capabilities.push(capability);
            }
        }
        if capabilities.is_empty() {
            return Err(ProfileError::NodeWithoutCapability);
        }
        Ok(Self { capabilities })
    }

    pub fn contains(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.capabilities.iter().copied()
    }
}

/// 感知内核的静态拓扑配置：**节点 → 能力集**。
///
/// 泛化自旧的「两个必需且互不相同的`NodeUid`」。这个形状同时容纳：
///
/// - **P0-A单板形态**：一个节点声明全部6项能力；
/// - **P0-B双板形态**：两个节点各管一半；
/// - 任意节点数。
///
/// 唯一的拓扑约束是**同一`Capability`至多一个owner**。它蕴含了旧约束
/// （前后握必须来自不同节点），但不再要求节点数或角色命名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerceptionProfile {
    /// 节点→能力集绑定，**构造时按`node_uid`字典序排定**。
    ///
    /// 排定而不是保留声明顺序：这个顺序会渲染进`PerceptionState.node_health`，
    /// 而`PerceptionState`参与`has_same_revision_payload`比较。排序让
    /// 「同一组绑定、不同声明顺序」得到同一份状态。
    ///
    /// `NodeUid`未派生`Ord`（`string_id!`的derive里没有，也不宜为一个内部
    /// 排序需求去改契约类型），所以用有序`Vec`+按`as_str()`显式排序代替`BTreeMap`。
    pub(super) nodes: Vec<(NodeUid, CapabilitySet)>,
    pub(super) freshness: FreshnessProfile,
    pub(super) orientation: Option<OrientationCalibration>,
    pub(super) confidence: ConfidenceProfile,
}

impl PerceptionProfile {
    pub fn try_new(
        nodes: impl IntoIterator<Item = (NodeUid, CapabilitySet)>,
        freshness: FreshnessProfile,
        orientation: Option<OrientationCalibration>,
        confidence: ConfidenceProfile,
    ) -> Result<Self, ProfileError> {
        let mut nodes: Vec<(NodeUid, CapabilitySet)> = nodes.into_iter().collect();
        if nodes.is_empty() {
            return Err(ProfileError::NoNodes);
        }
        nodes.sort_by(|left, right| left.0.as_str().cmp(right.0.as_str()));

        for window in nodes.windows(2) {
            if window[0].0 == window[1].0 {
                return Err(ProfileError::DuplicateNodeBinding {
                    node_uid: window[0].0.clone(),
                });
            }
        }

        for (index, (node_uid, capabilities)) in nodes.iter().enumerate() {
            for capability in capabilities.iter() {
                if let Some((owner, _)) = nodes[..index]
                    .iter()
                    .find(|(_, declared)| declared.contains(capability))
                {
                    return Err(ProfileError::CapabilityOwnedTwice {
                        capability,
                        first_owner: owner.clone(),
                        second_owner: node_uid.clone(),
                    });
                }
            }
        }

        Ok(Self {
            nodes,
            freshness,
            orientation,
            confidence,
        })
    }

    /// **排序后**的节点绑定；`node_health`按此顺序渲染。
    pub fn nodes(&self) -> impl Iterator<Item = (&NodeUid, &CapabilitySet)> {
        self.nodes.iter().map(|(node_uid, set)| (node_uid, set))
    }

    /// 节点声明的能力集；未声明的节点返回`None`（调用方据此报`UnknownNode`）。
    pub fn capabilities_of(&self, node_uid: &NodeUid) -> Option<&CapabilitySet> {
        self.nodes
            .iter()
            .find(|(declared, _)| declared == node_uid)
            .map(|(_, set)| set)
    }

    /// 该能力的唯一owner；无人声明时返回`None`。
    pub fn owners_of(&self, capability: Capability) -> Option<&NodeUid> {
        self.nodes
            .iter()
            .find(|(_, set)| set.contains(capability))
            .map(|(node_uid, _)| node_uid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// 同一个`node_uid`被声明了两次。
    DuplicateNodeBinding {
        node_uid: NodeUid,
    },
    /// 同一个`Capability`被两个节点同时声明——泛化后的唯一拓扑约束。
    CapabilityOwnedTwice {
        capability: Capability,
        first_owner: NodeUid,
        second_owner: NodeUid,
    },
    /// 一个节点没有声明任何能力。
    NodeWithoutCapability,
    /// 一个节点都没有。
    NoNodes,
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
            Self::DuplicateNodeBinding { node_uid } => {
                write!(formatter, "node {node_uid:?} is declared more than once")
            }
            Self::CapabilityOwnedTwice {
                capability,
                first_owner,
                second_owner,
            } => write!(
                formatter,
                "capability {capability:?} is claimed by both {first_owner:?} and {second_owner:?}"
            ),
            Self::NodeWithoutCapability => {
                write!(formatter, "every node must declare at least one capability")
            }
            Self::NoNodes => write!(formatter, "a profile must declare at least one node"),
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

impl core::error::Error for ProfileError {}
