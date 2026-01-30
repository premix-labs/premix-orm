use premix_core::QueryBuilder;
use premix_core::dialect::SqlDialect;
use premix_core::executor::Executor;
use premix_core::model::{Model, ModelHooks, ModelValidation, UpdateResult};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, TestRunner};
use sqlx::Sqlite;

#[allow(dead_code)]
struct DummyModel {
    id: i32,
    name: String,
}

impl ModelHooks for DummyModel {}
impl ModelValidation for DummyModel {}

impl Model<Sqlite> for DummyModel {
    fn table_name() -> &'static str {
        "dummy_models"
    }

    fn create_table_sql() -> String {
        "CREATE TABLE dummy_models (id INTEGER PRIMARY KEY, name TEXT)".to_string()
    }

    fn list_columns() -> Vec<String> {
        vec!["id".to_string(), "name".to_string()]
    }

    #[allow(clippy::manual_async_fn)]
    fn save<'a, E>(
        &'a mut self,
        _executor: E,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: premix_core::executor::IntoExecutor<'a, DB = Sqlite>,
    {
        async move { Ok(()) }
    }

    #[allow(clippy::manual_async_fn)]
    fn update<'a, E>(
        &'a mut self,
        _executor: E,
    ) -> impl std::future::Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: premix_core::executor::IntoExecutor<'a, DB = Sqlite>,
    {
        async move { Ok(UpdateResult::Success) }
    }

    #[allow(clippy::manual_async_fn)]
    fn delete<'a, E>(
        &'a mut self,
        _executor: E,
    ) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: premix_core::executor::IntoExecutor<'a, DB = Sqlite>,
    {
        async move { Ok(()) }
    }

    #[allow(clippy::manual_async_fn)]
    fn find_by_id<'a, E>(
        _executor: E,
        _id: i32,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        E: premix_core::executor::IntoExecutor<'a, DB = Sqlite>,
    {
        async move { Ok(None) }
    }

    fn has_soft_delete() -> bool {
        false
    }

    fn sensitive_fields() -> &'static [&'static str] {
        &[]
    }

    fn from_row_fast(_row: &<Sqlite as sqlx::Database>::Row) -> Result<Self, sqlx::Error> {
        Ok(DummyModel {
            id: 0,
            name: String::new(),
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for DummyModel {
    fn from_row(_row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(DummyModel {
            id: 0,
            name: String::new(),
        })
    }
}

fn column_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9_\\-\\s]{0,32}".prop_map(|s| s)
}

#[tokio::test]
async fn filter_eq_quotes_identifier() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let mut runner = TestRunner::new(Config {
        cases: 128,
        failure_persistence: None,
        ..Config::default()
    });

    let column = column_strategy();
    let value = any::<i64>();

    for _ in 0..128 {
        let column = column.new_tree(&mut runner).unwrap().current();
        let value = value.new_tree(&mut runner).unwrap().current();
        let qb = QueryBuilder::<DummyModel, Sqlite>::new(Executor::from(&pool));
        let sql = qb.filter_eq(column.clone(), value).to_sql();
        let quoted = <Sqlite as SqlDialect>::quote_identifier(&column);
        assert!(sql.contains(&quoted));
    }
}

#[tokio::test]
async fn filter_in_handles_empty_list() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let mut runner = TestRunner::new(Config {
        cases: 64,
        failure_persistence: None,
        ..Config::default()
    });

    let column = column_strategy();
    for _ in 0..64 {
        let column = column.new_tree(&mut runner).unwrap().current();
        let qb = QueryBuilder::<DummyModel, Sqlite>::new(Executor::from(&pool));
        let sql = qb.filter_in(column, Vec::<i64>::new()).to_sql();
        assert!(sql.contains("1=0"));
    }
}
