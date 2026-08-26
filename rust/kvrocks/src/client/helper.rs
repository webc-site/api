use crate::{
    error::{Error, Result},
    resp3::{
        Cmd, FromValue, Value,
        constants::{AUTH, COUNT, MATCH, NOVALUES},
    },
};

pub fn build_auth_cmd(username: Option<&str>, password: &str) -> Cmd {
    Cmd::new(AUTH).arg_opt_bytes(username).arg_bytes(password)
}

#[inline]
pub fn parse_cursor(val: Option<&Value>) -> u64 {
    val.and_then(|f| f.as_str())
        .unwrap_or("0")
        .parse::<u64>()
        .unwrap_or(0)
}

pub fn apply_scan_opts(
    cmd: Cmd,
    r#match: Option<&str>,
    count: Option<usize>,
    no_values: bool,
) -> Cmd {
    cmd.arg_keyword_opt_bytes(MATCH, r#match)
        .arg_keyword_opt_int(COUNT, count)
        .arg_if(no_values, NOVALUES)
}

pub fn parse_scan_result(res: Value) -> Result<(u64, Vec<String>)> {
    let mut arr = res.into_array()?;
    let next_cursor = parse_cursor(arr.first());
    let items = if arr.len() >= 2 {
        Vec::<String>::from_value(arr.swap_remove(1))?
    } else {
        Vec::new()
    };
    Ok((next_cursor, items))
}

pub fn parse_scan_pair_result<T1: FromValue, T2: FromValue>(
    res: Value,
) -> Result<(u64, Vec<(T1, T2)>)> {
    let mut arr = res.into_array()?;
    let next_cursor = parse_cursor(arr.first());
    let pairs = if arr.len() >= 2 {
        parse_pair_array(arr.swap_remove(1))?
    } else {
        Vec::new()
    };
    Ok((next_cursor, pairs))
}

pub fn parse_pair_array<T1: FromValue, T2: FromValue>(res: Value) -> Result<Vec<(T1, T2)>> {
    match res {
        Value::Map(pairs) => {
            let mut res_pairs = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                res_pairs.push((T1::from_value(k)?, T2::from_value(v)?));
            }
            Ok(res_pairs)
        }
        Value::Array(arr) | Value::Set(arr) | Value::Push(arr) => {
            if let Some(first) = arr.first()
                && matches!(first, Value::Array(_))
            {
                let mut pairs = Vec::with_capacity(arr.len());
                for item in arr {
                    let sub = item.into_array()?;
                    if sub.len() >= 2 {
                        let mut iter = sub.into_iter();
                        let v1 = iter.next().unwrap();
                        let v2 = iter.next().unwrap();
                        pairs.push((T1::from_value(v1)?, T2::from_value(v2)?));
                    }
                }
                return Ok(pairs);
            }
            let mut pairs = Vec::with_capacity(arr.len() / 2);
            let mut iter = arr.into_iter();
            while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                pairs.push((T1::from_value(k)?, T2::from_value(v)?));
            }
            Ok(pairs)
        }
        Value::Null => Ok(Vec::new()),
        Value::Error(e) | Value::BlobError(e) => Err(Error::Redis(e)),
        _ => Err(Error::Protocol(
            "expected map or array for pair array".into(),
        )),
    }
}

pub fn parse_single_or_array<T: FromValue>(res: Value) -> Result<Vec<T>> {
    match res {
        Value::Array(_) | Value::Set(_) => Vec::<T>::from_value(res),
        Value::Null => Ok(Vec::new()),
        v => Ok(vec![T::from_value(v)?]),
    }
}
