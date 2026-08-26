use std::str::from_utf8;

use bytes::BytesMut;
use kvrocks::{Cmd, Decoder, Value};

#[test]
fn test_encoder() -> aok::Void {
    let cmd = Cmd::new("SET").arg("key").arg("val");
    let bytes = cmd.to_bytes();
    assert_eq!(
        from_utf8(&bytes)?,
        "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$3\r\nval\r\n"
    );
    aok::OK
}

#[test]
fn test_decoder_primitives() -> aok::Void {
    // Simple string
    let mut buf = BytesMut::from("+OK\r\n");
    assert_eq!(
        Decoder::decode(&mut buf)?,
        Some(Value::SimpleString("OK".into()))
    );
    assert!(buf.is_empty());

    // Error
    let mut buf = BytesMut::from("-ERR unknown command\r\n");
    assert_eq!(
        Decoder::decode(&mut buf)?,
        Some(Value::Error("ERR unknown command".into()))
    );

    // Integer
    let mut buf = BytesMut::from(":1000\r\n");
    assert_eq!(Decoder::decode(&mut buf)?, Some(Value::Integer(1000)));

    // Null
    let mut buf = BytesMut::from("_\r\n");
    assert_eq!(Decoder::decode(&mut buf)?, Some(Value::Null));

    // Double
    let mut buf = BytesMut::from(",1.23\r\n");
    assert_eq!(Decoder::decode(&mut buf)?, Some(Value::Double(1.23)));

    // Boolean
    let mut buf = BytesMut::from("#t\r\n#f\r\n");
    assert_eq!(Decoder::decode(&mut buf)?, Some(Value::Boolean(true)));
    assert_eq!(Decoder::decode(&mut buf)?, Some(Value::Boolean(false)));
    assert!(buf.is_empty());

    // Blob string
    let mut buf = BytesMut::from("$6\r\nfoobar\r\n");
    assert_eq!(
        Decoder::decode(&mut buf)?,
        Some(Value::BlobString(b"foobar".to_vec()))
    );
    aok::OK
}

#[test]
fn test_decoder_aggregates() -> aok::Void {
    // Array
    let mut buf = BytesMut::from("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    let val = Decoder::decode(&mut buf)?.unwrap();
    assert_eq!(
        val,
        Value::Array(vec![
            Value::BlobString(b"foo".to_vec()),
            Value::BlobString(b"bar".to_vec())
        ])
    );

    // Set
    let mut buf = BytesMut::from("~1\r\n:42\r\n");
    let val = Decoder::decode(&mut buf)?.unwrap();
    assert_eq!(val, Value::Set(vec![Value::Integer(42)]));

    // Map
    let mut buf = BytesMut::from("%1\r\n+first\r\n:1\r\n");
    let val = Decoder::decode(&mut buf)?.unwrap();
    assert_eq!(
        val,
        Value::Map(vec![(
            Value::SimpleString("first".into()),
            Value::Integer(1)
        )])
    );
    aok::OK
}

#[test]
fn test_decoder_partial_and_attribute() -> aok::Void {
    // 分段到达
    let mut buf = BytesMut::from("$5\r\nhel");
    assert_eq!(Decoder::decode(&mut buf)?, None);
    buf.extend_from_slice(b"lo\r\n");
    assert_eq!(
        Decoder::decode(&mut buf)?,
        Some(Value::BlobString(b"hello".to_vec()))
    );

    // Attribute |1\r\n+ttl\r\n:3600\r\n+OK\r\n
    let mut buf = BytesMut::from("|1\r\n+ttl\r\n:3600\r\n+OK\r\n");
    assert_eq!(
        Decoder::decode(&mut buf)?,
        Some(Value::SimpleString("OK".into()))
    );
    aok::OK
}
