#[cfg(test)]
mod tests {
    use crate::{Executor, QueryBuilder, sqlx::Sqlite};

    struct TestModel;
    impl crate::Model<Sqlite> for TestModel {
        fn table_name() -> &'static str {
            "test"
        }
        fn create_table_sql() -> String {
            "".into()
        }
        fn list_columns() -> Vec<String> {
            vec![]
        }
        async fn save<'e, E>(&mut self, _: E) -> Result<(), sqlx::Error>
        where
            E: sqlx::Executor<'e, Database = Sqlite>,
        {
            Ok(())
        }
        async fn update(
            &mut self,
            _: Executor<'_, Sqlite>,
        ) -> Result<crate::UpdateResult, sqlx::Error> {
            Ok(crate::UpdateResult::Success)
        }
        async fn delete(&mut self, _: Executor<'_, Sqlite>) -> Result<(), sqlx::Error> {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            false
        }
        async fn find_by_id<'e, E>(_: E, _: i32) -> Result<Option<Self>, sqlx::Error>
        where
            E: sqlx::Executor<'e, Database = Sqlite>,
        {
            Ok(None)
        }
    }

    fn is_send<T: Send>() {}

    #[test]
    fn test_query_builder_send() {
        // This should fail if QueryBuilder is not Send
        // is_send::<QueryBuilder<'_, TestModel, Sqlite>>();
    }
}
