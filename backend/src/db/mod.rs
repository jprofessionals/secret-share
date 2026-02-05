mod repository;
pub mod postgres;

pub use repository::SecretRepository;
#[cfg(test)]
pub use repository::MockSecretRepository;
pub use postgres::PostgresRepository;

// Backward compatibility alias
pub type Database = PostgresRepository;
