use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::config::Config;
use crate::crypto::{decrypt_secret, encrypt_secret, generate_passphrase};
use crate::db::SecretRepository;
use crate::error::AppError;
use crate::models::{
    CreateSecretRequest, CreateSecretResponse, ExtendSecretRequest, ExtendSecretResponse,
    RetrieveSecretResponse, Secret,
};

pub async fn create(
    repo: &impl SecretRepository,
    config: &Config,
    request: CreateSecretRequest,
) -> Result<CreateSecretResponse, AppError> {
    let passphrase = generate_passphrase()?;
    let encrypted_data = encrypt_secret(&request.secret, &passphrase)?;

    let secret = Secret::new(
        encrypted_data,
        request.max_views,
        request.expires_in_hours,
        request.extendable,
    );

    repo.create_secret(&secret).await?;

    Ok(CreateSecretResponse {
        id: secret.id,
        passphrase,
        expires_at: secret.expires_at,
        share_url: format!("{}/secret/{}", config.base_url, secret.id),
    })
}

pub async fn retrieve(
    repo: &impl SecretRepository,
    config: &Config,
    id: Uuid,
    passphrase: &str,
) -> Result<RetrieveSecretResponse, AppError> {
    let mut secret = repo.get_secret(&id).await?.ok_or(AppError::NotFound)?;

    // Check if expired
    if secret.is_expired() {
        repo.delete_secret(&id).await?;
        return Err(AppError::NotFound);
    }

    // Check if max views reached
    if secret.is_max_views_reached() {
        repo.delete_secret(&id).await?;
        return Err(AppError::NotFound);
    }

    // Attempt decrypt
    match decrypt_secret(&secret.encrypted_data, passphrase) {
        Ok(decrypted) => {
            // Success - reset failed attempts and increment views
            secret.failed_attempts = 0;
            secret.views += 1;
            repo.update_secret(&secret).await?;

            let views_remaining = secret.max_views.map(|max| max - secret.views);

            // Delete if max views reached after this view
            if views_remaining == Some(0) {
                repo.delete_secret(&id).await?;
            }

            Ok(RetrieveSecretResponse {
                secret: decrypted,
                views_remaining,
                extendable: secret.extendable,
                expires_at: secret.expires_at,
            })
        }
        Err(_) => {
            // Failed - increment failed attempts
            secret.failed_attempts += 1;

            // After 2 free attempts, start consuming views
            if secret.failed_attempts > 2 {
                if let Some(max_views) = secret.max_views {
                    // Consume a view
                    secret.views += 1;
                    if secret.views >= max_views {
                        // Views depleted - delete secret
                        repo.delete_secret(&id).await?;
                        return Err(AppError::NotFound);
                    }
                } else {
                    // Unlimited views - check max_failed_attempts
                    if secret.failed_attempts >= config.max_failed_attempts {
                        repo.delete_secret(&id).await?;
                        return Err(AppError::NotFound);
                    }
                }
            }

            repo.update_secret(&secret).await?;
            Err(AppError::InvalidPassphrase)
        }
    }
}

pub async fn extend(
    repo: &impl SecretRepository,
    config: &Config,
    id: Uuid,
    request: ExtendSecretRequest,
) -> Result<ExtendSecretResponse, AppError> {
    let secret = repo.get_secret(&id).await?.ok_or(AppError::NotFound)?;

    // Check if expired
    if secret.is_expired() {
        repo.delete_secret(&id).await?;
        return Err(AppError::NotFound);
    }

    // Check if extendable
    if !secret.extendable {
        return Err(AppError::NotExtendable);
    }

    // Verify passphrase by attempting decryption
    decrypt_secret(&secret.encrypted_data, &request.passphrase)?;

    // Validate extension values
    if request.add_days.is_none() && request.add_views.is_none() {
        return Err(AppError::BadRequest);
    }
    if request.add_days.map(|d| d <= 0).unwrap_or(false)
        || request.add_views.map(|v| v <= 0).unwrap_or(false)
    {
        return Err(AppError::BadRequest);
    }

    // Calculate new expiration
    let new_expires_at = if let Some(days) = request.add_days {
        let proposed = secret.expires_at + Duration::days(days as i64);
        let max_allowed = Utc::now() + Duration::days(config.max_secret_days as i64);
        if proposed > max_allowed {
            return Err(AppError::ExceedsLimits);
        }
        proposed
    } else {
        secret.expires_at
    };

    // Calculate new max views
    let new_max_views = if let Some(add) = request.add_views {
        let current_max = secret.max_views.unwrap_or(0);
        let proposed = current_max + add;
        if proposed > config.max_secret_views {
            return Err(AppError::ExceedsLimits);
        }
        Some(proposed)
    } else {
        secret.max_views
    };

    // Update database
    repo.extend_secret(&id, new_expires_at, new_max_views).await?;

    Ok(ExtendSecretResponse {
        expires_at: new_expires_at,
        max_views: new_max_views,
        views: secret.views,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_secret;
    use crate::db::MockSecretRepository;
    use chrono::{Duration, Utc};

    fn test_config() -> Config {
        Config {
            database: crate::config::DatabaseConfig::Postgres {
                url: String::new(),
            },
            base_url: "https://example.com".to_string(),
            port: 3000,
            max_secret_days: 30,
            max_secret_views: 100,
            max_failed_attempts: 10,
        }
    }

    // ==================== CREATE TESTS ====================

    #[tokio::test]
    async fn test_create_returns_share_url_and_passphrase() {
        let mut mock = MockSecretRepository::new();
        mock.expect_create_secret().returning(|_| Ok(()));

        let request = CreateSecretRequest {
            secret: "my secret".into(),
            max_views: Some(5),
            expires_in_hours: None,
            extendable: true,
        };

        let result = create(&mock, &test_config(), request).await.unwrap();

        assert!(result.share_url.starts_with("https://example.com/secret/"));
        assert_eq!(result.passphrase.split('-').count(), 3);
    }

    #[tokio::test]
    async fn test_create_stores_encrypted_secret() {
        let mut mock = MockSecretRepository::new();
        mock.expect_create_secret()
            .withf(|secret| !secret.encrypted_data.is_empty() && secret.max_views == Some(10))
            .returning(|_| Ok(()));

        let request = CreateSecretRequest {
            secret: "sensitive data".into(),
            max_views: Some(10),
            expires_in_hours: Some(48),
            extendable: false,
        };

        let result = create(&mock, &test_config(), request).await;
        assert!(result.is_ok());
    }

    // ==================== RETRIEVE TESTS ====================

    #[tokio::test]
    async fn test_retrieve_success_increments_views() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_update_secret()
            .withf(|s| s.views == 1 && s.failed_attempts == 0)
            .returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, passphrase)
            .await
            .unwrap();

        assert_eq!(result.secret, "my secret");
        assert_eq!(result.views_remaining, Some(4));
    }

    #[tokio::test]
    async fn test_retrieve_deletes_on_last_view() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(1),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_update_secret().returning(|_| Ok(()));
        mock.expect_delete_secret().returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, passphrase)
            .await
            .unwrap();

        assert_eq!(result.views_remaining, Some(0));
    }

    #[tokio::test]
    async fn test_retrieve_not_found() {
        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret().returning(|_| Ok(None));

        let result = retrieve(&mock, &test_config(), Uuid::new_v4(), "any").await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn test_retrieve_expired_secret_is_deleted() {
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: "data".into(),
            created_at: Utc::now() - Duration::hours(48),
            expires_at: Utc::now() - Duration::hours(1), // Expired
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_delete_secret().returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, "any").await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn test_retrieve_wrong_passphrase_increments_failed_attempts() {
        let encrypted = encrypt_secret("my secret", "correct passphrase").unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_update_secret()
            .withf(|s| s.failed_attempts == 1 && s.views == 0)
            .returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, "wrong passphrase").await;

        assert!(matches!(result, Err(AppError::InvalidPassphrase)));
    }

    #[tokio::test]
    async fn test_retrieve_third_wrong_attempt_consumes_view() {
        let encrypted = encrypt_secret("my secret", "correct passphrase").unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 2, // Already had 2 free attempts
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_update_secret()
            .withf(|s| s.failed_attempts == 3 && s.views == 1)
            .returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, "wrong passphrase").await;

        assert!(matches!(result, Err(AppError::InvalidPassphrase)));
    }

    #[tokio::test]
    async fn test_retrieve_views_depleted_deletes_secret() {
        let encrypted = encrypt_secret("my secret", "correct passphrase").unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(3),
            views: 2, // One more wrong attempt will deplete
            extendable: true,
            failed_attempts: 2,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_delete_secret().returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, "wrong passphrase").await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn test_retrieve_unlimited_views_max_failed_attempts() {
        let encrypted = encrypt_secret("my secret", "correct passphrase").unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: None, // Unlimited views
            views: 0,
            extendable: true,
            failed_attempts: 9, // One more will hit max_failed_attempts (10)
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_delete_secret().returning(|_| Ok(()));

        let result = retrieve(&mock, &test_config(), secret_id, "wrong passphrase").await;

        assert!(matches!(result, Err(AppError::NotFound)));
    }

    // ==================== EXTEND TESTS ====================

    #[tokio::test]
    async fn test_extend_adds_days() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let expires_at = Utc::now() + Duration::hours(24);
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at,
            max_views: Some(5),
            views: 1,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_extend_secret().returning(|_, _, _| Ok(()));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: Some(7),
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request)
            .await
            .unwrap();

        assert!(result.expires_at > expires_at);
    }

    #[tokio::test]
    async fn test_extend_adds_views() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 1,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));
        mock.expect_extend_secret().returning(|_, _, _| Ok(()));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: None,
            add_views: Some(10),
        };

        let result = extend(&mock, &test_config(), secret_id, request)
            .await
            .unwrap();

        assert_eq!(result.max_views, Some(15));
    }

    #[tokio::test]
    async fn test_extend_not_extendable_returns_error() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: false, // Not extendable
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: Some(7),
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::NotExtendable)));
    }

    #[tokio::test]
    async fn test_extend_wrong_passphrase_returns_error() {
        let encrypted = encrypt_secret("my secret", "correct passphrase").unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: "wrong passphrase".to_string(),
            add_days: Some(7),
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::InvalidPassphrase)));
    }

    #[tokio::test]
    async fn test_extend_exceeds_max_days_returns_error() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(25), // Already 25 days out
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: Some(10), // Would exceed 30 day max
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::ExceedsLimits)));
    }

    #[tokio::test]
    async fn test_extend_exceeds_max_views_returns_error() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(95), // Already at 95
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: None,
            add_views: Some(10), // Would exceed 100 max
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::ExceedsLimits)));
    }

    #[tokio::test]
    async fn test_extend_no_values_returns_bad_request() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: None,
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::BadRequest)));
    }

    #[tokio::test]
    async fn test_extend_negative_values_returns_bad_request() {
        let passphrase = "word1 word2 word3";
        let encrypted = encrypt_secret("my secret", passphrase).unwrap();
        let secret = Secret {
            id: Uuid::new_v4(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            max_views: Some(5),
            views: 0,
            extendable: true,
            failed_attempts: 0,
        };
        let secret_id = secret.id;

        let mut mock = MockSecretRepository::new();
        mock.expect_get_secret()
            .returning(move |_| Ok(Some(secret.clone())));

        let request = ExtendSecretRequest {
            passphrase: passphrase.to_string(),
            add_days: Some(-1),
            add_views: None,
        };

        let result = extend(&mock, &test_config(), secret_id, request).await;

        assert!(matches!(result, Err(AppError::BadRequest)));
    }
}
