use std::str::from_utf8;

use rapidhash::{HashMapExt, RapidHashMap as HashMap};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    SimpleString(String),
    Error(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
    BlobString(Vec<u8>),
    BlobError(String),
    VerbatimString { format: [u8; 3], data: String },
    BigNumber(String),
    Array(Vec<Value>),
    Set(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Push(Vec<Value>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::SimpleString(s) | Self::BigNumber(s) => Some(s.as_str()),
            Self::BlobString(b) => from_utf8(b).ok(),
            Self::VerbatimString { data, .. } => Some(data.as_str()),
            _ => None,
        }
    }

    pub fn into_string(self) -> Result<String> {
        match self {
            Self::SimpleString(s) | Self::BigNumber(s) => Ok(s),
            Self::BlobString(b) => {
                String::from_utf8(b).map_err(|e| Error::Protocol(format!("invalid utf-8: {e}")))
            }
            Self::VerbatimString { data, .. } => Ok(data),
            Self::Integer(i) => Ok(i.to_string()),
            Self::Double(d) => Ok(d.to_string()),
            Self::Boolean(b) => Ok(b.to_string()),
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e)),
            Self::Null => Err(Error::Protocol("unexpected null value".into())),
            _ => Err(Error::Protocol("cannot convert value to string".into())),
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BlobString(b) => Some(b.as_slice()),
            Self::SimpleString(s) | Self::BigNumber(s) => Some(s.as_bytes()),
            Self::VerbatimString { data, .. } => Some(data.as_bytes()),
            _ => None,
        }
    }

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::BlobString(b) => Ok(b),
            Self::SimpleString(s) | Self::BigNumber(s) => Ok(s.into_bytes()),
            Self::VerbatimString { data, .. } => Ok(data.into_bytes()),
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e)),
            Self::Null => Err(Error::Protocol("unexpected null value".into())),
            _ => Err(Error::Protocol("cannot convert value to bytes".into())),
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match self {
            Self::Integer(i) => Ok(*i),
            Self::SimpleString(s) | Self::BigNumber(s) => s
                .parse()
                .map_err(|e| Error::Protocol(format!("invalid integer: {e}"))),
            Self::BlobString(b) => from_utf8(b)
                .map_err(|e| Error::Protocol(format!("invalid utf-8: {e}")))?
                .parse()
                .map_err(|e| Error::Protocol(format!("invalid integer: {e}"))),
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e.clone())),
            _ => Err(Error::Protocol("expected integer".into())),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Boolean(b) => Ok(*b),
            Self::Integer(i) => Ok(*i != 0),
            Self::SimpleString(s) => match s.as_str() {
                "OK" | "1" | "true" | "t" | "TRUE" => Ok(true),
                "0" | "false" | "f" | "FALSE" => Ok(false),
                _ => Err(Error::Protocol("expected boolean".into())),
            },
            Self::BlobString(b) => match b.as_slice() {
                b"OK" | b"1" | b"true" | b"t" | b"TRUE" => Ok(true),
                b"0" | b"false" | b"f" | b"FALSE" => Ok(false),
                _ => Err(Error::Protocol("expected boolean".into())),
            },
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e.clone())),
            _ => Err(Error::Protocol("expected boolean".into())),
        }
    }

    pub fn into_array(self) -> Result<Vec<Value>> {
        match self {
            Self::Array(a) | Self::Set(a) | Self::Push(a) => Ok(a),
            Self::Null => Ok(Vec::new()),
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e)),
            _ => Err(Error::Protocol("expected array".into())),
        }
    }

    pub fn into_map(self) -> Result<HashMap<String, Value>> {
        match self {
            Self::Map(pairs) => {
                let mut map = HashMap::with_capacity(pairs.len());
                for (k, v) in pairs {
                    map.insert(k.into_string()?, v);
                }
                Ok(map)
            }
            Self::Array(arr) if arr.len() % 2 == 0 => {
                let mut map = HashMap::with_capacity(arr.len() / 2);
                let mut iter = arr.into_iter();
                while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                    map.insert(k.into_string()?, v);
                }
                Ok(map)
            }
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e)),
            _ => Err(Error::Protocol("expected map".into())),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            Self::Double(d) => Ok(*d),
            Self::Integer(i) => Ok(*i as f64),
            Self::SimpleString(s) | Self::BigNumber(s) => s
                .parse()
                .map_err(|e| Error::Protocol(format!("invalid float: {e}"))),
            Self::BlobString(b) => from_utf8(b)
                .map_err(|e| Error::Protocol(format!("invalid utf-8: {e}")))?
                .parse()
                .map_err(|e| Error::Protocol(format!("invalid float: {e}"))),
            Self::Error(e) | Self::BlobError(e) => Err(Error::Redis(e.clone())),
            _ => Err(Error::Protocol("expected float".into())),
        }
    }
}

pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self>;
}

impl FromValue for Value {
    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Error(e) | Value::BlobError(e) => Err(Error::Redis(e)),
            v => Ok(v),
        }
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<Self> {
        value.into_string()
    }
}

impl FromValue for Vec<u8> {
    fn from_value(value: Value) -> Result<Self> {
        value.into_bytes()
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<Self> {
        value.as_i64()
    }
}

macro_rules! impl_from_value_int {
    ($($t:ty),*) => {
        $(
            impl FromValue for $t {
                fn from_value(value: Value) -> Result<Self> {
                    let i = value.as_i64()?;
                    <$t>::try_from(i).map_err(|e| Error::Protocol(format!("cannot convert to {}: {e}", stringify!($t))))
                }
            }
        )*
    };
}

impl_from_value_int!(i8, i16, i32, isize, u16, u32, u64, usize);

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<Self> {
        value.as_f64()
    }
}

impl FromValue for f32 {
    fn from_value(value: Value) -> Result<Self> {
        value.as_f64().map(|f| f as f32)
    }
}

impl FromValue for bool {
    fn from_value(value: Value) -> Result<Self> {
        value.as_bool()
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            Value::Array(ref arr) if arr.is_empty() => Ok(None),
            _ => T::from_value(value).map(Some),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: Value) -> Result<Self> {
        let arr = value.into_array()?;
        let mut res = Vec::with_capacity(arr.len());
        for item in arr {
            res.push(T::from_value(item)?);
        }
        Ok(res)
    }
}

impl<T: FromValue> FromValue for HashMap<String, T> {
    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Map(raw_map) => {
                let mut map = HashMap::with_capacity(raw_map.len());
                for (k, v) in raw_map {
                    map.insert(k.into_string()?, T::from_value(v)?);
                }
                Ok(map)
            }
            Value::Array(arr) => {
                let mut map = HashMap::with_capacity(arr.len() / 2);
                let mut iter = arr.into_iter();
                while let Some(k_val) = iter.next() {
                    let k = k_val.into_string()?;
                    if let Some(v_val) = iter.next() {
                        map.insert(k, T::from_value(v_val)?);
                    }
                }
                Ok(map)
            }
            other => Err(Error::Protocol(format!(
                "expected map or array, got {:?}",
                other
            ))),
        }
    }
}

impl<T1: FromValue, T2: FromValue> FromValue for (T1, T2) {
    fn from_value(value: Value) -> Result<Self> {
        let arr = value.into_array()?;
        if arr.len() < 2 {
            return Err(Error::Protocol(
                "expected array of at least 2 elements for pair".into(),
            ));
        }
        let mut iter = arr.into_iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();
        Ok((T1::from_value(v1)?, T2::from_value(v2)?))
    }
}

impl<T1: FromValue, T2: FromValue, T3: FromValue> FromValue for (T1, T2, T3) {
    fn from_value(value: Value) -> Result<Self> {
        let arr = value.into_array()?;
        if arr.len() < 3 {
            return Err(Error::Protocol(
                "expected array of at least 3 elements for tuple".into(),
            ));
        }
        let mut iter = arr.into_iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();
        let v3 = iter.next().unwrap();
        Ok((
            T1::from_value(v1)?,
            T2::from_value(v2)?,
            T3::from_value(v3)?,
        ))
    }
}

impl FromValue for () {
    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Error(e) | Value::BlobError(e) => Err(Error::Redis(e)),
            _ => Ok(()),
        }
    }
}
