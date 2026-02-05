mod repository;
pub mod postgres;
pub mod dynamodb;

pub use repository::SecretRepository;
#[cfg(test)]
pub use repository::MockSecretRepository;
pub use postgres::PostgresRepository;
pub use dynamodb::DynamoDbRepository;

// Backward compatibility alias
pub type Database = PostgresRepository;
