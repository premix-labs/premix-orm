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

/// Hooks that can be implemented to run logic before or after database operations.
pub trait ModelHooks {
    /// Ran before a model is saved to the database.
    #[inline(never)]
    fn before_save(&mut self) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
    /// Ran after a model is successfully saved to the database.
    #[inline(never)]
    fn after_save(&mut self) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
}

// Chapter 9: Optimistic Locking
/// The result of an update operation, particularly relevant for optimistic locking.
#[derive(Debug, PartialEq)]
pub enum UpdateResult {
    /// The update was successful.
    Success,
    /// The update failed due to a version mismatch (optimistic locking).
    VersionConflict,
    /// The record was not found.
    NotFound,
    /// The update operation is not implemented for this model.
    NotImplemented,
}

// Chapter 10: Validation
/// An error that occurred during model validation.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The name of the field that failed validation.
    pub field: String,
    /// A human-readable message describing the validation failure.
    pub message: String,
}

/// A trait for validating model data before it is saved to the database.
pub trait ModelValidation {
    /// Validates the model. Returns `Err` with a list of validation errors if validation fails.
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        Ok(())
    }
}

/// The core trait for database models.
///
/// This trait provides the foundation for all database interactions for a specific entity.
/// It is usually implemented automatically via `#[derive(Model)]`.
pub trait Model<DB: Database>: Sized + Send + Sync + Unpin
where
    DB: SqlDialect,
    for<'r> Self: FromRow<'r, DB::Row>,
{
    /// Returns the name of the database table associated with this model.
    fn table_name() -> &'static str;
    /// Returns the SQL string required to create the table for this model.
    fn create_table_sql() -> String;
    /// Returns a list of column names for this model.
    fn list_columns() -> Vec<String>;

    /// Saves the current instance to the database.
    fn save<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Updates the current instance in the database using optimistic locking if a `version` field exists.
    fn update<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    // Chapter 16: Soft Delete support
    /// Deletes the current instance from the database (either hard or soft delete).
    fn delete<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;
    /// Returns whether this model supports soft deletes (via a `deleted_at` field).
    fn has_soft_delete() -> bool;
    /// Returns a list of fields that are considered sensitive and should be redacted in logs.
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

    /// Loads related models for a list of instances.
    #[inline(never)]
    fn eager_load<'a>(
        _models: &mut [Self],
        _relation: &str,
        _executor: Executor<'a, DB>,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async move { default_model_hook_result() }
    }
    /// Creates a new [`QueryBuilder`] for this model.
    fn find<'a, E>(executor: E) -> QueryBuilder<'a, Self, DB>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        QueryBuilder::new(executor.into_executor())
    }

    // Convenience helpers
    /// Creates a new [`QueryBuilder`] using a connection pool.
    fn find_in_pool(pool: &sqlx::Pool<DB>) -> QueryBuilder<'_, Self, DB> {
        QueryBuilder::new(Executor::Pool(pool))
    }

    /// Creates a new [`QueryBuilder`] using an active database connection.
    fn find_in_tx(conn: &mut DB::Connection) -> QueryBuilder<'_, Self, DB> {
        QueryBuilder::new(Executor::Conn(conn))
    }
}
