mod integration;

use integration::TestContext;
use serde_json::json;

#[tokio::test]
async fn test_create_secret_returns_valid_response() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "my-api-key-12345",
            "max_views": 5,
            "expires_in_hours": 24
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["id"].is_string());
    assert!(body["passphrase"].is_string());
    assert!(body["share_url"].is_string());
    assert!(body["expires_at"].is_string());

    // Verify passphrase is 3 words separated by dashes
    let passphrase = body["passphrase"].as_str().unwrap();
    let word_count = passphrase.split('-').count();
    assert_eq!(word_count, 3, "Passphrase should be 3 words: {}", passphrase);
}

#[tokio::test]
async fn test_create_secret_with_extendable_true() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "extendable-secret",
            "max_views": 5,
            "expires_in_hours": 24,
            "extendable": true
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_secret_with_extendable_false() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "non-extendable-secret",
            "extendable": false
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn test_create_secret_defaults_extendable_true() {
    let ctx = TestContext::new().await;

    // Create without specifying extendable
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 200);
    let created: serde_json::Value = create_response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let passphrase = created["passphrase"].as_str().unwrap();

    // Retrieve and verify extendable defaults to true
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
}
