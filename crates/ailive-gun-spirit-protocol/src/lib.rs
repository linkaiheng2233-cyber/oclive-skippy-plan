//! P0-A 帧协议 v1：分帧、CRC 校验与消息类型（V4 验证对象）。
//!
//! 线格式（小端）：
//!
//! ```text
//! [0..2)   magic      0x47 0x53（ASCII "GS"）
//! [2]      version    固定 1
//! [3]      type       消息类型
//! [4]      flags      保留位，接收方不得因未知位拒绝
//! [5..7)   length     payload 字节数（u16 小端）
//! [7..)    payload
//! [..+2)   crc        CRC-16/CCITT-FALSE，覆盖 header + payload（不含自身）
//! ```
//!
//! 设计要点：
//! - **先按 `type` 分派，再做类型专属校验**（上层职责，本 crate 只做分帧与完整性）。
//! - 坏 magic 由调用方**滑动窗口**重新同步；本 crate 返回 `BadMagic` 让它知道该前进。
//! - 超出 `MAX_PAYLOAD` 属硬错误（草案要求断开），不做部分接收。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod payload;

/// 线首两字节："GS"。
pub const MAGIC: [u8; 2] = [0x47, 0x53];
/// 当前协议版本。
pub const VERSION: u8 = 1;
/// 头部长度。
pub const HEADER_LEN: usize = 7;
/// 尾部 CRC 长度。
pub const CRC_LEN: usize = 2;
/// 单帧 payload 上限。
pub const MAX_PAYLOAD: usize = 4096;
/// 单帧总长度上限。
pub const MAX_FRAME: usize = HEADER_LEN + MAX_PAYLOAD + CRC_LEN;

/// 消息类型。数值是线格式的一部分，**不得重排**。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0x01,
    State = 0x02,
    Event = 0x03,
    Expression = 0x04,
    Ack = 0x05,
    Nack = 0x06,
    Ping = 0x07,
    Pong = 0x08,
    /// 可选高频流；不参与连续性校验，丢包即丢。
    Aim = 0x09,
}

impl MessageType {
    /// 全部已定义类型，供编解码与文档一致性测试使用。
    pub const ALL: [Self; 9] = [
        Self::Hello,
        Self::State,
        Self::Event,
        Self::Expression,
        Self::Ack,
        Self::Nack,
        Self::Ping,
        Self::Pong,
        Self::Aim,
    ];

    /// 从线格式数值解析；未知值返回 `None`（上层应按 `Nack`/断开处置）。
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Hello),
            0x02 => Some(Self::State),
            0x03 => Some(Self::Event),
            0x04 => Some(Self::Expression),
            0x05 => Some(Self::Ack),
            0x06 => Some(Self::Nack),
            0x07 => Some(Self::Ping),
            0x08 => Some(Self::Pong),
            0x09 => Some(Self::Aim),
            _ => None,
        }
    }

    /// 线格式数值。
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// 解析成功的帧视图（借用输入缓冲区）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<'a> {
    pub message_type: MessageType,
    pub flags: u8,
    pub payload: &'a [u8],
}

/// 分帧错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// 输入还不足以构成完整帧，调用方应继续读取。
    NeedMoreData,
    /// magic 不匹配：调用方应向前滑动一字节后重试。
    BadMagic,
    /// 版本不支持：按草案断开连接。
    UnsupportedVersion(u8),
    /// 未知消息类型。
    UnknownType(u8),
    /// payload 超过 `MAX_PAYLOAD`：按草案硬断开。
    PayloadTooLarge(usize),
    /// CRC 不符。
    CrcMismatch { expected: u16, actual: u16 },
    /// 输出缓冲区太小。
    OutputTooSmall(usize),
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NeedMoreData => formatter.write_str("need more data"),
            Self::BadMagic => formatter.write_str("bad magic"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported version {version}"),
            Self::UnknownType(value) => write!(formatter, "unknown message type {value:#04x}"),
            Self::PayloadTooLarge(size) => write!(formatter, "payload too large: {size}"),
            Self::CrcMismatch { expected, actual } => {
                write!(
                    formatter,
                    "crc mismatch expected {expected:#06x} actual {actual:#06x}"
                )
            }
            Self::OutputTooSmall(needed) => write!(formatter, "output buffer too small: {needed}"),
        }
    }
}

impl core::error::Error for FrameError {}

/// CRC-16/CCITT-FALSE：多项式 `0x1021`，初值 `0xFFFF`，不反射，xorout `0x0000`。
#[must_use]
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in data {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// 给定 payload 长度时整帧所需的字节数。
#[must_use]
pub const fn encoded_len(payload_len: usize) -> usize {
    HEADER_LEN + payload_len + CRC_LEN
}

/// 编码一帧到 `output`，返回写入的字节数。
pub fn encode(
    output: &mut [u8],
    message_type: MessageType,
    flags: u8,
    payload: &[u8],
) -> Result<usize, FrameError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(FrameError::PayloadTooLarge(payload.len()));
    }
    let total = encoded_len(payload.len());
    if output.len() < total {
        return Err(FrameError::OutputTooSmall(total));
    }
    output[0..2].copy_from_slice(&MAGIC);
    output[2] = VERSION;
    output[3] = message_type.as_u8();
    output[4] = flags;
    output[5..7].copy_from_slice(
        &u16::try_from(payload.len())
            .unwrap_or(u16::MAX)
            .to_le_bytes(),
    );
    output[7..7 + payload.len()].copy_from_slice(payload);
    let crc = crc16_ccitt_false(&output[..HEADER_LEN + payload.len()]);
    output[HEADER_LEN + payload.len()..total].copy_from_slice(&crc.to_le_bytes());
    Ok(total)
}

/// 解析一帧，返回帧视图与消耗字节数。
///
/// 错误语义见 `FrameError`：只有 `NeedMoreData` 表示"继续读"，
/// 其余错误都应由调用方决定是滑动重同步还是断开。
pub fn decode(input: &[u8]) -> Result<(Frame<'_>, usize), FrameError> {
    if input.len() < HEADER_LEN {
        return Err(FrameError::NeedMoreData);
    }
    if input[0..2] != MAGIC {
        return Err(FrameError::BadMagic);
    }
    if input[2] != VERSION {
        return Err(FrameError::UnsupportedVersion(input[2]));
    }
    let message_type = MessageType::from_u8(input[3]).ok_or(FrameError::UnknownType(input[3]))?;
    let flags = input[4];
    let length = usize::from(u16::from_le_bytes([input[5], input[6]]));
    if length > MAX_PAYLOAD {
        return Err(FrameError::PayloadTooLarge(length));
    }
    let total = encoded_len(length);
    if input.len() < total {
        return Err(FrameError::NeedMoreData);
    }
    let expected = u16::from_le_bytes([input[total - CRC_LEN], input[total - 1]]);
    let actual = crc16_ccitt_false(&input[..HEADER_LEN + length]);
    if expected != actual {
        return Err(FrameError::CrcMismatch { expected, actual });
    }
    Ok((
        Frame {
            message_type,
            flags,
            payload: &input[HEADER_LEN..HEADER_LEN + length],
        },
        total,
    ))
}

#[cfg(test)]
mod tests {
    // 测试代码用 expect 表达"此处不可能失败"最直观；产品代码仍受 workspace 约束。
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    #[test]
    fn crc_matches_standard_check_value() {
        // CRC-16/CCITT-FALSE 的标准校验值：输入 "123456789" 应得 0x29B1。
        assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
    }

    #[test]
    fn round_trip_every_message_type() {
        let payload = [0xAA_u8, 0x00, 0x7F, 0xFF];
        for message_type in MessageType::ALL {
            let mut buffer = [0u8; MAX_FRAME];
            let written = encode(&mut buffer, message_type, 0x02, &payload).expect("encode");
            assert_eq!(written, encoded_len(payload.len()));
            let (frame, consumed) = decode(&buffer[..written]).expect("decode");
            assert_eq!(consumed, written);
            assert_eq!(frame.message_type, message_type);
            assert_eq!(frame.flags, 0x02);
            assert_eq!(frame.payload, payload);
        }
    }

    #[test]
    fn empty_payload_round_trip() {
        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::Ping, 0, &[]).expect("encode");
        assert_eq!(written, HEADER_LEN + CRC_LEN);
        let (frame, _) = decode(&buffer[..written]).expect("decode");
        assert!(frame.payload.is_empty());
    }

    #[test]
    fn truncated_input_asks_for_more_data() {
        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::State, 0, &[1, 2, 3]).expect("encode");
        for shorter in 0..written - 1 {
            assert_eq!(
                decode(&buffer[..shorter]),
                Err(FrameError::NeedMoreData),
                "长度 {shorter} 应要求继续读取"
            );
        }
    }

    #[test]
    fn bad_magic_is_reported_for_resync() {
        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::Hello, 0, &[9]).expect("encode");
        buffer[0] = 0x00;
        assert_eq!(decode(&buffer[..written]), Err(FrameError::BadMagic));
    }

    #[test]
    fn version_and_type_are_validated() {
        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::Event, 0, &[1]).expect("encode");
        buffer[2] = 7;
        assert_eq!(
            decode(&buffer[..written]),
            Err(FrameError::UnsupportedVersion(7))
        );

        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::Event, 0, &[1]).expect("encode");
        buffer[3] = 0xEE;
        assert_eq!(
            decode(&buffer[..written]),
            Err(FrameError::UnknownType(0xEE))
        );
    }

    #[test]
    fn corrupted_payload_is_caught_by_crc() {
        let mut buffer = [0u8; MAX_FRAME];
        let written = encode(&mut buffer, MessageType::State, 0, &[1, 2, 3, 4]).expect("encode");
        buffer[HEADER_LEN] ^= 0xFF;
        match decode(&buffer[..written]) {
            Err(FrameError::CrcMismatch { expected, actual }) => assert_ne!(expected, actual),
            other => panic!("期望 CrcMismatch，实际 {other:?}"),
        }
    }

    #[test]
    fn oversize_payload_is_rejected_at_encode() {
        let mut buffer = [0u8; MAX_FRAME + 1];
        let payload = [0u8; MAX_PAYLOAD + 1];
        assert_eq!(
            encode(&mut buffer, MessageType::State, 0, &payload),
            Err(FrameError::PayloadTooLarge(MAX_PAYLOAD + 1))
        );
    }

    #[test]
    fn declared_length_beyond_limit_is_hard_error() {
        // 头部声称超长 payload：按草案属硬错误，不是"继续读"。
        let mut header = [0u8; HEADER_LEN];
        header[0..2].copy_from_slice(&MAGIC);
        header[2] = VERSION;
        header[3] = MessageType::State.as_u8();
        header[5..7].copy_from_slice(&(u16::try_from(MAX_PAYLOAD + 1).unwrap_or(0)).to_le_bytes());
        assert_eq!(
            decode(&header),
            Err(FrameError::PayloadTooLarge(MAX_PAYLOAD + 1))
        );
    }

    #[test]
    fn frames_can_be_streamed_back_to_back() {
        let mut buffer = [0u8; MAX_FRAME * 2];
        let first = encode(&mut buffer, MessageType::Ping, 0, &[]).expect("encode ping");
        let second = encode(&mut buffer[first..], MessageType::Pong, 0, &[7]).expect("encode pong");
        let (frame_one, consumed_one) = decode(&buffer[..first + second]).expect("decode first");
        assert_eq!(frame_one.message_type, MessageType::Ping);
        let (frame_two, _) = decode(&buffer[consumed_one..first + second]).expect("decode second");
        assert_eq!(frame_two.message_type, MessageType::Pong);
        assert_eq!(frame_two.payload, [7]);
    }
}
