mod common;
use common::get_client;
use kvrocks::client::{GeoRadius, GeoSearch, GeoSearchStore};

#[tokio::test]
async fn test_all_geo_commands() -> aok::Void {
    let client = get_client().await?;

    let key = "test_geo_all_k";
    let key_dst = "test_geo_all_dst";
    let _ = client.mdel(&[key, key_dst]).await;

    // 1. geoadd, geodist, geohash, geopos
    let added = client
        .geoadd(
            key,
            &[
                (13.361389, 38.115556, "Palermo"),
                (15.087269, 37.502669, "Catania"),
                (12.496366, 41.902782, "Rome"),
            ],
        )
        .await?;
    assert_eq!(added, 3);

    let dist_m = client.geodist(key, "Palermo", "Catania", Some("m")).await?;
    assert!(dist_m.is_some() && dist_m.unwrap() > 100000.0);

    let dist_km = client
        .geodist(key, "Palermo", "Catania", Some("km"))
        .await?;
    assert!(dist_km.is_some() && dist_km.unwrap() > 100.0);

    let dist_none = client.geodist(key, "Palermo", "NonExistent", None).await?;
    assert!(dist_none.is_none());

    let hashes: Vec<Option<String>> = client.geohash(key, &["Palermo", "Catania", "None"]).await?;
    assert_eq!(hashes.len(), 3);
    assert!(hashes[0].is_some());
    assert!(hashes[1].is_some());
    assert!(hashes[2].is_none());

    let positions: Vec<Option<(f64, f64)>> = client.geopos(key, &["Palermo", "None"]).await?;
    assert_eq!(positions.len(), 2);
    assert!(positions[0].is_some());
    assert!(positions[1].is_none());

    // 2. georadius, georadiusbymember, georadius_ro, georadiusbymember_ro
    let _ = client
        .georadius(
            key,
            15.0,
            37.0,
            200.0,
            "km",
            &[GeoRadius::WithCoord, GeoRadius::WithDist, GeoRadius::Asc],
        )
        .await;
    let _ = client
        .georadiusbymember(
            key,
            "Palermo",
            200.0,
            "km",
            &[GeoRadius::WithHash, GeoRadius::Desc, GeoRadius::Count(5)],
        )
        .await;
    let _ = client
        .georadius_ro(key, 15.0, 37.0, 200.0, "km", &[GeoRadius::WithDist])
        .await;
    let _ = client
        .georadiusbymember_ro(key, "Palermo", 200.0, "km", &[GeoRadius::WithCoord])
        .await;

    // 3. geosearch, geosearchstore (FromMember / FromLonLat / ByRadius / ByBox)
    let _ = client
        .geosearch(
            key,
            &[
                GeoSearch::FromMember("Palermo"),
                GeoSearch::ByRadius(100.0, "km"),
                GeoSearch::Asc,
                GeoSearch::WithCoord,
                GeoSearch::WithDist,
                GeoSearch::WithHash,
                GeoSearch::Count(2),
            ],
        )
        .await;
    let _ = client
        .geosearch(
            key,
            &[
                GeoSearch::FromLonLat(13.361389, 38.115556),
                GeoSearch::ByBox(200.0, 200.0, "km"),
                GeoSearch::Desc,
            ],
        )
        .await;
    let _ = client
        .geosearchstore(
            key_dst,
            key,
            &[
                GeoSearchStore::FromMember("Palermo"),
                GeoSearchStore::ByRadius(100.0, "km"),
                GeoSearchStore::StoreDist,
            ],
        )
        .await;

    let _ = client.mdel(&[key, key_dst]).await;

    aok::OK
}
