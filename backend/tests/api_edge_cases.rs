mod integration;

use integration::TestContext;
use serde_json::json;

#[tokio::test]
async fn test_wrong_passphrase_returns_401() {
    let ctx = TestContext::new().await;

    // Create a secret
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "my-secret"
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();

    // Try with wrong passphrase
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({
            "passphrase": "wrong-passphrase-here"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_not_found_returns_404() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .post(ctx.url("/api/secrets/00000000-0000-0000-0000-000000000000"))
        .json(&json!({
            "passphrase": "any-passphrase-here"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_max_views_reached_deletes_secret() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=1
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "one-time-secret",
            "max_views": 1,
            "expires_in_hours": 24
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // First retrieval succeeds
    let r1 = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 200);

    // Second retrieval fails (secret deleted)
    let r2 = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 404);
}

#[tokio::test]
async fn test_health_check() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .get(ctx.url("/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn test_extend_secret_success() {
    let ctx = TestContext::new().await;

    // Create extendable secret
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test secret",
            "max_views": 5,
            "expires_in_hours": 1,
            "extendable": true
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), 200);
    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Extend the secret
    let extend_response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}/extend", id)))
        .json(&json!({
            "passphrase": passphrase,
            "add_days": 1,
            "add_views": 5
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(extend_response.status(), 200);
    let extended: serde_json::Value = extend_response.json().await.unwrap();
    assert_eq!(extended["max_views"], 10);
}

#[tokio::test]
async fn test_extend_non_extendable_secret_returns_403() {
    let ctx = TestContext::new().await;

    // Create non-extendable secret
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test secret",
            "extendable": false
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), 200);
    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Try to extend
    let extend_response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}/extend", id)))
        .json(&json!({
            "passphrase": passphrase,
            "add_days": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(extend_response.status(), 403);
}

#[tokio::test]
async fn test_extend_wrong_passphrase_returns_401() {
    let ctx = TestContext::new().await;

    // Create secret
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test secret",
            "extendable": true
        }))
        .send()
        .await
        .unwrap();

    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();

    // Try to extend with wrong passphrase
    let extend_response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}/extend", id)))
        .json(&json!({
            "passphrase": "wrong-passphrase",
            "add_days": 1
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(extend_response.status(), 401);
}

#[tokio::test]
async fn test_retrieve_returns_extendable_info() {
    let ctx = TestContext::new().await;

    // Create extendable secret
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test secret",
            "extendable": true
        }))
        .send()
        .await
        .unwrap();

    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Retrieve and check extendable field is present
    let retrieve_response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), 200);
    let body: serde_json::Value = retrieve_response.json().await.unwrap();
    assert_eq!(body["extendable"], true);
    assert!(body["expires_at"].is_string());
}

#[tokio::test]
async fn test_wrong_passphrase_twice_no_view_consumed() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=3
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret",
            "max_views": 3
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // Two wrong attempts
    for _ in 0..2 {
        let response = ctx
            .client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong-passphrase" }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 401);
    }

    // Correct attempt should still show 3 views remaining (none consumed)
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["views_remaining"], 2); // 3 - 1 (this view) = 2
}

#[tokio::test]
async fn test_wrong_passphrase_three_times_consumes_view() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=3
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret",
            "max_views": 3
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // Three wrong attempts (3rd consumes a view)
    for _ in 0..3 {
        let response = ctx
            .client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong-passphrase" }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 401);
    }

    // Correct attempt should show only 1 view remaining (3 - 1 consumed - 1 this view = 1)
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["views_remaining"], 1);
}

#[tokio::test]
async fn test_correct_passphrase_resets_failed_attempts() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=5
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret",
            "max_views": 5
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // Two wrong attempts
    for _ in 0..2 {
        ctx.client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    // Correct attempt (resets counter, uses 1 view)
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Two more wrong attempts (counter was reset, so still free)
    for _ in 0..2 {
        ctx.client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    // Should still have 3 views remaining (5 - 1 used - 1 this view = 3)
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["views_remaining"], 3);
}

#[tokio::test]
async fn test_wrong_passphrase_deletes_secret_when_views_depleted() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=1
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret",
            "max_views": 1
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // 3 wrong attempts: 2 free + 1 that consumes the only view
    for i in 0..3 {
        let response = ctx
            .client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong" }))
            .send()
            .await
            .unwrap();

        if i < 2 {
            assert_eq!(response.status(), 401);
        } else {
            // 3rd attempt deletes the secret, returns 404
            assert_eq!(response.status(), 404);
        }
    }

    // Correct passphrase should now return 404 (secret deleted)
    let response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_unlimited_views_deleted_after_max_failed_attempts() {
    let ctx = TestContext::new().await;

    // Create secret with unlimited views (max_views: null)
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret"
            // max_views not set = unlimited
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();

    // Default max_failed_attempts is 10
    // After 10 wrong attempts, secret should be deleted
    for i in 0..10 {
        let response = ctx
            .client
            .post(ctx.url(&format!("/api/secrets/{}", id)))
            .json(&json!({ "passphrase": "wrong" }))
            .send()
            .await
            .unwrap();

        if i < 9 {
            assert_eq!(response.status(), 401, "Attempt {} should return 401", i + 1);
        } else {
            // 10th attempt deletes the secret
            assert_eq!(response.status(), 404, "Attempt 10 should return 404 (deleted)");
        }
    }
}
