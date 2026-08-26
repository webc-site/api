use std::borrow::Cow;

use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    pub name: Cow<'static, str>,
    pub args: Vec<Vec<u8>>,
}

#[inline]
fn write_marker_len(dst: &mut BytesMut, marker: u8, len: usize) {
    let mut buf = itoa::Buffer::new();
    let s = buf.format(len);
    dst.reserve(1 + s.len() + 2);
    dst.put_u8(marker);
    dst.put_slice(s.as_bytes());
    dst.put_slice(b"\r\n");
}

impl Cmd {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<Vec<u8>>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn arg_bytes(mut self, arg: impl AsRef<[u8]>) -> Self {
        self.args.push(arg.as_ref().to_vec());
        self
    }

    pub fn arg_int(mut self, n: impl itoa::Integer) -> Self {
        let mut buf = itoa::Buffer::new();
        self.args.push(buf.format(n).as_bytes().to_vec());
        self
    }

    pub fn arg_float(mut self, f: f64) -> Self {
        let mut buf = ryu::Buffer::new();
        self.args.push(buf.format(f).as_bytes().to_vec());
        self
    }

    pub fn arg_opt(mut self, opt: Option<impl Into<Vec<u8>>>) -> Self {
        if let Some(arg) = opt {
            self.args.push(arg.into());
        }
        self
    }

    pub fn arg_opt_bytes(mut self, opt: Option<impl AsRef<[u8]>>) -> Self {
        if let Some(arg) = opt {
            self.args.push(arg.as_ref().to_vec());
        }
        self
    }

    pub fn arg_opt_int(mut self, opt: Option<impl itoa::Integer>) -> Self {
        if let Some(n) = opt {
            let mut buf = itoa::Buffer::new();
            self.args.push(buf.format(n).as_bytes().to_vec());
        }
        self
    }

    pub fn arg_opt_float(mut self, opt: Option<f64>) -> Self {
        if let Some(f) = opt {
            let mut buf = ryu::Buffer::new();
            self.args.push(buf.format(f).as_bytes().to_vec());
        }
        self
    }

    pub fn arg_if(mut self, cond: bool, arg: impl Into<Vec<u8>>) -> Self {
        if cond {
            self.args.push(arg.into());
        }
        self
    }

    pub fn arg_keyword_int(mut self, kw: impl Into<Vec<u8>>, val: impl itoa::Integer) -> Self {
        let mut buf = itoa::Buffer::new();
        self.args.push(kw.into());
        self.args.push(buf.format(val).as_bytes().to_vec());
        self
    }

    pub fn arg_keyword_opt_int(
        mut self,
        kw: impl Into<Vec<u8>>,
        opt: Option<impl itoa::Integer>,
    ) -> Self {
        if let Some(val) = opt {
            let mut buf = itoa::Buffer::new();
            self.args.push(kw.into());
            self.args.push(buf.format(val).as_bytes().to_vec());
        }
        self
    }

    pub fn arg_keyword_opt_bytes(
        mut self,
        kw: impl Into<Vec<u8>>,
        opt: Option<impl AsRef<[u8]>>,
    ) -> Self {
        if let Some(arg) = opt {
            self.args.push(kw.into());
            self.args.push(arg.as_ref().to_vec());
        }
        self
    }

    pub fn args<I, T>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Vec<u8>>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn args_slice<T: AsRef<[u8]>>(mut self, slice: &[T]) -> Self {
        self.args.reserve(slice.len());
        for item in slice {
            self.args.push(item.as_ref().to_vec());
        }
        self
    }

    pub fn args_pairs<K: AsRef<[u8]>, V: AsRef<[u8]>>(mut self, pairs: &[(K, V)]) -> Self {
        self.args.reserve(pairs.len() * 2);
        for (k, v) in pairs {
            self.args.push(k.as_ref().to_vec());
            self.args.push(v.as_ref().to_vec());
        }
        self
    }

    pub fn args_ints<T: itoa::Integer + Copy>(mut self, ints: &[T]) -> Self {
        self.args.reserve(ints.len());
        let mut buf = itoa::Buffer::new();
        for &n in ints {
            self.args.push(buf.format(n).as_bytes().to_vec());
        }
        self
    }

    pub fn args_floats(mut self, floats: &[f64]) -> Self {
        self.args.reserve(floats.len());
        let mut buf = ryu::Buffer::new();
        for &f in floats {
            self.args.push(buf.format(f).as_bytes().to_vec());
        }
        self
    }

    pub fn first_key(&self) -> Option<&[u8]> {
        let name = self.name.as_ref();
        if name.eq_ignore_ascii_case("PING")
            || name.eq_ignore_ascii_case("HELLO")
            || name.eq_ignore_ascii_case("AUTH")
            || name.eq_ignore_ascii_case("SELECT")
            || name.eq_ignore_ascii_case("QUIT")
            || name.eq_ignore_ascii_case("SENTINEL")
            || name.eq_ignore_ascii_case("CLUSTER")
            || name.eq_ignore_ascii_case("INFO")
            || name.eq_ignore_ascii_case("TIME")
            || name.eq_ignore_ascii_case("CLIENT")
            || name.eq_ignore_ascii_case("CONFIG")
            || name.eq_ignore_ascii_case("DBSIZE")
            || name.eq_ignore_ascii_case("FLUSHALL")
            || name.eq_ignore_ascii_case("FLUSHDB")
            || name.eq_ignore_ascii_case("COMMAND")
            || name.eq_ignore_ascii_case("RESET")
        {
            return None;
        }

        if name.eq_ignore_ascii_case("EVAL")
            || name.eq_ignore_ascii_case("EVALSHA")
            || name.eq_ignore_ascii_case("FCALL")
            || name.eq_ignore_ascii_case("FCALL_RO")
        {
            if self.args.len() >= 3
                && let Some(b) = self.args[1].first()
                && *b > b'0'
                && *b <= b'9'
            {
                return Some(&self.args[2]);
            }
            return None;
        }

        self.args.first().map(|v| v.as_slice())
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        let count = 1 + self.args.len();
        let name_bytes = self.name.as_bytes();
        let total_est =
            16 + name_bytes.len() + self.args.iter().map(|a| a.len() + 16).sum::<usize>();
        dst.reserve(total_est);

        write_marker_len(dst, b'*', count);
        write_marker_len(dst, b'$', name_bytes.len());
        dst.put_slice(name_bytes);
        dst.put_slice(b"\r\n");

        for arg in &self.args {
            write_marker_len(dst, b'$', arg.len());
            dst.put_slice(arg);
            dst.put_slice(b"\r\n");
        }
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(128);
        self.encode(&mut buf);
        buf
    }
}
