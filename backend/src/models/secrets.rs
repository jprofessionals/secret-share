use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Secret {
    pub id: Uuid,
    pub encrypted_data: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_views: Option<i32>,
    pub views: i32,
    pub extendable: bool,
    pub failed_attempts: i32,
}

impl Secret {
    pub fn new(
        encrypted_data: String,
        max_views: Option<i32>,
        expires_in_hours: Option<i32>,
        extendable: bool,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(expires_in_hours.unwrap_or(24) as i64);

        Self {
            id: Uuid::new_v4(),
            encrypted_data,
            created_at: now,
            expires_at,
            max_views,
            views: 0,
            extendable,
            failed_attempts: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_max_views_reached(&self) -> bool {
        if let Some(max) = self.max_views {
            self.views >= max
        } else {
            false
        }
    }
}

fn default_extendable() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSecretRequest {
    /// The secret content (already encrypted client-side)
    pub secret: String,
    /// Maximum number of views allowed (optional)
    pub max_views: Option<i32>,
    /// Hours until expiration (default: 24)
    pub expires_in_hours: Option<i32>,
    /// Whether the secret can be extended (default: true)
    #[serde(default = "default_extendable")]
    pub extendable: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateSecretResponse {
    /// Unique identifier for the secret
    pub id: Uuid,
    /// Passphrase needed to decrypt (share this securely)
    pub passphrase: String,
    /// When the secret expires
    pub expires_at: DateTime<Utc>,
    /// Full URL to share with recipient
    pub share_url: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RetrieveSecretRequest {
    /// The passphrase to decrypt the secret
    pub passphrase: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RetrieveSecretResponse {
    /// The decrypted secret content
    pub secret: String,
    /// Number of views remaining (if max_views was set)
    pub views_remaining: Option<i32>,
    /// Whether the secret can be extended
    pub extendable: bool,
    /// When the secret expires
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExtendSecretRequest {
    /// The passphrase to verify access
    pub passphrase: String,
    /// Days to add to expiration (optional)
    pub add_days: Option<i32>,
    /// Views to add to max_views (optional)
    pub add_views: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtendSecretResponse {
    /// New expiration time
    pub expires_at: DateTime<Utc>,
    /// New maximum views (null if unlimited)
    pub max_views: Option<i32>,
    /// Current view count
    pub views: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_secret_new_defaults() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            None,
            None,
            true,
        );

        assert!(!secret.id.is_nil());
        assert_eq!(secret.encrypted_data, "encrypted_data");
        assert_eq!(secret.views, 0);
        assert_eq!(secret.max_views, None);
        assert!(secret.extendable);
        assert_eq!(secret.failed_attempts, 0);
    }

    #[test]
    fn test_secret_new_with_options() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            Some(5),
            Some(48),
            false,
        );

        assert_eq!(secret.max_views, Some(5));
        assert!(!secret.extendable);
    }

    #[test]
    fn test_secret_expiration_calculation() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            None,
            Some(24),
            true,
        );

        // Expires at should be approximately 24 hours from created_at
        let expected_duration = Duration::hours(24);
        let actual_duration = secret.expires_at - secret.created_at;

        // Allow 1 second tolerance for test execution time
        assert!((actual_duration - expected_duration).num_seconds().abs() < 1);
    }

    #[test]
    fn test_secret_default_expiration() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            None,
            None, // Should default to 24 hours
            true,
        );

        let expected_duration = Duration::hours(24);
        let actual_duration = secret.expires_at - secret.created_at;

        assert!((actual_duration - expected_duration).num_seconds().abs() < 1);
    }

    #[test]
    fn test_is_expired_with_future_time() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            None,
            Some(24),
            true,
        );

        assert!(!secret.is_expired());
    }

    #[test]
    fn test_is_expired_with_past_time() {
        let mut secret = Secret::new(
            "encrypted_data".to_string(),
            None,
            Some(24),
            true,
        );

        // Set expires_at to the past
        secret.expires_at = Utc::now() - Duration::hours(1);

        assert!(secret.is_expired());
    }

    #[test]
    fn test_is_max_views_reached_with_none() {
        let secret = Secret::new(
            "encrypted_data".to_string(),
            None, // No max views limit
            None,
            true,
        );

        assert!(!secret.is_max_views_reached());
    }

    #[test]
    fn test_is_max_views_reached_below_limit() {
        let mut secret = Secret::new(
            "encrypted_data".to_string(),
            Some(5),
            None,
            true,
        );

        secret.views = 3;
        assert!(!secret.is_max_views_reached());
    }

    #[test]
    fn test_is_max_views_reached_at_limit() {
        let mut secret = Secret::new(
            "encrypted_data".to_string(),
            Some(5),
            None,
            true,
        );

        secret.views = 5;
        assert!(secret.is_max_views_reached());
    }

    #[test]
    fn test_is_max_views_reached_above_limit() {
        let mut secret = Secret::new(
            "encrypted_data".to_string(),
            Some(5),
            None,
            true,
        );

        secret.views = 10;
        assert!(secret.is_max_views_reached());
    }
}
