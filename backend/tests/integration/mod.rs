pub mod postgres_context;
pub mod dynamodb_context;
#[macro_use]
pub mod test_macro;

pub use postgres_context::PostgresTestContext as TestContext;

pub trait TestContextTrait {
    fn url(&self, path: &str) -> String;
    fn client(&self) -> &reqwest::Client;
}

impl TestContextTrait for postgres_context::PostgresTestContext {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl TestContextTrait for dynamodb_context::DynamoDbTestContext {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
