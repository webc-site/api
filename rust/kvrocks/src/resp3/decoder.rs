use std::str::from_utf8;

use bytes::{Buf, BytesMut};
use memchr::memchr;

use crate::{
    error::{Error, Result},
    resp3::types::Value,
};

pub struct Decoder;

impl Decoder {
    pub fn decode(src: &mut BytesMut) -> Result<Option<Value>> {
        if src.is_empty() {
            return Ok(None);
        }
        match Self::parse_value(src)? {
            Some((val, consumed)) => {
                src.advance(consumed);
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn find_crlf(src: &[u8]) -> Option<usize> {
        let mut offset = 0;
        while let Some(pos) = memchr(b'\r', &src[offset..]) {
            let abs = offset + pos;
            if abs + 1 < src.len() && src[abs + 1] == b'\n' {
                return Some(abs);
            }
            offset = abs + 1;
        }
        None
    }

    #[inline]
    fn parse_i64(src: &[u8]) -> Result<i64> {
        if src.is_empty() {
            return Err(Error::Protocol("empty integer".into()));
        }
        let (neg, digits) = match src[0] {
            b'-' => (true, &src[1..]),
            b'+' => (false, &src[1..]),
            _ => (false, src),
        };
        if digits.is_empty() {
            return Err(Error::Protocol("no digits in integer".into()));
        }
        let mut val: i64 = 0;
        for &b in digits {
            if !b.is_ascii_digit() {
                return Err(Error::Protocol("invalid digit in integer".into()));
            }
            val = val
                .checked_mul(10)
                .and_then(|v| v.checked_add((b - b'0') as i64))
                .ok_or_else(|| Error::Protocol("integer overflow".into()))?;
        }
        Ok(if neg { -val } else { val })
    }

    #[inline]
    fn parse_usize(src: &[u8]) -> Result<usize> {
        if src.is_empty() {
            return Err(Error::Protocol("empty length".into()));
        }
        let mut val: usize = 0;
        for &b in src {
            if !b.is_ascii_digit() {
                return Err(Error::Protocol("invalid digit in length".into()));
            }
            val = val
                .checked_mul(10)
                .and_then(|v| v.checked_add((b - b'0') as usize))
                .ok_or_else(|| Error::Protocol("length overflow".into()))?;
        }
        Ok(val)
    }

    #[inline]
    fn parse_crlf_str(
        rest: &[u8],
        ctor: impl FnOnce(String) -> Value,
    ) -> Result<Option<(Value, usize)>> {
        if let Some(crlf) = Self::find_crlf(rest) {
            let s = from_utf8(&rest[..crlf])
                .map_err(|e| Error::Protocol(format!("invalid string: {e}")))?
                .to_string();
            Ok(Some((ctor(s), crlf + 3)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn read_blob_slice(src: &[u8], crlf: usize, len: usize) -> Option<(&[u8], usize)> {
        let data_start = crlf + 2;
        let total_needed = 1 + data_start + len + 2;
        if src.len() < total_needed {
            return None;
        }
        Some((&src[1 + data_start..1 + data_start + len], total_needed))
    }

    fn parse_value(src: &[u8]) -> Result<Option<(Value, usize)>> {
        if src.is_empty() {
            return Ok(None);
        }

        let marker = src[0];
        let rest = &src[1..];

        match marker {
            b'+' => Self::parse_crlf_str(rest, Value::SimpleString),
            b'-' => Self::parse_crlf_str(rest, Value::Error),
            b'(' => Self::parse_crlf_str(rest, Value::BigNumber),
            b':' => {
                // Integer: :<num>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let num = Self::parse_i64(&rest[..crlf])?;
                    Ok(Some((Value::Integer(num), crlf + 3)))
                } else {
                    Ok(None)
                }
            }
            b'_' => {
                // Null: _\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    Ok(Some((Value::Null, crlf + 3)))
                } else {
                    Ok(None)
                }
            }
            b',' => {
                // Double: ,<float>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let s = from_utf8(&rest[..crlf])
                        .map_err(|e| Error::Protocol(format!("invalid float string: {e}")))?;
                    let d: f64 = match s {
                        "inf" => f64::INFINITY,
                        "-inf" => f64::NEG_INFINITY,
                        "nan" => f64::NAN,
                        _ => s
                            .parse()
                            .map_err(|e| Error::Protocol(format!("invalid double: {e}")))?,
                    };
                    Ok(Some((Value::Double(d), crlf + 3)))
                } else {
                    Ok(None)
                }
            }
            b'#' => {
                // Boolean: #t\r\n or #f\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let b = match &rest[..crlf] {
                        b"t" => true,
                        b"f" => false,
                        _ => return Err(Error::Protocol("invalid boolean value".into())),
                    };
                    Ok(Some((Value::Boolean(b), crlf + 3)))
                } else {
                    Ok(None)
                }
            }
            b'$' => {
                // Blob String: $<len>\r\n<data>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let len = Self::parse_i64(&rest[..crlf])?;
                    if len == -1 {
                        return Ok(Some((Value::Null, crlf + 3)));
                    }
                    let len = usize::try_from(len)
                        .map_err(|_| Error::Protocol("negative blob string length".into()))?;
                    if let Some((data, total_needed)) = Self::read_blob_slice(src, crlf, len) {
                        Ok(Some((Value::BlobString(data.to_vec()), total_needed)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            b'!' => {
                // Blob Error: !<len>\r\n<error>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let len = Self::parse_usize(&rest[..crlf])?;
                    if let Some((data, total_needed)) = Self::read_blob_slice(src, crlf, len) {
                        let data = from_utf8(data)
                            .map_err(|e| {
                                Error::Protocol(format!("invalid blob error string: {e}"))
                            })?
                            .to_string();
                        Ok(Some((Value::BlobError(data), total_needed)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            b'=' => {
                // Verbatim String: =<len>\r\n<format:3>:<data>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let len = Self::parse_usize(&rest[..crlf])?;
                    if let Some((raw, total_needed)) = Self::read_blob_slice(src, crlf, len) {
                        if raw.len() < 4 || raw[3] != b':' {
                            return Err(Error::Protocol("invalid verbatim string payload".into()));
                        }
                        let format = [raw[0], raw[1], raw[2]];
                        let data = from_utf8(&raw[4..])
                            .map_err(|e| {
                                Error::Protocol(format!("invalid verbatim string content: {e}"))
                            })?
                            .to_string();
                        Ok(Some((Value::VerbatimString { format, data }, total_needed)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            b'*' | b'~' | b'>' => {
                // Array: *<len>\r\n / Set: ~<len>\r\n / Push: ><len>\r\n
                if let Some(crlf) = Self::find_crlf(rest) {
                    let len = Self::parse_i64(&rest[..crlf])?;
                    if len == -1 {
                        return Ok(Some((Value::Null, crlf + 3)));
                    }
                    let count = usize::try_from(len)
                        .map_err(|_| Error::Protocol("negative aggregate length".into()))?;
                    let mut offset = 1 + crlf + 2;
                    let mut items = Vec::with_capacity(count);

                    for _ in 0..count {
                        if offset >= src.len() {
                            return Ok(None);
                        }
                        match Self::parse_value(&src[offset..])? {
                            Some((val, consumed)) => {
                                items.push(val);
                                offset += consumed;
                            }
                            None => return Ok(None),
                        }
                    }

                    let val = match marker {
                        b'*' => Value::Array(items),
                        b'~' => Value::Set(items),
                        b'>' => Value::Push(items),
                        _ => unreachable!(),
                    };
                    Ok(Some((val, offset)))
                } else {
                    Ok(None)
                }
            }
            b'%' => {
                // Map: %<len>\r\n (len is number of key/value pairs)
                if let Some(crlf) = Self::find_crlf(rest) {
                    let len = Self::parse_i64(&rest[..crlf])?;
                    if len == -1 {
                        return Ok(Some((Value::Null, crlf + 3)));
                    }
                    let count = usize::try_from(len)
                        .map_err(|_| Error::Protocol("negative map length".into()))?;
                    let mut offset = 1 + crlf + 2;
                    let mut pairs = Vec::with_capacity(count);

                    for _ in 0..count {
                        if offset >= src.len() {
                            return Ok(None);
                        }
                        let key = match Self::parse_value(&src[offset..])? {
                            Some((val, consumed)) => {
                                offset += consumed;
                                val
                            }
                            None => return Ok(None),
                        };

                        if offset >= src.len() {
                            return Ok(None);
                        }
                        let value = match Self::parse_value(&src[offset..])? {
                            Some((val, consumed)) => {
                                offset += consumed;
                                val
                            }
                            None => return Ok(None),
                        };

                        pairs.push((key, value));
                    }

                    Ok(Some((Value::Map(pairs), offset)))
                } else {
                    Ok(None)
                }
            }
            b'|' => {
                // Attribute: |<len>\r\n (skip attribute map, then parse underlying payload)
                if let Some(crlf) = Self::find_crlf(rest) {
                    let count = Self::parse_usize(&rest[..crlf])?;
                    let mut offset = 1 + crlf + 2;

                    for _ in 0..(count * 2) {
                        if offset >= src.len() {
                            return Ok(None);
                        }
                        match Self::parse_value(&src[offset..])? {
                            Some((_, consumed)) => {
                                offset += consumed;
                            }
                            None => return Ok(None),
                        }
                    }

                    // 解析真正的数据帧
                    if offset >= src.len() {
                        return Ok(None);
                    }
                    match Self::parse_value(&src[offset..])? {
                        Some((val, consumed)) => Ok(Some((val, offset + consumed))),
                        None => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            other => Err(Error::Protocol(format!(
                "unknown resp3 marker: {other} (0x{other:02x})"
            ))),
        }
    }
}
