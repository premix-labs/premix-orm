use crate::dialect::SqlDialect;
use crate::executor::Executor;
use crate::executor::IntoExecutor;
use crate::query::QueryBuilder;
use sqlx::{Database, FromRow};
use std::future::Future;

// Chapter 8: Weak Hook Pattern
#[inline(never)]
fn default_model_hook_result() -> Result<(), sqlx::Error> {
    Ok(())
}

pub trait ModelHooks {
    #[inline(never)]
    fn before_save(&mut self) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
    #[inline(never)]
    fn after_save(&mut self) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
}

// Chapter 9: Optimistic Locking
#[derive(Debug, PartialEq)]
pub enum UpdateResult {
    Success,
    VersionConflict,
    NotFound,
    NotImplemented,
}

// Chapter 10: Validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub trait ModelValidation {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        Ok(())
    }
}

pub trait Model<DB: Database>: Sized + Send + Sync + Unpin
where
    DB: SqlDialect,
    for<'r> Self: FromRow<'r, DB::Row>,
{
    fn table_name() -> &'static str;
    fn create_table_sql() -> String;
    fn list_columns() -> Vec<String>;

    /// Saves the current instance to the database.
    fn save<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    fn update<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    // Chapter 16: Soft Delete support
    fn delete<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;
    fn has_soft_delete() -> bool;
    fn sensitive_fields() -> &'static [&'static str] {
        &[]
    }

    /// Finds a record by its Primary Key.
    fn find_by_id<'a, E>(
        executor: E,
        id: i32,
    ) -> impl Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Use raw SQL and map rows into the current model type.
    fn raw_sql<'q>(
        sql: &'q str,
    ) -> sqlx::query::QueryAs<'q, DB, Self, <DB as Database>::Arguments<'q>> {
        sqlx::query_as::<DB, Self>(sql)
    }

    #[inline(never)]
    fn eager_load<'a>(
        _models: &mut [Self],
        _relation: &str,
        _executor: Executor<'a, DB>,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
    fn find<'a, E>(executor: E) -> QueryBuilder<'a, Self, DB>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        QueryBuilder::new(executor.into_executor())
    }

    // Convenience helpers
    fn find_in_pool(pool: &sqlx::Pool<DB>) -> QueryBuilder<'_, Self, DB> {
        QueryBuilder::new(Executor::Pool(pool))
    }

    fn find_in_tx(conn: &mut DB::Connection) -> QueryBuilder<'_, Self, DB> {
        QueryBuilder::new(Executor::Conn(conn))
    }
}
