#![allow(
    clippy::significant_drop_tightening,
    reason = "test db lifetime is the function scope"
)]

use serde_json;
use xray_tui_db::Database;
use xray_tui_db::models::{Group, Profile};

/// Helper: create an in-memory database for testing.
async fn test_db() -> Database {
    Database::in_memory().await.expect("open in-memory db")
}

fn test_group(id: &str) -> Group {
    Group {
        id: id.to_string(),
        name: Some(id.to_string()),
        subscription_url: None,
        subscription_enabled: None,
        user_agent: None,
        convert_target: None,
        core_type: None,
        sort_order: None,
        is_system: None,
    }
}

fn make_profile(id_counter: i64) -> Profile {
    let spec_blob = serde_json::to_vec(&serde_json::json!({
        "remarks": format!("test-{id_counter}"),
        "user_id": "uuid",
    }))
    .unwrap_or_default();
    Profile {
        id: id_counter,
        sig: id_counter,
        cred_hash: id_counter,
        proto_kind: "test".to_string(),
        spec_blob,
        config_type: 1,
        core_type: "xray".to_string(),
        address: "127.0.0.1".to_string(),
        port: 1080,
        transport: Some("tcp".to_string()),
        security: Some("auto".to_string()),
        created_at: 0,
        extension: Default::default(),
        server_stat: Default::default(),
    }
}

fn spec_remarks(blob: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(blob).ok()?;
    v.get("remarks")?.as_str().map(|s| s.to_string())
}

#[tokio::test]
async fn test_create_and_read_profile() {
    let db = test_db().await;
    db.insert_group(&test_group("test-group"))
        .await
        .expect("insert group");
    let p = make_profile(1);
    db.insert_profile(&p, "test-group")
        .await
        .expect("insert profile");

    let found = db
        .get_profile(1)
        .await
        .expect("get profile")
        .expect("profile should exist");

    assert_eq!(found.id, 1);
    assert_eq!(found.config_type, 1);
    assert_eq!(spec_remarks(&found.spec_blob).as_deref(), Some("test-1"));
}

#[tokio::test]
async fn test_update_profile() {
    let db = test_db().await;
    db.insert_group(&test_group("test-group"))
        .await
        .expect("insert group");
    let p = make_profile(2);
    db.insert_profile(&p, "test-group").await.expect("insert");

    let mut updated = make_profile(2);
    updated.address = "192.168.1.1".to_string();
    updated.port = 2080;
    db.update_profile(&updated).await.expect("update");

    let found = db
        .get_profile(2)
        .await
        .expect("get profile")
        .expect("profile should exist after update");

    assert_eq!(found.address, "192.168.1.1");
    assert_eq!(found.port, 2080);
}

#[tokio::test]
async fn test_delete_profile() {
    let db = test_db().await;
    db.insert_group(&test_group("test-group"))
        .await
        .expect("insert group");
    let p = make_profile(3);
    db.insert_profile(&p, "test-group").await.expect("insert");

    assert!(db.get_profile(3).await.expect("get").is_some());

    db.delete_profile(3).await.expect("delete");

    let found = db.get_profile(3).await.expect("get");
    assert!(found.is_none(), "deleted profile should not exist");
}

#[tokio::test]
async fn test_delete_group_cascade() {
    let db = test_db().await;

    db.insert_group(&test_group("test-group-1"))
        .await
        .expect("insert group");

    let p1 = make_profile(10);
    let p2 = make_profile(11);

    db.insert_profile(&p1, "test-group-1")
        .await
        .expect("insert p1");
    db.insert_profile(&p2, "test-group-1")
        .await
        .expect("insert p2");

    let profiles = db
        .get_profiles_by_group("test-group-1")
        .await
        .expect("get profiles by group");
    assert_eq!(profiles.len(), 2, "two profiles in group");

    db.delete_group("test-group-1").await.expect("delete group");

    let profiles_after = db
        .get_profiles_by_group("test-group-1")
        .await
        .expect("get profiles after delete");
    assert!(profiles_after.is_empty(), "profiles should be cleaned up");

    let all_groups = db.get_all_groups().await.expect("get all groups");
    assert!(
        !all_groups.iter().any(|g| g.id == "test-group-1"),
        "group should be deleted"
    );
}

#[tokio::test]
async fn test_concurrent_reads() {
    let db = std::sync::Arc::new(test_db().await);

    db.insert_group(&test_group("test-group"))
        .await
        .expect("insert group");

    for i in 0..5 {
        let p = make_profile(100 + i);
        db.insert_profile(&p, "test-group").await.expect("insert");
    }

    let mut handles = Vec::new();
    for _ in 0..5 {
        let db_clone = std::sync::Arc::clone(&db);
        handles.push(tokio::spawn(async move {
            db_clone.get_all_profiles().await.expect("concurrent read")
        }));
    }

    for h in handles {
        let results = h.await.expect("join");
        assert_eq!(results.len(), 5, "should see 5 profiles");
    }
}

#[tokio::test]
async fn test_multi_step_atomicity() {
    let db = test_db().await;

    db.insert_group(&test_group("test-group"))
        .await
        .expect("insert group");

    let p = make_profile(200);
    db.insert_profile(&p, "test-group")
        .await
        .expect("insert valid profile");

    // Upsert: duplicate id silently succeeds (ON CONFLICT DO UPDATE)
    let dupe = make_profile(200);
    let result = db.insert_profile(&dupe, "test-group").await;
    assert!(result.is_ok(), "upsert duplicate should succeed");

    let found = db
        .get_profile(200)
        .await
        .expect("get")
        .expect("original profile should exist");
    assert_eq!(found.id, 200);
}
