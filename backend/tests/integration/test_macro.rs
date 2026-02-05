#[macro_export]
macro_rules! test_both_databases {
    ($test_name:ident, $test_fn:expr) => {
        paste::paste! {
            #[cfg(feature = "postgres-tests")]
            #[tokio::test]
            async fn [<$test_name _postgres>]() {
                use crate::integration::postgres_context::PostgresTestContext;
                let ctx = PostgresTestContext::new().await;
                let test_fn = $test_fn;
                test_fn(&ctx).await;
            }

            #[cfg(feature = "dynamodb-tests")]
            #[tokio::test]
            async fn [<$test_name _dynamodb>]() {
                use crate::integration::dynamodb_context::DynamoDbTestContext;
                let ctx = DynamoDbTestContext::new().await;
                let test_fn = $test_fn;
                test_fn(&ctx).await;
            }
        }
    };
}
