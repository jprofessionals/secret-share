mod integration;

use integration::TestContext;
use serde_json::json;

#[tokio::test]
async fn test_retrieve_secret_success() {
    let ctx = TestContext::new().await;

    // Create a secret first
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "test-secret-value",
            "max_views": 5,
            "expires_in_hours": 24,
            "require_2fa": false
        }))
        .send()
        .await
        .expect("Failed to create secret");

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // Retrieve the secret
    let retrieve_response = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({
            "passphrase": passphrase,
            "verified_2fa": false
        }))
        .send()
        .await
        .expect("Failed to retrieve secret");

    assert_eq!(retrieve_response.status(), 200);

    let retrieve_body: serde_json::Value = retrieve_response.json().await.unwrap();
    assert_eq!(retrieve_body["secret"], "test-secret-value");
    assert_eq!(retrieve_body["views_remaining"], 4);
}

#[tokio::test]
async fn test_retrieve_decrements_view_count() {
    let ctx = TestContext::new().await;

    // Create secret with max_views=3
    let create_response = ctx
        .client
        .post(ctx.url("/api/secrets"))
        .json(&json!({
            "secret": "counting-views",
            "max_views": 3,
            "expires_in_hours": 24,
            "require_2fa": false
        }))
        .send()
        .await
        .unwrap();

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let id = create_body["id"].as_str().unwrap();
    let passphrase = create_body["passphrase"].as_str().unwrap();

    // First retrieval: views_remaining should be 2
    let r1 = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase, "verified_2fa": false }))
        .send()
        .await
        .unwrap();
    let b1: serde_json::Value = r1.json().await.unwrap();
    assert_eq!(b1["views_remaining"], 2);

    // Second retrieval: views_remaining should be 1
    let r2 = ctx
        .client
        .post(ctx.url(&format!("/api/secrets/{}", id)))
        .json(&json!({ "passphrase": passphrase, "verified_2fa": false }))
        .send()
        .await
        .unwrap();
    let b2: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(b2["views_remaining"], 1);
}
