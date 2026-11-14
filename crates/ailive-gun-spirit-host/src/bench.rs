//! 台架会话核心（M2）。
//!
//! 这一层只管**语义**，不碰 socket/串口：把"收到一帧之后该怎么判断"写成可测试的状态机，
//! 再由 `bin/bench_host.rs` 用模拟设备或 TCP 驱动它。
//!
//! 覆盖的四类判据（与 `PROTOCOL_V1_DRAFT.md` 对齐）：
//! 1. 握手：必须先 `HELLO`，版本不匹配直接拒绝；
//! 2. 状态：`revision` 只能单调不降，否则拒绝且不覆盖当前快照；
//! 3. 事件：按 `event_id` 去重，重传不得二次触发；
//! 4. 会话：`boot_id` 变化（设备重启）必须重建基线，不把旧会话的 revision 当参考。

use core::fmt;

use ailive_gun_spirit_protocol::event::WireEvent;
use ailive_gun_spirit_protocol::payload::{Hello, PayloadError};
use ailive_gun_spirit_protocol::state::WireState;

/// 期望的契约主版本（v0.3 → 0）。
pub const EXPECTED_CONTRACT_MAJOR: u16 = 0;
/// 期望的契约次版本（v0.3 → 3）。
pub const EXPECTED_CONTRACT_MINOR: u16 = 3;
/// 记住的 `event_id` 上限，防止无限增长。
pub const MAX_REMEMBERED_EVENTS: usize = 256;

/// 拒绝原因：台架程序据此决定是记录、丢弃还是断开。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// 尚未握手就收到状态或事件。
    HandshakeRequired,
    /// 契约版本不匹配（携带对方声称的 major/minor）。
    ContractMismatch { major: u16, minor: u16 },
    /// `revision` 倒退。
    RevisionRegressed { previous: u64, current: u64 },
    /// 帧层或载荷层解析失败。
    Malformed(PayloadError),
}

impl fmt::Display for Rejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HandshakeRequired => formatter.write_str("handshake required"),
            Self::ContractMismatch { major, minor } => {
                write!(formatter, "contract mismatch: peer claims v{major}.{minor}")
            }
            Self::RevisionRegressed { previous, current } => {
                write!(formatter, "revision regressed: {previous} -> {current}")
            }
            Self::Malformed(error) => write!(formatter, "malformed: {error}"),
        }
    }
}

impl From<PayloadError> for Rejection {
    fn from(error: PayloadError) -> Self {
        Self::Malformed(error)
    }
}

/// 台架统计。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stats {
    pub hellos: u64,
    pub states: u64,
    pub states_rejected: u64,
    pub events_new: u64,
    pub events_duplicate: u64,
    pub events_rejected: u64,
    pub reconnects: u64,
}

/// 会话状态机。
#[derive(Debug, Default)]
pub struct Session {
    boot_id: Option<String>,
    revision: u64,
    event_ids: Vec<String>,
    stats: Stats,
    snapshot: Option<WireState>,
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前快照（未握手或未收到状态时为 `None`）。
    #[must_use]
    pub fn snapshot(&self) -> Option<&WireState> {
        self.snapshot.as_ref()
    }

    #[must_use]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    #[must_use]
    pub fn boot_id(&self) -> Option<&str> {
        self.boot_id.as_deref()
    }

    /// 处理 `HELLO`。版本不匹配返回拒绝，但会话仍保持未握手状态。
    pub fn on_hello(&mut self, hello: &Hello<'_>) -> Result<(), Rejection> {
        if hello.contract_major != EXPECTED_CONTRACT_MAJOR
            || hello.contract_minor != EXPECTED_CONTRACT_MINOR
        {
            return Err(Rejection::ContractMismatch {
                major: hello.contract_major,
                minor: hello.contract_minor,
            });
        }
        let rebooted = self.boot_id.as_deref() != Some(hello.boot_id);
        if rebooted {
            self.boot_id = Some(hello.boot_id.to_owned());
            self.revision = 0;
            self.event_ids.clear();
            self.snapshot = None;
            if self.stats.hellos > 0 {
                self.stats.reconnects += 1;
            }
        }
        self.stats.hellos += 1;
        Ok(())
    }

    /// 处理 `STATE`。
    pub fn on_state(&mut self, state: WireState) -> Result<(), Rejection> {
        if self.boot_id.is_none() {
            return Err(Rejection::HandshakeRequired);
        }
        if state.revision < self.revision {
            self.stats.states_rejected += 1;
            return Err(Rejection::RevisionRegressed {
                previous: self.revision,
                current: state.revision,
            });
        }
        self.revision = state.revision;
        self.snapshot = Some(state);
        self.stats.states += 1;
        Ok(())
    }

    /// 处理 `EVENT`。返回 `true` 表示这是一条新事件，`false` 表示重复（已被去重）。
    pub fn on_event(&mut self, event: &WireEvent) -> Result<bool, Rejection> {
        if self.boot_id.is_none() {
            return Err(Rejection::HandshakeRequired);
        }
        if self.event_ids.iter().any(|known| known == &event.event_id) {
            self.stats.events_duplicate += 1;
            return Ok(false);
        }
        if self.event_ids.len() >= MAX_REMEMBERED_EVENTS {
            // 有界丢弃最旧的一条，避免无限增长；真实实现可换成环形缓冲。
            self.event_ids.remove(0);
        }
        self.event_ids.push(event.event_id.clone());
        self.stats.events_new += 1;
        Ok(true)
    }

    /// 记录一次无法解析的帧；会话本身不因此失效。
    pub fn note_malformed(&mut self) {
        self.stats.events_rejected += 1;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use ailive_gun_spirit_protocol::state::{
        WireFact, WireNodeHealth, CARRIER_HELD, CARRIER_STANDBY, CONTACT_ENGAGED,
    };

    use super::*;

    fn hello(boot: &str) -> Hello<'_> {
        Hello {
            contract_major: EXPECTED_CONTRACT_MAJOR,
            contract_minor: EXPECTED_CONTRACT_MINOR,
            boot_id: boot,
            capabilities: 0,
        }
    }

    fn fact_known(value: u8) -> WireFact {
        WireFact::Known {
            value_code: value,
            observed_at_ms: 1,
            fresh_until_ms: 2,
            confidence_milli: 900,
        }
    }

    fn state(revision: u64) -> WireState {
        WireState {
            contract_major: EXPECTED_CONTRACT_MAJOR,
            contract_minor: EXPECTED_CONTRACT_MINOR,
            producer_boot_id: String::from("board-boot-1"),
            revision,
            produced_at_ms: revision * 10,
            carrier_state: fact_known(CARRIER_HELD),
            front_grip: fact_known(CONTACT_ENGAGED),
            rear_grip: fact_known(CONTACT_ENGAGED),
            motion: WireFact::Unknown {
                reason_code: 2,
                since_revision: revision,
            },
            pose: fact_known(0),
            primary_control: fact_known(1),
            node_health: vec![WireNodeHealth {
                node_uid: String::from("node-front"),
                state_code: 0,
            }],
        }
    }

    fn event(id: &str) -> WireEvent {
        WireEvent {
            contract_major: EXPECTED_CONTRACT_MAJOR,
            contract_minor: EXPECTED_CONTRACT_MINOR,
            event_id: String::from(id),
            producer_boot_id: String::from("board-boot-1"),
            occurred_at_ms: 5,
            caused_by_revision: 1,
            kind_code: 0,
        }
    }

    #[test]
    fn state_before_handshake_is_rejected() {
        let mut session = Session::new();
        assert_eq!(
            session.on_state(state(1)),
            Err(Rejection::HandshakeRequired)
        );
    }

    #[test]
    fn hello_with_wrong_version_is_rejected() {
        let mut session = Session::new();
        let wrong = Hello {
            contract_major: 0,
            contract_minor: 2,
            boot_id: "board-boot-1",
            capabilities: 0,
        };
        assert_eq!(
            session.on_hello(&wrong),
            Err(Rejection::ContractMismatch { major: 0, minor: 2 })
        );
        assert!(session.boot_id().is_none(), "版本不符不得建立会话");
    }

    #[test]
    fn revision_must_not_regress() {
        let mut session = Session::new();
        session.on_hello(&hello("board-boot-1")).expect("hello");
        session.on_state(state(5)).expect("state 5");
        assert_eq!(
            session.on_state(state(4)),
            Err(Rejection::RevisionRegressed {
                previous: 5,
                current: 4
            })
        );
        assert_eq!(
            session.snapshot().map(|state| state.revision),
            Some(5),
            "被拒的快照不得覆盖当前值"
        );
    }

    #[test]
    fn duplicate_events_are_counted_once() {
        let mut session = Session::new();
        session.on_hello(&hello("board-boot-1")).expect("hello");
        assert_eq!(session.on_event(&event("evt-1")), Ok(true));
        assert_eq!(session.on_event(&event("evt-1")), Ok(false));
        assert_eq!(session.on_event(&event("evt-2")), Ok(true));
        assert_eq!(session.stats().events_new, 2);
        assert_eq!(session.stats().events_duplicate, 1);
    }

    #[test]
    fn reboot_rebuilds_baseline() {
        let mut session = Session::new();
        session.on_hello(&hello("board-boot-1")).expect("hello 1");
        session.on_state(state(9)).expect("state 9");
        session.on_event(&event("evt-1")).expect("event");

        // 设备重启：新 boot_id、revision 从 0 重新开始。
        session.on_hello(&hello("board-boot-2")).expect("hello 2");
        assert_eq!(session.stats().reconnects, 1);
        assert!(session.snapshot().is_none(), "重启后旧快照必须清空");
        session.on_state(state(0)).expect("新会话接受 revision 0");
        // 事件 ID 集合也被清空（旧会话的 ID 不再参与去重）。
        assert_eq!(session.on_event(&event("evt-1")), Ok(true));
    }

    #[test]
    fn malformed_frames_do_not_kill_session() {
        let mut session = Session::new();
        session.on_hello(&hello("board-boot-1")).expect("hello");
        session.note_malformed();
        assert!(session.on_state(state(1)).is_ok());
        assert_eq!(session.stats().events_rejected, 1);
    }

    #[test]
    fn standby_state_is_accepted_as_snapshot() {
        let mut session = Session::new();
        session.on_hello(&hello("board-boot-1")).expect("hello");
        let mut snapshot = state(1);
        snapshot.carrier_state = fact_known(CARRIER_STANDBY);
        session.on_state(snapshot).expect("state");
        assert_eq!(
            session.snapshot().map(|state| state.revision),
            Some(1),
            "Standby 也是合法快照"
        );
    }
}
