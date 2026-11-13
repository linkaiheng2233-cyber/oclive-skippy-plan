//! `EVENT` 载荷：`DeviceEvent` 的线格式。
//!
//! 与 `STATE` 不同，事件必须携带 `event_id`——主机侧靠它去重
//! （同一事件被链路重传时只应触发一次角色反应）。
//!
//! 布局（小端）：
//!
//! ```text
//! u8  format_version = 1
//! u16 contract_major
//! u16 contract_minor
//! 1+n event_id   (n 1..=128)
//! 1+n host_boot_id (n 1..=128)
//! u64 occurred_at_monotonic_ms
//! u64 caused_by_perception_revision
//! u8  kind_code (0..=5)
//! ```

use alloc::string::String;

use crate::codec::{Reader, Writer};
use crate::payload::{PayloadError, MAX_BOOT_ID_LEN};

/// 线格式版本。
pub const EVENT_FORMAT_VERSION: u8 = 1;
/// 标识符上限，与契约定一致。
pub const MAX_ID_LEN: usize = MAX_BOOT_ID_LEN;

/// `DeviceEventKind` 的线格式编码；数值即协议，不得重排。
pub const KIND_CARRIER_HELD_ENTERED: u8 = 0;
pub const KIND_CARRIER_HELD_EXITED: u8 = 1;
pub const KIND_CARRIER_READY_ENTERED: u8 = 2;
pub const KIND_CARRIER_READY_EXITED: u8 = 3;
pub const KIND_CONTROL_PRIMARY_ENGAGED: u8 = 4;
pub const KIND_CONTROL_PRIMARY_RELEASED: u8 = 5;

/// 事件线格式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEvent {
    pub contract_major: u16,
    pub contract_minor: u16,
    pub event_id: String,
    pub host_boot_id: String,
    pub occurred_at_ms: u64,
    pub caused_by_revision: u64,
    pub kind_code: u8,
}

impl WireEvent {
    /// `kind_code` 是否为已定义值。
    #[must_use]
    pub fn is_known_kind(code: u8) -> bool {
        code <= KIND_CONTROL_PRIMARY_RELEASED
    }

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        if !Self::is_known_kind(self.kind_code) {
            return Err(PayloadError::InvalidValue);
        }
        let mut writer = Writer::new(output);
        writer.u8(EVENT_FORMAT_VERSION)?;
        writer.u16(self.contract_major)?;
        writer.u16(self.contract_minor)?;
        writer.string(&self.event_id, MAX_ID_LEN)?;
        writer.string(&self.host_boot_id, MAX_ID_LEN)?;
        writer.u64(self.occurred_at_ms)?;
        writer.u64(self.caused_by_revision)?;
        writer.u8(self.kind_code)?;
        Ok(writer.written())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        let mut reader = Reader::new(bytes);
        if reader.u8()? != EVENT_FORMAT_VERSION {
            return Err(PayloadError::InvalidValue);
        }
        let contract_major = reader.u16()?;
        let contract_minor = reader.u16()?;
        let event_id = String::from(reader.string(MAX_ID_LEN)?);
        let host_boot_id = String::from(reader.string(MAX_ID_LEN)?);
        let occurred_at_ms = reader.u64()?;
        let caused_by_revision = reader.u64()?;
        let kind_code = reader.u8()?;
        if !Self::is_known_kind(kind_code) {
            return Err(PayloadError::InvalidValue);
        }
        reader.finish()?;
        Ok(Self {
            contract_major,
            contract_minor,
            event_id,
            host_boot_id,
            occurred_at_ms,
            caused_by_revision,
            kind_code,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    fn sample() -> WireEvent {
        WireEvent {
            contract_major: 0,
            contract_minor: 3,
            event_id: String::from("evt-000123"),
            host_boot_id: String::from("board-boot-7"),
            occurred_at_ms: 98_765,
            caused_by_revision: 41,
            kind_code: KIND_CARRIER_READY_ENTERED,
        }
    }

    #[test]
    fn event_round_trip() {
        let event = sample();
        let mut buffer = [0u8; 256];
        let written = event.encode(&mut buffer).expect("encode");
        assert!(written < 64, "事件应远小于 64 字节，实际 {written}");
        assert_eq!(
            WireEvent::decode(&buffer[..written]).expect("decode"),
            event
        );
    }

    #[test]
    fn event_rejects_truncation_at_every_prefix() {
        let event = sample();
        let mut buffer = [0u8; 256];
        let written = event.encode(&mut buffer).expect("encode");
        for shorter in 0..written {
            assert_eq!(
                WireEvent::decode(&buffer[..shorter]),
                Err(PayloadError::Truncated),
                "长度 {shorter} 应报截断"
            );
        }
    }

    #[test]
    fn event_rejects_unknown_kind() {
        let mut event = sample();
        event.kind_code = 6;
        let mut buffer = [0u8; 256];
        assert_eq!(event.encode(&mut buffer), Err(PayloadError::InvalidValue));

        // 构造一个 kind 非法的字节流，验证解码侧同样拒绝。
        let good = sample();
        let mut encoded = [0u8; 256];
        let written = good.encode(&mut encoded).expect("encode");
        encoded[written - 1] = 6;
        assert_eq!(
            WireEvent::decode(&encoded[..written]),
            Err(PayloadError::InvalidValue)
        );
    }

    #[test]
    fn event_rejects_trailing_bytes() {
        let event = sample();
        let mut buffer = [0u8; 256];
        let written = event.encode(&mut buffer).expect("encode");
        buffer[written] = 0x00;
        assert_eq!(
            WireEvent::decode(&buffer[..written + 1]),
            Err(PayloadError::TrailingBytes(1))
        );
    }

    #[test]
    fn event_rejects_empty_and_oversize_ids() {
        let mut event = sample();
        event.event_id = String::new();
        let mut buffer = [0u8; 512];
        assert_eq!(event.encode(&mut buffer), Err(PayloadError::TooLong));

        event.event_id = "e".repeat(MAX_ID_LEN + 1);
        assert_eq!(event.encode(&mut buffer), Err(PayloadError::TooLong));
    }

    #[test]
    fn all_six_kinds_round_trip() {
        for kind in 0..=KIND_CONTROL_PRIMARY_RELEASED {
            let mut event = sample();
            event.kind_code = kind;
            let mut buffer = [0u8; 256];
            let written = event.encode(&mut buffer).expect("encode");
            assert_eq!(
                WireEvent::decode(&buffer[..written])
                    .expect("decode")
                    .kind_code,
                kind
            );
        }
    }
}
