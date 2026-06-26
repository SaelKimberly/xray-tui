use xray_tui_db::Database;
use xray_tui_db::models::{Group, Profile};

/// Helper: create an in-memory database for testing.
async fn test_db() -> Database {
    Database::open_in_memory()
        .await
        .expect("open in-memory db")
}

fn make_profile(id: &str, sub_uid: i64) -> Profile {
    Profile {
        id: id.to_string(),
        config_type: 1,
        core_type: "xray".to_string(),
        remarks: Some(format!("test-{id}")),
        address: Some("127.0.0.1".to_string()),
        port: Some(1080),
        user_id: Some("uuid".to_string()),
        security: Some("auto".to_string()),
        network: Some("tcp".to_string()),
        stream_settings: None,
        protocol_settings: None,
        is_sub: Some(0),
        sub_id: None,
        group_id: None,
        sort_order: Some(0),
        is_active: Some(0),
        created_at: None,
        updated_at: None,
        sub_uid: Some(sub_uid),
    }
}

#[tokio::test]
async fn test_create_and_read_profile() {
    let db = test_db().await;
    let p = make_profile("prof-1", 42);
    db.insert_profile(&p).await.expect("insert profile");

    let found = db
        .get_profile("prof-1")
        .await
        .expect("get profile")
        .expect("profile should exist");

    assert_eq!(found.id, "prof-1");
    assert_eq!(found.config_type, 1);
    assert_eq!(found.remarks.as_deref(), Some("test-prof-1"));
    assert_eq!(found.sub_uid, Some(42));
}

#[tokio::test]
async fn test_update_profile() {
    let db = test_db().await;
    let p = make_profile("prof-up", 43);
    db.insert_profile(&p).await.expect("insert");

    let mut updated = p.clone();
    updated.remarks = Some("updated-remark".to_string());
    updated.port = Some(2080);
    db.update_profile(&updated).await.expect("update");

    let found = db
        .get_profile("prof-up")
        .await
        .expect("get profile")
        .expect("profile should exist after update");

    assert_eq!(found.remarks.as_deref(), Some("updated-remark"));
    assert_eq!(found.port, Some(2080));
}

#[tokio::test]
async fn test_delete_profile() {
    let db = test_db().await;
    let p = make_profile("prof-del", 44);
    db.insert_profile(&p).await.expect("insert");

    assert!(db.get_profile("prof-del").await.expect("get").is_some());

    db.delete_profile("prof-del").await.expect("delete");

    let found = db.get_profile("prof-del").await.expect("get");
    assert!(found.is_none(), "deleted profile should not exist");
}

#[tokio::test]
async fn test_delete_group_cascade() {
    let db = test_db().await;

    let group = Group {
        id: "test-group-1".to_string(),
        name: Some("test-group".to_string()),
        subscription_url: None,
        subscription_enabled: Some(0),
        user_agent: None,
        convert_target: None,
        core_type: None,
        sort_order: Some(0),
        is_system: Some(0),
    };
    db.insert_group(&group).await.expect("insert group");

    let mut p1 = make_profile("prof-g1", 50);
    p1.group_id = Some("test-group-1".to_string());
    let mut p2 = make_profile("prof-g2", 51);
    p2.group_id = Some("test-group-1".to_string());

    db.insert_profile(&p1).await.expect("insert p1");
    db.insert_profile(&p2).await.expect("insert p2");

    let profiles = db
        .get_profiles_by_group("test-group-1")
        .await
        .expect("get profiles by group");
    assert_eq!(profiles.len(), 2);

    db.delete_group("test-group-1")
        .await
        .expect("delete group");

    let profiles_after = db
        .get_profiles_by_group("test-group-1")
        .await
        .expect("get profiles after delete");
    assert!(profiles_after.is_empty(), "profiles should be cascaded");

    let all_groups = db.get_all_groups().await.expect("get all groups");
    assert!(
        !all_groups.iter().any(|g| g.id == "test-group-1"),
        "group should be deleted"
    );
}

#[tokio::test]
async fn test_concurrent_reads() {
    let db = std::sync::Arc::new(test_db().await);

    for i in 0..5 {
        let p = make_profile(&format!("concurrent-{i}"), 100 + i);
        db.insert_profile(&p).await.expect("insert");
    }

    let mut handles = Vec::new();
    for _ in 0..5 {
        let db_clone = std::sync::Arc::clone(&db);
        handles.push(tokio::spawn(async move {
            let profiles = db_clone
                .get_all_profiles()
                .await
                .expect("concurrent read");
            profiles
        }));
    }

    for h in handles {
        let result = h.await.expect("join");
        assert_eq!(result.len(), 5);
    }
}

#[tokio::test]
async fn test_multi_step_atomicity() {
    let db = test_db().await;

    let p = make_profile("atomic-1", 200);
    db.insert_profile(&p).await.expect("insert valid profile");

    let bad = make_profile("atomic-2", 0);
    let err = db.insert_profile(&bad).await;
    assert!(
        err.is_err(),
        "insert with sub_uid=0 should return error"
    );

    let found = db
        .get_profile("atomic-1")
        .await
        .expect("get")
        .expect("original profile should exist");
    assert_eq!(found.sub_uid, Some(200));

    let not_found = db.get_profile("atomic-2").await.expect("get");
    assert!(not_found.is_none(), "bad profile should not be present");
}
