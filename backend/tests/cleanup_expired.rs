mod integration;

use chrono::{Duration, Utc};
use integration::helpers::TestContext;
use secret_share_backend::db::SecretRepository;
use secret_share_backend::models::Secret;
use uuid::Uuid;

#[tokio::test]
async fn test_cleanup_expired_removes_only_expired_secrets() {
    let ctx = TestContext::new().await;

    // Create an expired secret (expired 1 hour ago)
    let expired_secret = Secret {
        id: Uuid::new_v4(),
        encrypted_data: "expired_data".to_string(),
        created_at: Utc::now() - Duration::hours(25),
        expires_at: Utc::now() - Duration::hours(1),
        max_views: None,
        views: 0,
        extendable: true,
        failed_attempts: 0,
    };

    // Create a valid secret (expires in 1 hour)
    let valid_secret = Secret {
        id: Uuid::new_v4(),
        encrypted_data: "valid_data".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        max_views: None,
        views: 0,
        extendable: true,
        failed_attempts: 0,
    };

    ctx.db.create_secret(&expired_secret).await.unwrap();
    ctx.db.create_secret(&valid_secret).await.unwrap();

    // Run cleanup
    let deleted = ctx.db.cleanup_expired().await.unwrap();

    // Verify only expired secret was deleted
    assert_eq!(deleted, 1);
    assert!(ctx.db.get_secret(&expired_secret.id).await.unwrap().is_none());
    assert!(ctx.db.get_secret(&valid_secret.id).await.unwrap().is_some());
}

#[tokio::test]
async fn test_cleanup_expired_returns_zero_when_no_expired() {
    let ctx = TestContext::new().await;

    // Create only valid secrets
    let valid_secret = Secret {
        id: Uuid::new_v4(),
        encrypted_data: "valid_data".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1),
        max_views: None,
        views: 0,
        extendable: true,
        failed_attempts: 0,
    };

    ctx.db.create_secret(&valid_secret).await.unwrap();

    let deleted = ctx.db.cleanup_expired().await.unwrap();

    assert_eq!(deleted, 0);
    assert!(ctx.db.get_secret(&valid_secret.id).await.unwrap().is_some());
}
