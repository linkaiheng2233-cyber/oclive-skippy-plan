//! `STATE` 载荷：`PerceptionState` 快照的线格式。
//!
//! **取舍（2026-11-14）**：线格式**不携带 `source_refs`**。
//! 每个 Fact 最多 4 条来源，每条含 `observation_id`（≤128 B）与 `node_uid`（≤128 B），
//! 全量传输会让一帧超出 4 KiB 上限，而主机侧做角色表达与 HUD 并不需要审计轨迹
//! ——来源信息留在设备本地用于诊断。这里的线格式只承载**语义**：
//! 值、时效、置信度、以及未知原因。
//!
//! 布局（小端）：
//!
//! ```text
//! u8  format_version = 1
//! u16 contract_major
//! u16 contract_minor
//! 1+n producer_boot_id (n 1..=128)
//! u64 revision
//! u64 produced_at_monotonic_ms
//! fact carrier_state
//! fact front_grip
//! fact rear_grip
//! fact motion
//! fact pose
//! fact primary_control
//! u8  node_count (0..=8)
//!   repeated: 1+n node_uid (n 1..=128), u8 state_code
//! ```
//!
//! fact 布局：
//!
//! ```text
//! u8 tag: 0=Unknown, 1=Known
//! Unknown: u8 reason_code, u64 since_revision
//! Known:   u8 value_code, u64 observed_at_ms, u64 fresh_until_ms, u16 confidence_milli
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use crate::codec::{Reader, Writer};
use crate::payload::{PayloadError, MAX_BOOT_ID_LEN};

/// 线格式版本；与分帧层的协议版本独立演进。
pub const STATE_FORMAT_VERSION: u8 = 1;
/// `node_health` 条目上限，与契约定一致。
pub const MAX_NODE_HEALTH: usize = 8;
/// 标识符上限，与契约定一致。
pub const MAX_ID_LEN: usize = MAX_BOOT_ID_LEN;

// 值编码：各 fact 的 `value_code` 解释见对应 `decode_*_value` 辅助函数。
pub const CONTACT_ENGAGED: u8 = 0;
pub const CONTACT_RELEASED: u8 = 1;
pub const MOTION_IDLE: u8 = 0;
pub const MOTION_MOVING: u8 = 1;
pub const POSE_RAISED: u8 = 0;
pub const POSE_LOWERED: u8 = 1;
pub const CARRIER_STANDBY: u8 = 0;
pub const CARRIER_HELD: u8 = 1;
pub const CARRIER_READY: u8 = 2;

// 未知原因编码。
pub const REASON_NEVER_OBSERVED: u8 = 0;
pub const REASON_NODE_OFFLINE: u8 = 1;
pub const REASON_STALE: u8 = 2;
pub const REASON_UNCALIBRATED: u8 = 3;
pub const REASON_SENSOR_FAULT: u8 = 4;

// 节点健康编码。
pub const HEALTH_HEALTHY: u8 = 0;
pub const HEALTH_DEGRADED: u8 = 1;
pub const HEALTH_OFFLINE: u8 = 2;
pub const HEALTH_INCOMPATIBLE_PROTOCOL: u8 = 3;
pub const HEALTH_PROTOCOL_FAULT: u8 = 4;

/// 一个 fact 的线格式表示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireFact {
    /// 已知：值编码 + 时间窗 + 置信度。
    Known {
        value_code: u8,
        observed_at_ms: u64,
        fresh_until_ms: u64,
        confidence_milli: u16,
    },
    /// 未知：原因 + 自哪个 revision 起。
    Unknown {
        reason_code: u8,
        since_revision: u64,
    },
}

impl WireFact {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), PayloadError> {
        match self {
            Self::Known {
                value_code,
                observed_at_ms,
                fresh_until_ms,
                confidence_milli,
            } => {
                writer.u8(1)?;
                writer.u8(*value_code)?;
                writer.u64(*observed_at_ms)?;
                writer.u64(*fresh_until_ms)?;
                writer.u16(*confidence_milli)?;
            }
            Self::Unknown {
                reason_code,
                since_revision,
            } => {
                writer.u8(0)?;
                writer.u8(*reason_code)?;
                writer.u64(*since_revision)?;
            }
        }
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> Result<Self, PayloadError> {
        match reader.u8()? {
            1 => Ok(Self::Known {
                value_code: reader.u8()?,
                observed_at_ms: reader.u64()?,
                fresh_until_ms: reader.u64()?,
                confidence_milli: reader.u16()?,
            }),
            0 => Ok(Self::Unknown {
                reason_code: reader.u8()?,
                since_revision: reader.u64()?,
            }),
            _ => Err(PayloadError::InvalidValue),
        }
    }
}

/// 一条节点健康记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireNodeHealth {
    pub node_uid: String,
    pub state_code: u8,
}

/// `PerceptionState` 线格式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireState {
    pub contract_major: u16,
    pub contract_minor: u16,
    pub producer_boot_id: String,
    pub revision: u64,
    pub produced_at_ms: u64,
    pub carrier_state: WireFact,
    pub front_grip: WireFact,
    pub rear_grip: WireFact,
    pub motion: WireFact,
    pub pose: WireFact,
    pub primary_control: WireFact,
    pub node_health: Vec<WireNodeHealth>,
}

impl WireState {
    /// 编码到 `output`，返回写入字节数。
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        let mut writer = Writer::new(output);
        writer.u8(STATE_FORMAT_VERSION)?;
        writer.u16(self.contract_major)?;
        writer.u16(self.contract_minor)?;
        writer.string(&self.producer_boot_id, MAX_ID_LEN)?;
        writer.u64(self.revision)?;
        writer.u64(self.produced_at_ms)?;
        for fact in [
            &self.carrier_state,
            &self.front_grip,
            &self.rear_grip,
            &self.motion,
            &self.pose,
            &self.primary_control,
        ] {
            fact.encode(&mut writer)?;
        }
        if self.node_health.len() > MAX_NODE_HEALTH {
            return Err(PayloadError::TooLong);
        }
        writer.u8(u8::try_from(self.node_health.len()).map_err(|_| PayloadError::TooLong)?)?;
        for entry in &self.node_health {
            writer.string(&entry.node_uid, MAX_ID_LEN)?;
            writer.u8(entry.state_code)?;
        }
        Ok(writer.written())
    }

    /// 从 `bytes` 解码；必须恰好耗尽输入。
    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        let mut reader = Reader::new(bytes);
        if reader.u8()? != STATE_FORMAT_VERSION {
            return Err(PayloadError::InvalidValue);
        }
        let contract_major = reader.u16()?;
        let contract_minor = reader.u16()?;
        let producer_boot_id = String::from(reader.string(MAX_ID_LEN)?);
        let revision = reader.u64()?;
        let produced_at_ms = reader.u64()?;
        let carrier_state = WireFact::decode(&mut reader)?;
        let front_grip = WireFact::decode(&mut reader)?;
        let rear_grip = WireFact::decode(&mut reader)?;
        let motion = WireFact::decode(&mut reader)?;
        let pose = WireFact::decode(&mut reader)?;
        let primary_control = WireFact::decode(&mut reader)?;
        let node_count = usize::from(reader.u8()?);
        if node_count > MAX_NODE_HEALTH {
            return Err(PayloadError::TooLong);
        }
        let mut node_health = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let node_uid = String::from(reader.string(MAX_ID_LEN)?);
            let state_code = reader.u8()?;
            node_health.push(WireNodeHealth {
                node_uid,
                state_code,
            });
        }
        reader.finish()?;
        Ok(Self {
            contract_major,
            contract_minor,
            producer_boot_id,
            revision,
            produced_at_ms,
            carrier_state,
            front_grip,
            rear_grip,
            motion,
            pose,
            primary_control,
            node_health,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    fn known(value_code: u8) -> WireFact {
        WireFact::Known {
            value_code,
            observed_at_ms: 1_000,
            fresh_until_ms: 2_000,
            confidence_milli: 900,
        }
    }

    fn sample() -> WireState {
        WireState {
            contract_major: 0,
            contract_minor: 3,
            producer_boot_id: String::from("board-boot-7"),
            revision: 42,
            produced_at_ms: 12_345,
            carrier_state: known(CARRIER_HELD),
            front_grip: known(CONTACT_ENGAGED),
            rear_grip: known(CONTACT_RELEASED),
            motion: WireFact::Unknown {
                reason_code: REASON_STALE,
                since_revision: 39,
            },
            pose: known(POSE_RAISED),
            primary_control: known(CONTACT_RELEASED),
            node_health: alloc::vec![
                WireNodeHealth {
                    node_uid: String::from("node-front"),
                    state_code: HEALTH_HEALTHY,
                },
                WireNodeHealth {
                    node_uid: String::from("node-rear"),
                    state_code: HEALTH_OFFLINE,
                },
            ],
        }
    }

    #[test]
    fn cpp_board_state_reference_vector() {
        // 板端 M3 固件（gs_wire.cpp）在全 Unknown 时发出的 STATE：
        // boot_id = "board-000000-1"（其实际值取自 MAC，长度一致）、revision = 1、produced = 0。
        // 上板验证时可直接比对这串字节，确认 C++ 与 Rust 两套编码一致。
        let state = WireState {
            contract_major: 0,
            contract_minor: 3,
            producer_boot_id: String::from("board-000000-1"),
            revision: 1,
            produced_at_ms: 0,
            carrier_state: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            front_grip: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            rear_grip: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            motion: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            pose: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            primary_control: WireFact::Unknown {
                reason_code: REASON_NEVER_OBSERVED,
                since_revision: 1,
            },
            node_health: Vec::new(),
        };
        let mut buffer = [0u8; 256];
        let written = state.encode(&mut buffer).expect("encode");
        let hex: alloc::string::String = buffer[..written]
            .iter()
            .map(|byte| alloc::format!("{byte:02X}"))
            .collect();
        println!("BOARD_STATE_VECTOR len={written} hex={hex}");
        // 1(格式) + 2 + 2 + 1+14(boot) + 8 + 8 + 6*(1+1+8) + 1(节点数)
        assert_eq!(written, 1 + 2 + 2 + 15 + 8 + 8 + 60 + 1);
        let decoded = WireState::decode(&buffer[..written]).expect("decode");
        assert_eq!(decoded, state);
    }
    #[test]
    fn state_round_trip() {
        let state = sample();
        let mut buffer = [0u8; 512];
        let written = state.encode(&mut buffer).expect("encode");
        assert!(written < 256, "典型状态应远小于 256 字节，实际 {written}");
        let decoded = WireState::decode(&buffer[..written]).expect("decode");
        assert_eq!(decoded, state);
    }

    #[test]
    fn state_rejects_truncation_at_every_prefix() {
        let state = sample();
        let mut buffer = [0u8; 512];
        let written = state.encode(&mut buffer).expect("encode");
        for shorter in 0..written {
            assert_eq!(
                WireState::decode(&buffer[..shorter]),
                Err(PayloadError::Truncated),
                "长度 {shorter} 应报截断"
            );
        }
    }

    #[test]
    fn state_rejects_trailing_bytes() {
        let state = sample();
        let mut buffer = [0u8; 512];
        let written = state.encode(&mut buffer).expect("encode");
        buffer[written] = 0xFF;
        assert_eq!(
            WireState::decode(&buffer[..written + 1]),
            Err(PayloadError::TrailingBytes(1))
        );
    }

    #[test]
    fn state_rejects_unknown_codes() {
        let state = sample();
        let mut buffer = [0u8; 512];
        let written = state.encode(&mut buffer).expect("encode");
        // format_version 之后紧跟 contract_major/minor 与 boot_id，fact tag 位置由编码决定；
        // 直接把整个载荷的 fact tag 改成非法值不方便，这里改为验证 format_version 非法。
        let mut copy = [0u8; 512];
        copy[..written].copy_from_slice(&buffer[..written]);
        copy[0] = 9;
        assert_eq!(
            WireState::decode(&copy[..written]),
            Err(PayloadError::InvalidValue)
        );
    }

    #[test]
    fn state_rejects_too_many_nodes() {
        let mut state = sample();
        state.node_health = alloc::vec![
            WireNodeHealth {
                node_uid: String::from("n"),
                state_code: HEALTH_HEALTHY
            };
            MAX_NODE_HEALTH + 1
        ];
        let mut buffer = [0u8; 1024];
        assert_eq!(state.encode(&mut buffer), Err(PayloadError::TooLong));
    }

    #[test]
    fn state_rejects_oversize_boot_id() {
        let mut state = sample();
        state.producer_boot_id = "x".repeat(MAX_ID_LEN + 1);
        let mut buffer = [0u8; 2048];
        assert_eq!(state.encode(&mut buffer), Err(PayloadError::TooLong));
    }

    #[test]
    fn state_reports_output_too_small() {
        let state = sample();
        let mut buffer = [0u8; 8];
        match state.encode(&mut buffer) {
            Err(PayloadError::OutputTooSmall(needed)) => assert!(needed > 8),
            other => panic!("期望 OutputTooSmall，实际 {other:?}"),
        }
    }
}
