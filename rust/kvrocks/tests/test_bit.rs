mod common;
use common::get_client;
use kvrocks::client::{BitIndexUnit, Bitfield};

#[tokio::test]
async fn test_all_bit_commands() -> aok::Void {
    let client = get_client().await?;

    let k1 = "test_bit_all_1";
    let k2 = "test_bit_all_2";
    let dst = "test_bit_all_dst";
    let _ = client.mdel(&[k1, k2, dst]).await;

    // 1. setbit, getbit, bitcount, bitpos (with unit)
    assert_eq!(client.setbit(k1, 7, 1).await?, 0);
    assert_eq!(client.getbit(k1, 7).await?, 1);
    assert_eq!(client.getbit(k1, 0).await?, 0);
    assert_eq!(client.bitcount(k1, None, None).await?, 1);
    assert_eq!(
        client
            .bitcount_opt(k1, Some(0), Some(0), Some(BitIndexUnit::Byte))
            .await?,
        1
    );
    assert_eq!(client.bitpos(k1, 1, None, None).await?, 7);
    assert_eq!(
        client
            .bitpos_opt(k1, 1, Some(0), Some(0), Some(BitIndexUnit::Byte))
            .await?,
        7
    );

    // 2. bitop (AND, OR, XOR, NOT)
    assert_eq!(client.setbit(k2, 7, 1).await?, 0);
    assert_eq!(client.bitop("AND", dst, &[k1, k2]).await?, 1);
    assert_eq!(client.bitop("OR", dst, &[k1, k2]).await?, 1);
    assert_eq!(client.bitop("XOR", dst, &[k1, k2]).await?, 1);
    assert_eq!(client.bitop("NOT", dst, &[k1]).await?, 1);

    // 3. bitfield, bitfield_ro (GET, SET, INCRBY, OVERFLOW)
    let bf_res: Vec<Option<i64>> = client
        .bitfield(
            k1,
            &[
                Bitfield::Get("u8", "0"),
                Bitfield::Set("u8", "0", 2),
                Bitfield::IncrBy("u8", "0", 1),
                Bitfield::Overflow("SAT"),
            ],
        )
        .await?;
    assert_eq!(bf_res.len(), 3);

    let bf_ro_res: Vec<Option<i64>> = client.bitfield_ro(k1, &[Bitfield::Get("u8", "0")]).await?;
    assert_eq!(bf_ro_res.len(), 1);

    let _ = client.mdel(&[k1, k2, dst]).await;

    aok::OK
}
