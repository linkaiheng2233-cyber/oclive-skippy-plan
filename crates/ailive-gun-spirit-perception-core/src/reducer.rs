use ailive_gun_spirit_contracts::{
    BootId, Capability, CarrierState, Contact, ContractValidate, ContractVersion, EvidenceRelation,
    Fact, GripBinding, MotionClass, NodeHealth, NodeHealthState, NodeUid, PerceptionState,
    Permille, Pose, SensorObservation, SensorObservationKind, SourceRef, UnknownReason,
    ValidationError,
};

mod profile;

pub use profile::{
    CapabilitySet, ConfidenceProfile, FreshnessProfile, OrientationCalibration, PerceptionProfile,
    ProfileError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionDisposition {
    DuplicateIgnored,
    SnapshotRefreshed,
    SemanticStateAdvanced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductionOutput {
    pub disposition: ReductionDisposition,
    pub snapshot: PerceptionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReducerError {
    InvalidObservation(Vec<ValidationError>),
    UnknownNode(NodeUid),
    CapabilityNotBound {
        node_uid: NodeUid,
        capability: Capability,
    },
    HostClockRegressed {
        previous_ms: u64,
        current_ms: u64,
    },
    SequenceRegressed {
        node_uid: NodeUid,
        previous: u64,
        current: u64,
    },
    SequenceCollision {
        node_uid: NodeUid,
        sequence: u64,
    },
    RevisionOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence<T> {
    value: T,
    observed_at_monotonic_ms: u64,
    fresh_until_monotonic_ms: u64,
    confidence_milli: Permille,
    source_refs: Vec<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceMeta {
    observed_at_monotonic_ms: u64,
    fresh_until_monotonic_ms: u64,
    confidence_milli: Permille,
    source_refs: Vec<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrientationEvidence {
    gravity_mg: [i32; 3],
    observed_at_monotonic_ms: u64,
    fresh_until_monotonic_ms: u64,
    quality_milli: Permille,
    source_refs: Vec<SourceRef>,
}

#[derive(Debug, Clone, Default)]
struct NodeSession {
    boot_id: Option<BootId>,
    last_sequence: Option<u64>,
    last_observation: Option<SensorObservation>,
    last_seen_host_monotonic_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Resolved<T> {
    Known(Evidence<T>),
    Unknown(UnknownReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedSnapshot {
    carrier_state: Resolved<CarrierState>,
    front_grip: Resolved<Contact>,
    rear_grip: Resolved<Contact>,
    motion: Resolved<MotionClass>,
    pose: Resolved<Pose>,
    primary_control: Resolved<Contact>,
    node_health: Vec<NodeHealth>,
}

/// Pure SensorObservation -> PerceptionState reducer.
///
/// All durations, calibration anchors and evidence strengths come from `PerceptionProfile`.
/// Adapters still own ADC/IMU sampling, filtering, BLE and Host time acquisition.
#[derive(Debug, Clone)]
pub struct PerceptionReducer {
    host_boot_id: BootId,
    profile: PerceptionProfile,
    /// 每个已声明节点一条session，键为`node_uid`，与`profile.nodes()`同序。
    sessions: Vec<(NodeUid, NodeSession)>,
    /// 以下五个是**语义槽位，不是节点槽位**：`front_grip`指「前握持」这件事，
    /// 不指「前节点」。P0-A单板形态下同一个节点会同时写前、后两个握持槽。
    /// 这也正是`resolve_carrier`无需改动的原因——它只认这些具名语义。
    front_grip: Option<Evidence<Contact>>,
    rear_grip: Option<Evidence<Contact>>,
    motion: Option<Evidence<MotionClass>>,
    orientation: Option<OrientationEvidence>,
    primary_control: Option<Evidence<Contact>>,
    standby_candidate_since_ms: Option<u64>,
    last_host_monotonic_ms: u64,
    current: PerceptionState,
}

impl PerceptionReducer {
    pub fn new(host_boot_id: BootId, profile: PerceptionProfile, now_ms: u64) -> Self {
        let sessions = profile
            .nodes()
            .map(|(node_uid, _)| (node_uid.clone(), NodeSession::default()))
            .collect();
        let node_health = profile
            .nodes()
            .map(|(node_uid, _)| NodeHealth {
                node_uid: node_uid.clone(),
                state: NodeHealthState::Offline,
            })
            .collect();
        let current = PerceptionState {
            contract_version: ContractVersion::V0_3,
            host_boot_id: host_boot_id.clone(),
            revision: 0,
            produced_at_monotonic_ms: now_ms,
            carrier_state: never_observed(),
            front_grip: never_observed(),
            rear_grip: never_observed(),
            motion: never_observed(),
            pose: never_observed(),
            primary_control: never_observed(),
            node_health,
        };
        Self {
            host_boot_id,
            profile,
            sessions,
            front_grip: None,
            rear_grip: None,
            motion: None,
            orientation: None,
            primary_control: None,
            standby_candidate_since_ms: None,
            last_host_monotonic_ms: now_ms,
            current,
        }
    }

    pub fn current_state(&self) -> &PerceptionState {
        &self.current
    }

    pub fn ingest(
        &mut self,
        observation: SensorObservation,
        received_at_host_monotonic_ms: u64,
    ) -> Result<ReductionOutput, ReducerError> {
        observation
            .validate()
            .map_err(ReducerError::InvalidObservation)?;
        self.ensure_clock(received_at_host_monotonic_ms)?;

        let node_uid = observation.node_uid.clone();
        // 节点必须**声明**过这个能力。能力本身是观测的函数（见`observation_capability`），
        // 节点只负责声明自己拥有哪些能力。
        let Some(declared) = self.profile.capabilities_of(&node_uid) else {
            return Err(ReducerError::UnknownNode(node_uid));
        };
        let capability = observation_capability(&observation.observation);
        if !declared.contains(capability) {
            return Err(ReducerError::CapabilityNotBound {
                node_uid,
                capability,
            });
        }

        let same_boot = self.session(&node_uid)?.boot_id.as_ref() == Some(&observation.boot_id);
        if same_boot {
            if let Some(previous_sequence) = self.session(&node_uid)?.last_sequence {
                if observation.sequence < previous_sequence {
                    return Err(ReducerError::SequenceRegressed {
                        node_uid,
                        previous: previous_sequence,
                        current: observation.sequence,
                    });
                }
                if observation.sequence == previous_sequence {
                    if self.session(&node_uid)?.last_observation.as_ref() != Some(&observation) {
                        return Err(ReducerError::SequenceCollision {
                            node_uid,
                            sequence: observation.sequence,
                        });
                    }
                    self.last_host_monotonic_ms = received_at_host_monotonic_ms;
                    return self.recompute(
                        received_at_host_monotonic_ms,
                        ReductionDisposition::DuplicateIgnored,
                    );
                }
            }
        } else {
            self.clear_node(&node_uid);
        }

        let source_ref = SourceRef {
            observation_id: observation.observation_id.clone(),
            node_uid: node_uid.clone(),
            capability,
            relation: EvidenceRelation::Supports,
        };
        self.store_observation(
            &observation.observation,
            received_at_host_monotonic_ms,
            source_ref,
        );

        let session = self.session_mut(&node_uid)?;
        session.boot_id = Some(observation.boot_id.clone());
        session.last_sequence = Some(observation.sequence);
        session.last_observation = Some(observation);
        session.last_seen_host_monotonic_ms = Some(received_at_host_monotonic_ms);
        self.last_host_monotonic_ms = received_at_host_monotonic_ms;
        self.recompute(
            received_at_host_monotonic_ms,
            ReductionDisposition::SnapshotRefreshed,
        )
    }

    pub fn tick(&mut self, now_ms: u64) -> Result<ReductionOutput, ReducerError> {
        self.ensure_clock(now_ms)?;
        self.last_host_monotonic_ms = now_ms;
        self.recompute(now_ms, ReductionDisposition::SnapshotRefreshed)
    }

    fn ensure_clock(&self, now_ms: u64) -> Result<(), ReducerError> {
        if now_ms < self.last_host_monotonic_ms {
            Err(ReducerError::HostClockRegressed {
                previous_ms: self.last_host_monotonic_ms,
                current_ms: now_ms,
            })
        } else {
            Ok(())
        }
    }

    /// 该节点owner的session与其离线判定。
    ///
    /// 返回`Option`而不是panic：无人声明该能力时`None`是**正常**情形
    /// （profile没配这类传感器），调用方据此把对应事实解成`NeverObserved`。
    /// 第二个`?`在profile自洽时不可达，真发生时退化为保守的`NeverObserved`，
    /// 而不是凭空造出一个事实——感知内核的失败方向必须是"少知道"，不是"编造"。
    fn owner_session(&self, capability: Capability, now_ms: u64) -> Option<(&NodeSession, bool)> {
        let owner = self.profile.owners_of(capability)?;
        let session = self
            .sessions
            .iter()
            .find(|(node_uid, _)| node_uid == owner)
            .map(|(_, session)| session)?;
        let offline = node_is_offline(session, now_ms, self.profile.freshness.node_offline_ttl_ms);
        Some((session, offline))
    }

    fn session(&self, node_uid: &NodeUid) -> Result<&NodeSession, ReducerError> {
        self.sessions
            .iter()
            .find(|(declared, _)| declared == node_uid)
            .map(|(_, session)| session)
            .ok_or_else(|| ReducerError::UnknownNode(node_uid.clone()))
    }

    fn session_mut(&mut self, node_uid: &NodeUid) -> Result<&mut NodeSession, ReducerError> {
        self.sessions
            .iter_mut()
            .find(|(declared, _)| declared == node_uid)
            .map(|(_, session)| session)
            .ok_or_else(|| ReducerError::UnknownNode(node_uid.clone()))
    }

    /// 节点重启（boot_id变化）时作废它带来的全部证据。
    ///
    /// 清的是**该节点名下能力**对应的槽，不再是"前节点清这几个、后节点清那几个"。
    /// 双板形态下结果与旧实现逐项相同（见`reducer/tests.rs`的回归覆盖）。
    fn clear_node(&mut self, node_uid: &NodeUid) {
        if let Some((_, session)) = self.sessions.iter_mut().find(|(uid, _)| uid == node_uid) {
            *session = NodeSession::default();
        }
        let declared: Vec<Capability> = self
            .profile
            .capabilities_of(node_uid)
            .map(|set| set.iter().collect())
            .unwrap_or_default();
        for capability in declared {
            match capability {
                Capability::FrontGrip => self.front_grip = None,
                Capability::RearGrip => self.rear_grip = None,
                Capability::Motion => self.motion = None,
                Capability::Orientation => self.orientation = None,
                Capability::PrimaryControl => self.primary_control = None,
                // ShockObserved 本来就不落证据槽（见store_observation）
                Capability::ShockDiagnostic => {}
            }
        }
    }

    fn store_observation(
        &mut self,
        observation: &SensorObservationKind,
        received_at_ms: u64,
        source_ref: SourceRef,
    ) {
        let source_refs = vec![source_ref];
        match observation {
            SensorObservationKind::GripContact {
                grip,
                contact,
                strength_milli,
            } => {
                let confidence_milli = match contact {
                    Contact::Engaged => *strength_milli,
                    Contact::Released => strength_milli.complement(),
                };
                let evidence = Evidence {
                    value: *contact,
                    observed_at_monotonic_ms: received_at_ms,
                    fresh_until_monotonic_ms: received_at_ms
                        .saturating_add(self.profile.freshness.grip_ttl_ms),
                    confidence_milli,
                    source_refs,
                };
                // 判别来自**载荷自身**，不再来自"哪个节点发的"。
                // 这正是单板形态成立的前提：一个节点可以同时报前握和后握。
                match grip {
                    GripBinding::Front => self.front_grip = Some(evidence),
                    GripBinding::Rear => self.rear_grip = Some(evidence),
                }
            }
            SensorObservationKind::MotionClass { motion } => {
                self.motion = Some(Evidence {
                    value: *motion,
                    observed_at_monotonic_ms: received_at_ms,
                    fresh_until_monotonic_ms: received_at_ms
                        .saturating_add(self.profile.freshness.motion_ttl_ms),
                    confidence_milli: self.profile.confidence.discrete_observation,
                    source_refs,
                });
            }
            SensorObservationKind::OrientationEstimate {
                gravity_mg_x,
                gravity_mg_y,
                gravity_mg_z,
                quality_milli,
            } => {
                self.orientation = Some(OrientationEvidence {
                    gravity_mg: [*gravity_mg_x, *gravity_mg_y, *gravity_mg_z],
                    observed_at_monotonic_ms: received_at_ms,
                    fresh_until_monotonic_ms: received_at_ms
                        .saturating_add(self.profile.freshness.orientation_ttl_ms),
                    quality_milli: *quality_milli,
                    source_refs,
                });
            }
            SensorObservationKind::AuxControlContact { contact, .. } => {
                self.primary_control = Some(Evidence {
                    value: *contact,
                    observed_at_monotonic_ms: received_at_ms,
                    fresh_until_monotonic_ms: received_at_ms
                        .saturating_add(self.profile.freshness.control_ttl_ms),
                    confidence_milli: self.profile.confidence.discrete_observation,
                    source_refs,
                });
            }
            SensorObservationKind::ShockObserved { .. } => {}
        }
    }

    fn recompute(
        &mut self,
        now_ms: u64,
        unchanged_disposition: ReductionDisposition,
    ) -> Result<ReductionOutput, ReducerError> {
        let resolved = self.resolve(now_ms);
        let same_revision_snapshot = render_snapshot(
            &self.host_boot_id,
            self.current.revision,
            now_ms,
            &resolved,
            &self.current,
        );
        if self
            .current
            .has_same_revision_payload(&same_revision_snapshot)
        {
            self.current = same_revision_snapshot;
            return Ok(ReductionOutput {
                disposition: unchanged_disposition,
                snapshot: self.current.clone(),
            });
        }

        let revision = self
            .current
            .revision
            .checked_add(1)
            .ok_or(ReducerError::RevisionOverflow)?;
        self.current = render_snapshot(
            &self.host_boot_id,
            revision,
            now_ms,
            &resolved,
            &self.current,
        );
        Ok(ReductionOutput {
            disposition: ReductionDisposition::SemanticStateAdvanced,
            snapshot: self.current.clone(),
        })
    }

    fn resolve(&mut self, now_ms: u64) -> ResolvedSnapshot {
        // 每个语义槽位问一次「谁声明了这个能力」，不再写死"前节点管这些、后节点管那些"。
        // 双板形态下 owner(FrontGrip)=front-node、owner(Motion)=front-node、
        // owner(PrimaryControl)=rear-node，取值与旧实现逐项相同。
        // 无人声明该能力是**正常**情形（profile没配这类传感器），解成NeverObserved。
        let front_grip = match self.owner_session(Capability::FrontGrip, now_ms) {
            Some((session, offline)) => {
                resolve_evidence(self.front_grip.as_ref(), session, offline, now_ms)
            }
            None => Resolved::Unknown(UnknownReason::NeverObserved),
        };
        let rear_grip = match self.owner_session(Capability::RearGrip, now_ms) {
            Some((session, offline)) => {
                resolve_evidence(self.rear_grip.as_ref(), session, offline, now_ms)
            }
            None => Resolved::Unknown(UnknownReason::NeverObserved),
        };
        let motion = match self.owner_session(Capability::Motion, now_ms) {
            Some((session, offline)) => {
                resolve_evidence(self.motion.as_ref(), session, offline, now_ms)
            }
            None => Resolved::Unknown(UnknownReason::NeverObserved),
        };
        let pose = match self.owner_session(Capability::Orientation, now_ms) {
            Some((session, offline)) => resolve_pose(
                self.orientation.as_ref(),
                session,
                offline,
                now_ms,
                self.profile.orientation.as_ref(),
            ),
            None => Resolved::Unknown(UnknownReason::NeverObserved),
        };
        let primary_control = match self.owner_session(Capability::PrimaryControl, now_ms) {
            Some((session, offline)) => {
                resolve_evidence(self.primary_control.as_ref(), session, offline, now_ms)
            }
            None => Resolved::Unknown(UnknownReason::NeverObserved),
        };
        let carrier_state = resolve_carrier(
            &front_grip,
            &rear_grip,
            &motion,
            &pose,
            &self.current.carrier_state,
            now_ms,
            self.profile.freshness.standby_dwell_ms,
            &mut self.standby_candidate_since_ms,
            self.profile.confidence,
        );

        // node_health 按 profile 排定（字典序）的节点顺序渲染，与 sessions 同序。
        let node_health = self
            .sessions
            .iter()
            .map(|(node_uid, session)| NodeHealth {
                node_uid: node_uid.clone(),
                state: if node_is_offline(
                    session,
                    now_ms,
                    self.profile.freshness.node_offline_ttl_ms,
                ) {
                    NodeHealthState::Offline
                } else {
                    NodeHealthState::Healthy
                },
            })
            .collect();

        ResolvedSnapshot {
            carrier_state,
            front_grip,
            rear_grip,
            motion,
            pose,
            primary_control,
            node_health,
        }
    }
}

/// 观察→能力的映射，**纯粹是观测自身的函数**：不含节点、不含角色、不返回`Result`。
///
/// 这是本次泛化做对了的标志——旧版签名是
/// `(role: NodeRole, node_uid, observation) -> Result<Capability, _>`，
/// 因为`GripContact`没有判别字段、只能靠"哪个节点发的"来区分前握与后握。
/// 判别字段进契约后，这个参数就彻底消失了。
///
/// 至于"这个节点有没有资格上报该能力"，由`PerceptionProfile`在`ingest`里判定
/// （`CapabilityNotBound`），不由本函数判定。
fn observation_capability(observation: &SensorObservationKind) -> Capability {
    match observation {
        SensorObservationKind::GripContact { grip, .. } => match grip {
            GripBinding::Front => Capability::FrontGrip,
            GripBinding::Rear => Capability::RearGrip,
        },
        SensorObservationKind::MotionClass { .. } => Capability::Motion,
        SensorObservationKind::OrientationEstimate { .. } => Capability::Orientation,
        SensorObservationKind::AuxControlContact { .. } => Capability::PrimaryControl,
        SensorObservationKind::ShockObserved { .. } => Capability::ShockDiagnostic,
    }
}

fn resolve_evidence<T: Clone>(
    evidence: Option<&Evidence<T>>,
    session: &NodeSession,
    node_offline: bool,
    now_ms: u64,
) -> Resolved<T> {
    if session.last_seen_host_monotonic_ms.is_none() {
        return Resolved::Unknown(UnknownReason::NeverObserved);
    }
    if node_offline {
        return Resolved::Unknown(UnknownReason::NodeOffline);
    }
    match evidence {
        None => Resolved::Unknown(UnknownReason::NeverObserved),
        Some(evidence) if now_ms > evidence.fresh_until_monotonic_ms => {
            Resolved::Unknown(UnknownReason::Stale)
        }
        Some(evidence) => Resolved::Known(evidence.clone()),
    }
}

fn resolve_pose(
    evidence: Option<&OrientationEvidence>,
    session: &NodeSession,
    node_offline: bool,
    now_ms: u64,
    calibration: Option<&OrientationCalibration>,
) -> Resolved<Pose> {
    if session.last_seen_host_monotonic_ms.is_none() {
        return Resolved::Unknown(UnknownReason::NeverObserved);
    }
    if node_offline {
        return Resolved::Unknown(UnknownReason::NodeOffline);
    }
    let Some(evidence) = evidence else {
        return Resolved::Unknown(UnknownReason::NeverObserved);
    };
    if now_ms > evidence.fresh_until_monotonic_ms {
        return Resolved::Unknown(UnknownReason::Stale);
    }
    let Some(calibration) = calibration else {
        return Resolved::Unknown(UnknownReason::Uncalibrated);
    };
    if evidence.quality_milli < calibration.minimum_quality_milli {
        return Resolved::Unknown(UnknownReason::SensorFault);
    }

    let raised_distance = squared_distance(evidence.gravity_mg, calibration.raised_gravity_mg);
    let lowered_distance = squared_distance(evidence.gravity_mg, calibration.lowered_gravity_mg);
    let value = if raised_distance < lowered_distance {
        Pose::Raised
    } else {
        Pose::Lowered
    };
    Resolved::Known(Evidence {
        value,
        observed_at_monotonic_ms: evidence.observed_at_monotonic_ms,
        fresh_until_monotonic_ms: evidence.fresh_until_monotonic_ms,
        confidence_milli: evidence.quality_milli,
        source_refs: evidence.source_refs.clone(),
    })
}

fn squared_distance(left: [i32; 3], right: [i32; 3]) -> i128 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| {
            let difference = i128::from(left) - i128::from(right);
            difference * difference
        })
        .sum()
}

#[allow(clippy::too_many_arguments)]
fn resolve_carrier(
    front_grip: &Resolved<Contact>,
    rear_grip: &Resolved<Contact>,
    motion: &Resolved<MotionClass>,
    pose: &Resolved<Pose>,
    previous: &Fact<CarrierState>,
    now_ms: u64,
    standby_dwell_ms: u64,
    standby_candidate_since_ms: &mut Option<u64>,
    confidence: ConfidenceProfile,
) -> Resolved<CarrierState> {
    let front_engaged = known_is(front_grip, &Contact::Engaged);
    let rear_engaged = known_is(rear_grip, &Contact::Engaged);
    let motion_moving = known_is(motion, &MotionClass::Moving);
    let pose_raised = known_is(pose, &Pose::Raised);
    let previous_held = matches!(
        known_fact_value(previous),
        Some(CarrierState::Held | CarrierState::Ready)
    );

    if rear_engaged && pose_raised {
        *standby_candidate_since_ms = None;
        let two_handed = front_engaged;
        let mut evidence = vec![known_meta(rear_grip), known_meta(pose)];
        if two_handed {
            evidence.push(known_meta(front_grip));
        }
        return combine_evidence(
            CarrierState::Ready,
            &evidence,
            if two_handed {
                confidence.ready_two_hand
            } else {
                confidence.ready_single_hand
            },
        );
    }

    if rear_engaged {
        *standby_candidate_since_ms = None;
        return combine_evidence(
            CarrierState::Held,
            &[known_meta(rear_grip)],
            confidence.discrete_observation,
        );
    }
    if front_engaged && motion_moving {
        *standby_candidate_since_ms = None;
        return combine_evidence(
            CarrierState::Held,
            &[known_meta(front_grip), known_meta(motion)],
            confidence.discrete_observation,
        );
    }
    if previous_held && (front_engaged || rear_engaged) {
        *standby_candidate_since_ms = None;
        let evidence = if rear_engaged {
            known_meta(rear_grip)
        } else {
            known_meta(front_grip)
        };
        return combine_evidence(
            CarrierState::Held,
            &[evidence],
            confidence.discrete_observation,
        );
    }

    let explicit_standby = known_is(front_grip, &Contact::Released)
        && known_is(rear_grip, &Contact::Released)
        && known_is(motion, &MotionClass::Idle);
    if explicit_standby {
        let candidate_since = standby_candidate_since_ms.get_or_insert(now_ms);
        if now_ms.saturating_sub(*candidate_since) >= standby_dwell_ms {
            return combine_evidence(
                CarrierState::Standby,
                &[
                    known_meta(front_grip),
                    known_meta(rear_grip),
                    known_meta(motion),
                ],
                confidence.discrete_observation,
            );
        }
        if previous_held {
            return combine_evidence(
                CarrierState::Held,
                &[
                    known_meta(front_grip),
                    known_meta(rear_grip),
                    known_meta(motion),
                ],
                confidence.discrete_observation,
            );
        }
        return Resolved::Unknown(UnknownReason::NeverObserved);
    }
    *standby_candidate_since_ms = None;

    if previous_held && all_known(&[front_grip, rear_grip]) && matches!(motion, Resolved::Known(_))
    {
        return combine_evidence(
            CarrierState::Held,
            &[
                known_meta(front_grip),
                known_meta(rear_grip),
                known_meta(motion),
            ],
            confidence.discrete_observation,
        );
    }

    Resolved::Unknown(first_unknown_reason(&[
        unknown_reason(front_grip),
        unknown_reason(rear_grip),
        unknown_reason(motion),
    ]))
}

fn known_is<T: PartialEq>(resolved: &Resolved<T>, expected: &T) -> bool {
    matches!(resolved, Resolved::Known(evidence) if &evidence.value == expected)
}

fn known_meta<T>(resolved: &Resolved<T>) -> EvidenceMeta {
    match resolved {
        Resolved::Known(evidence) => EvidenceMeta {
            observed_at_monotonic_ms: evidence.observed_at_monotonic_ms,
            fresh_until_monotonic_ms: evidence.fresh_until_monotonic_ms,
            confidence_milli: evidence.confidence_milli,
            source_refs: evidence.source_refs.clone(),
        },
        Resolved::Unknown(_) => unreachable!("caller checked that evidence is known"),
    }
}

fn known_fact_value<T>(fact: &Fact<T>) -> Option<&T> {
    match fact {
        Fact::Known { value, .. } => Some(value),
        Fact::Unknown { .. } => None,
    }
}

fn all_known<T>(resolved: &[&Resolved<T>]) -> bool {
    resolved
        .iter()
        .all(|value| matches!(value, Resolved::Known(_)))
}

fn unknown_reason<T>(resolved: &Resolved<T>) -> Option<UnknownReason> {
    match resolved {
        Resolved::Known(_) => None,
        Resolved::Unknown(reason) => Some(*reason),
    }
}

fn first_unknown_reason(reasons: &[Option<UnknownReason>]) -> UnknownReason {
    [
        UnknownReason::NodeOffline,
        UnknownReason::Stale,
        UnknownReason::SensorFault,
        UnknownReason::Uncalibrated,
        UnknownReason::NeverObserved,
    ]
    .into_iter()
    .find(|candidate| reasons.contains(&Some(*candidate)))
    .unwrap_or(UnknownReason::NeverObserved)
}

fn combine_evidence<T: Clone>(
    value: T,
    evidence: &[EvidenceMeta],
    confidence_cap: Permille,
) -> Resolved<T> {
    let observed_at_monotonic_ms = evidence
        .iter()
        .map(|item| item.observed_at_monotonic_ms)
        .max()
        .unwrap_or(0);
    let fresh_until_monotonic_ms = evidence
        .iter()
        .map(|item| item.fresh_until_monotonic_ms)
        .min()
        .unwrap_or(observed_at_monotonic_ms);
    let confidence_milli = evidence.iter().fold(confidence_cap, |current, item| {
        current.minimum(item.confidence_milli)
    });
    let mut source_refs = Vec::new();
    for item in evidence {
        for source_ref in &item.source_refs {
            if source_refs.len() == ailive_gun_spirit_contracts::MAX_SOURCE_REFS {
                break;
            }
            if !source_refs.contains(source_ref) {
                source_refs.push(source_ref.clone());
            }
        }
    }
    Resolved::Known(Evidence {
        value,
        observed_at_monotonic_ms,
        fresh_until_monotonic_ms,
        confidence_milli,
        source_refs,
    })
}

fn node_is_offline(session: &NodeSession, now_ms: u64, offline_ttl_ms: u64) -> bool {
    session
        .last_seen_host_monotonic_ms
        .is_none_or(|last_seen| now_ms > last_seen.saturating_add(offline_ttl_ms))
}

fn render_snapshot(
    host_boot_id: &BootId,
    revision: u64,
    now_ms: u64,
    resolved: &ResolvedSnapshot,
    previous: &PerceptionState,
) -> PerceptionState {
    PerceptionState {
        contract_version: ContractVersion::V0_3,
        host_boot_id: host_boot_id.clone(),
        revision,
        produced_at_monotonic_ms: now_ms,
        carrier_state: render_fact(&resolved.carrier_state, &previous.carrier_state, revision),
        front_grip: render_fact(&resolved.front_grip, &previous.front_grip, revision),
        rear_grip: render_fact(&resolved.rear_grip, &previous.rear_grip, revision),
        motion: render_fact(&resolved.motion, &previous.motion, revision),
        pose: render_fact(&resolved.pose, &previous.pose, revision),
        primary_control: render_fact(
            &resolved.primary_control,
            &previous.primary_control,
            revision,
        ),
        node_health: resolved.node_health.clone(),
    }
}

fn render_fact<T: Clone>(resolved: &Resolved<T>, previous: &Fact<T>, revision: u64) -> Fact<T> {
    match resolved {
        Resolved::Known(evidence) => Fact::Known {
            value: evidence.value.clone(),
            observed_at_monotonic_ms: evidence.observed_at_monotonic_ms,
            fresh_until_monotonic_ms: evidence.fresh_until_monotonic_ms,
            confidence_milli: evidence.confidence_milli,
            source_refs: evidence.source_refs.clone(),
        },
        Resolved::Unknown(reason) => Fact::Unknown {
            reason: *reason,
            since_revision: match previous {
                Fact::Unknown {
                    reason: previous_reason,
                    since_revision,
                } if previous_reason == reason => *since_revision,
                _ => revision,
            },
        },
    }
}

fn never_observed<T>() -> Fact<T> {
    Fact::Unknown {
        reason: UnknownReason::NeverObserved,
        since_revision: 0,
    }
}

#[cfg(test)]
mod tests;
