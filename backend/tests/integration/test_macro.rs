/// Macro for running tests against both PostgreSQL and DynamoDB.
///
/// Usage:
/// ```ignore
/// test_both_databases!(test_name, |ctx| async move {
///     // test code using ctx.client() and ctx.url()
/// });
/// ```
#[macro_export]
macro_rules! test_both_databases {
    ($test_name:ident, |$ctx:ident| async move $body:block) => {
        paste::paste! {
            #[cfg(feature = "postgres-tests")]
            #[tokio::test]
            async fn [<$test_name _postgres>]() {
                use crate::integration::postgres_context::PostgresTestContext;
                use crate::integration::TestContextTrait;
                let $ctx = PostgresTestContext::new().await;
                $body
            }

            #[cfg(feature = "dynamodb-tests")]
            #[tokio::test]
            async fn [<$test_name _dynamodb>]() {
                use crate::integration::dynamodb_context::DynamoDbTestContext;
                use crate::integration::TestContextTrait;
                let $ctx = DynamoDbTestContext::new().await;
                $body
            }
        }
    };
}
