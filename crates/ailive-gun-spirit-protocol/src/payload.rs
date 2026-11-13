//! 载荷编解码（S2 第一层）：HELLO / EXPR / ACK / NACK / PING / PONG。
//!
//! 设计取向：**手写定长二进制**——不引入 serde、不依赖堆，板上与主机共用同一份实现。
//! 每个类型都同时给出 `encode` 与 `decode`，并用"精确长度"策略拒绝多余字节，
//! 避免调用方把错位数据当成合法载荷。
//!
//! `STATE` / `EVENT` 涉及契约里的大结构与 128 字节标识符上限，放在下一层实现。

use core::str::from_utf8;

/// 与契约 `string_id!` 的上限一致：`boot_id` 最多 128 字节。
pub const MAX_BOOT_ID_LEN: usize = 128;
/// 表情标识（`neutral` / `happy` 等）的线格式上限。
pub const MAX_EXPRESSION_LEN: usize = 32;

/// `NACK` 的原因码。数值是线格式的一部分，不得重排。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NackCode {
    UnsupportedType = 1,
    BadPayload = 2,
    UnsupportedVersion = 3,
    NotReady = 4,
    Busy = 5,
}

impl NackCode {
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::UnsupportedType),
            2 => Some(Self::BadPayload),
            3 => Some(Self::UnsupportedVersion),
            4 => Some(Self::NotReady),
            5 => Some(Self::Busy),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// 载荷层错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadError {
    /// 输入不足。
    Truncated,
    /// 输出缓冲区不足（携带需要的字节数）。
    OutputTooSmall(usize),
    /// 字段超出允许长度。
    TooLong,
    /// 字符串不是合法 UTF-8。
    InvalidUtf8,
    /// 数值不在允许集合内（例如未知的原因码）。
    InvalidValue,
    /// 解码后仍有剩余字节：拒绝，避免错位被当成合法载荷。
    TrailingBytes(usize),
}

impl core::fmt::Display for PayloadError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("payload truncated"),
            Self::OutputTooSmall(needed) => write!(formatter, "output too small: {needed}"),
            Self::TooLong => formatter.write_str("field too long"),
            Self::InvalidUtf8 => formatter.write_str("invalid utf-8"),
            Self::InvalidValue => formatter.write_str("invalid value"),
            Self::TrailingBytes(count) => write!(formatter, "trailing bytes: {count}"),
        }
    }
}

impl core::error::Error for PayloadError {}

fn require(output: &[u8], needed: usize) -> Result<(), PayloadError> {
    if output.len() < needed {
        return Err(PayloadError::OutputTooSmall(needed));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// HELLO：契约版本 + 节点 boot_id + 能力位图
// ---------------------------------------------------------------------------

/// 握手：告诉对端"我是谁、说什么版本、有什么能力"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hello<'a> {
    /// 契约主版本（v0.3 → 0）。
    pub contract_major: u16,
    /// 契约次版本（v0.3 → 3）。
    pub contract_minor: u16,
    /// 节点启动标识，1..=128 字节。
    pub boot_id: &'a str,
    /// 能力位图，由上层约定各位含义。
    pub capabilities: u32,
}

impl<'a> Hello<'a> {
    /// 编码所需字节数。
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        2 + 2 + 1 + self.boot_id.len() + 4
    }

    /// 编码到 `output`，返回写入字节数。
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        if self.boot_id.is_empty() || self.boot_id.len() > MAX_BOOT_ID_LEN {
            return Err(PayloadError::TooLong);
        }
        let needed = self.encoded_len();
        require(output, needed)?;
        output[0..2].copy_from_slice(&self.contract_major.to_le_bytes());
        output[2..4].copy_from_slice(&self.contract_minor.to_le_bytes());
        output[4] = u8::try_from(self.boot_id.len()).map_err(|_| PayloadError::TooLong)?;
        output[5..5 + self.boot_id.len()].copy_from_slice(self.boot_id.as_bytes());
        let capability_offset = 5 + self.boot_id.len();
        output[capability_offset..capability_offset + 4]
            .copy_from_slice(&self.capabilities.to_le_bytes());
        Ok(needed)
    }

    /// 从 `bytes` 解码；必须恰好耗尽输入。
    pub fn decode(bytes: &'a [u8]) -> Result<Self, PayloadError> {
        if bytes.len() < 5 {
            return Err(PayloadError::Truncated);
        }
        let contract_major = u16::from_le_bytes([bytes[0], bytes[1]]);
        let contract_minor = u16::from_le_bytes([bytes[2], bytes[3]]);
        let boot_length = usize::from(bytes[4]);
        if boot_length == 0 || boot_length > MAX_BOOT_ID_LEN {
            return Err(PayloadError::TooLong);
        }
        let capabilities_offset = 5 + boot_length;
        if bytes.len() < capabilities_offset + 4 {
            return Err(PayloadError::Truncated);
        }
        let boot_id =
            from_utf8(&bytes[5..capabilities_offset]).map_err(|_| PayloadError::InvalidUtf8)?;
        let capabilities = u32::from_le_bytes([
            bytes[capabilities_offset],
            bytes[capabilities_offset + 1],
            bytes[capabilities_offset + 2],
            bytes[capabilities_offset + 3],
        ]);
        let consumed = capabilities_offset + 4;
        if bytes.len() != consumed {
            return Err(PayloadError::TrailingBytes(bytes.len() - consumed));
        }
        Ok(Self {
            contract_major,
            contract_minor,
            boot_id,
            capabilities,
        })
    }
}

// ---------------------------------------------------------------------------
// EXPR：表情下行
// ---------------------------------------------------------------------------

/// 表情下发：只传语义标识（例如 `happy`），不传像素。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expression<'a> {
    pub expression: &'a str,
}

impl<'a> Expression<'a> {
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        1 + self.expression.len()
    }

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        if self.expression.is_empty() || self.expression.len() > MAX_EXPRESSION_LEN {
            return Err(PayloadError::TooLong);
        }
        let needed = self.encoded_len();
        require(output, needed)?;
        output[0] = u8::try_from(self.expression.len()).map_err(|_| PayloadError::TooLong)?;
        output[1..needed].copy_from_slice(self.expression.as_bytes());
        Ok(needed)
    }

    pub fn decode(bytes: &'a [u8]) -> Result<Self, PayloadError> {
        if bytes.is_empty() {
            return Err(PayloadError::Truncated);
        }
        let length = usize::from(bytes[0]);
        if length == 0 || length > MAX_EXPRESSION_LEN {
            return Err(PayloadError::TooLong);
        }
        if bytes.len() < 1 + length {
            return Err(PayloadError::Truncated);
        }
        let expression = from_utf8(&bytes[1..1 + length]).map_err(|_| PayloadError::InvalidUtf8)?;
        if bytes.len() != 1 + length {
            return Err(PayloadError::TrailingBytes(bytes.len() - 1 - length));
        }
        Ok(Self { expression })
    }
}

// ---------------------------------------------------------------------------
// ACK / NACK
// ---------------------------------------------------------------------------

/// 确认：告诉对端"我收到你序号为 N 的帧"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack {
    pub acknowledged_sequence: u32,
}

impl Ack {
    pub const LEN: usize = 4;

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        require(output, Self::LEN)?;
        output[0..4].copy_from_slice(&self.acknowledged_sequence.to_le_bytes());
        Ok(Self::LEN)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        if bytes.len() < Self::LEN {
            return Err(PayloadError::Truncated);
        }
        if bytes.len() > Self::LEN {
            return Err(PayloadError::TrailingBytes(bytes.len() - Self::LEN));
        }
        Ok(Self {
            acknowledged_sequence: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }
}

/// 拒绝：告诉对端"你序号为 N 的帧被我拒了，原因是 C"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nack {
    pub rejected_sequence: u32,
    pub code: NackCode,
}

impl Nack {
    pub const LEN: usize = 5;

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        require(output, Self::LEN)?;
        output[0..4].copy_from_slice(&self.rejected_sequence.to_le_bytes());
        output[4] = self.code.as_u8();
        Ok(Self::LEN)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        if bytes.len() < Self::LEN {
            return Err(PayloadError::Truncated);
        }
        if bytes.len() > Self::LEN {
            return Err(PayloadError::TrailingBytes(bytes.len() - Self::LEN));
        }
        let code = NackCode::from_u8(bytes[4]).ok_or(PayloadError::InvalidValue)?;
        Ok(Self {
            rejected_sequence: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            code,
        })
    }
}

// ---------------------------------------------------------------------------
// PING / PONG
// ---------------------------------------------------------------------------

/// 心跳/授时探针：`nonce` 原样回显，用于往返估计与断线判定。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping {
    pub nonce: u32,
}

/// `PONG` 与 `PING` 同构。
pub type Pong = Ping;

impl Ping {
    pub const LEN: usize = 4;

    pub fn encode(&self, output: &mut [u8]) -> Result<usize, PayloadError> {
        require(output, Self::LEN)?;
        output[0..4].copy_from_slice(&self.nonce.to_le_bytes());
        Ok(Self::LEN)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        if bytes.len() < Self::LEN {
            return Err(PayloadError::Truncated);
        }
        if bytes.len() > Self::LEN {
            return Err(PayloadError::TrailingBytes(bytes.len() - Self::LEN));
        }
        Ok(Self {
            nonce: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    #[test]
    fn hello_round_trip() {
        let hello = Hello {
            contract_major: 0,
            contract_minor: 3,
            boot_id: "bt-A1B2C3-0007",
            capabilities: 0b1_0101,
        };
        let mut buffer = [0u8; 160];
        let written = hello.encode(&mut buffer).expect("encode");
        let decoded = Hello::decode(&buffer[..written]).expect("decode");
        assert_eq!(decoded, hello);
    }

    #[test]
    fn hello_rejects_empty_and_oversize_boot_id() {
        let mut buffer = [0u8; 160];
        let empty = Hello {
            contract_major: 0,
            contract_minor: 3,
            boot_id: "",
            capabilities: 0,
        };
        assert_eq!(empty.encode(&mut buffer), Err(PayloadError::TooLong));

        let long = "x".repeat(MAX_BOOT_ID_LEN + 1);
        let oversize = Hello {
            contract_major: 0,
            contract_minor: 3,
            boot_id: &long,
            capabilities: 0,
        };
        assert_eq!(oversize.encode(&mut buffer), Err(PayloadError::TooLong));
    }

    #[test]
    fn hello_rejects_truncation_and_trailing_bytes() {
        let hello = Hello {
            contract_major: 0,
            contract_minor: 3,
            boot_id: "bt-01",
            capabilities: 7,
        };
        let mut buffer = [0u8; 64];
        let written = hello.encode(&mut buffer).expect("encode");
        for shorter in 0..written {
            assert_eq!(
                Hello::decode(&buffer[..shorter]),
                Err(PayloadError::Truncated),
                "长度 {shorter} 应报截断"
            );
        }
        buffer[written] = 0xFF;
        assert_eq!(
            Hello::decode(&buffer[..written + 1]),
            Err(PayloadError::TrailingBytes(1))
        );
    }

    #[test]
    fn expression_round_trip_and_bounds() {
        let expression = Expression {
            expression: "happy",
        };
        let mut buffer = [0u8; 64];
        let written = expression.encode(&mut buffer).expect("encode");
        assert_eq!(
            Expression::decode(&buffer[..written]).expect("decode"),
            expression
        );

        let too_long = "y".repeat(MAX_EXPRESSION_LEN + 1);
        let oversize = Expression {
            expression: &too_long,
        };
        assert_eq!(oversize.encode(&mut buffer), Err(PayloadError::TooLong));
    }

    #[test]
    fn ack_nack_ping_round_trip() {
        let mut buffer = [0u8; 16];

        let ack = Ack {
            acknowledged_sequence: 42,
        };
        let written = ack.encode(&mut buffer).expect("encode ack");
        assert_eq!(Ack::decode(&buffer[..written]).expect("decode"), ack);

        let nack = Nack {
            rejected_sequence: 7,
            code: NackCode::BadPayload,
        };
        let written = nack.encode(&mut buffer).expect("encode nack");
        assert_eq!(Nack::decode(&buffer[..written]).expect("decode"), nack);

        let ping = Ping { nonce: 0xDEAD_BEEF };
        let written = ping.encode(&mut buffer).expect("encode ping");
        assert_eq!(Ping::decode(&buffer[..written]).expect("decode"), ping);
    }

    #[test]
    fn nack_rejects_unknown_code() {
        let mut buffer = [0u8; Nack::LEN];
        buffer[4] = 0x7F;
        assert_eq!(Nack::decode(&buffer), Err(PayloadError::InvalidValue));
    }

    #[test]
    fn fixed_length_payloads_reject_extra_bytes() {
        let mut buffer = [0u8; 8];
        let ack = Ack {
            acknowledged_sequence: 1,
        };
        let written = ack.encode(&mut buffer).expect("encode");
        assert_eq!(
            Ack::decode(&buffer[..written + 1]),
            Err(PayloadError::TrailingBytes(1))
        );
    }
}
