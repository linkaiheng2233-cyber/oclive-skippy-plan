//! 二进制读写游标（STATE / EVENT 载荷共用）。
//!
//! 所有多字节整数一律**小端**，与分帧层长度字段一致。
//! 读取侧对任何越界都返回 `PayloadError::Truncated`，不 panic、不越界。

use core::str::from_utf8;

use crate::payload::PayloadError;

/// 输入游标。
pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn take(&mut self, length: usize) -> Result<&'a [u8], PayloadError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(PayloadError::Truncated)?;
        if end > self.bytes.len() {
            return Err(PayloadError::Truncated);
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8, PayloadError> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, PayloadError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn u64(&mut self) -> Result<u64, PayloadError> {
        let bytes = self.take(8)?;
        let mut value = [0u8; 8];
        value.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(value))
    }

    /// 读长度前缀（u8）的 UTF-8 字符串，长度须在 `1..=max`。
    pub fn string(&mut self, max: usize) -> Result<&'a str, PayloadError> {
        let length = usize::from(self.u8()?);
        if length == 0 || length > max {
            return Err(PayloadError::TooLong);
        }
        let bytes = self.take(length)?;
        from_utf8(bytes).map_err(|_| PayloadError::InvalidUtf8)
    }

    /// 确认输入被恰好耗尽，否则报多余字节。
    pub fn finish(self) -> Result<(), PayloadError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining != 0 {
            return Err(PayloadError::TrailingBytes(remaining));
        }
        Ok(())
    }
}

/// 输出游标。
pub struct Writer<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a> Writer<'a> {
    #[must_use]
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn put(&mut self, value: &[u8]) -> Result<(), PayloadError> {
        let end = self
            .offset
            .checked_add(value.len())
            .ok_or(PayloadError::OutputTooSmall(usize::MAX))?;
        if end > self.bytes.len() {
            return Err(PayloadError::OutputTooSmall(end));
        }
        self.bytes[self.offset..end].copy_from_slice(value);
        self.offset = end;
        Ok(())
    }

    pub fn u8(&mut self, value: u8) -> Result<(), PayloadError> {
        self.put(&[value])
    }

    pub fn u16(&mut self, value: u16) -> Result<(), PayloadError> {
        self.put(&value.to_le_bytes())
    }

    pub fn u64(&mut self, value: u64) -> Result<(), PayloadError> {
        self.put(&value.to_le_bytes())
    }

    /// 写长度前缀（u8）的字符串；空串与超长都拒绝。
    pub fn string(&mut self, value: &str, max: usize) -> Result<(), PayloadError> {
        let length = value.len();
        if length == 0 || length > max {
            return Err(PayloadError::TooLong);
        }
        self.u8(u8::try_from(length).map_err(|_| PayloadError::TooLong)?)?;
        self.put(value.as_bytes())
    }

    /// 已写入字节数。
    #[must_use]
    pub fn written(&self) -> usize {
        self.offset
    }
}
