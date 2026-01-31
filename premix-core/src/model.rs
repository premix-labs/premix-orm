use crate::dialect::SqlDialect;
use crate::error::{PremixError, PremixResult};
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

/// Wrapper for fast positional row decoding.
#[derive(Debug)]
pub struct FastRow<DB, T>(T, std::marker::PhantomData<DB>);

impl<DB, T> FastRow<DB, T> {
    /// Extract the inner model.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<'r, DB, T> FromRow<'r, DB::Row> for FastRow<DB, T>
where
    DB: Database + SqlDialect,
    T: Model<DB>,
    usize: sqlx::ColumnIndex<DB::Row>,
    for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
{
    fn from_row(row: &'r DB::Row) -> Result<Self, sqlx::Error> {
        T::from_row_fast(row).map(|value| FastRow(value, std::marker::PhantomData))
    }
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

    /// Saves the current instance without hooks or extra safety checks.
    fn save_fast<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.save(executor)
    }

    /// Saves the current instance using the ultra-fast path (no hooks/extra checks).
    /// Note: On Postgres this may skip RETURNING and leave `id` unchanged.
    fn save_ultra<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.save_fast(executor)
    }

    /// Updates the current instance in the database using optimistic locking if a `version` field exists.
    fn update<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Updates the current instance without hooks or extra safety checks.
    fn update_fast<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.update(executor)
    }

    /// Updates the current instance using the ultra-fast path (no hooks/extra checks).
    fn update_ultra<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<UpdateResult, sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.update_fast(executor)
    }

    // Chapter 16: Soft Delete support
    /// Deletes the current instance from the database (either hard or soft delete).
    fn delete<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Deletes the current instance without hooks or extra safety checks.
    fn delete_fast<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.delete(executor)
    }

    /// Deletes the current instance using the ultra-fast path (no hooks/extra checks).
    fn delete_ultra<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        self.delete_fast(executor)
    }
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

    /// Fast row mapping using positional indices (override in derives for speed).
    fn from_row_fast(row: &DB::Row) -> Result<Self, sqlx::Error>
    where
        usize: sqlx::ColumnIndex<DB::Row>,
        for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
        for<'r> Self: FromRow<'r, DB::Row>,
    {
        <Self as sqlx::FromRow<'_, DB::Row>>::from_row(row)
    }

    /// Use raw SQL and map rows into the current model type.
    fn raw_sql<'q>(
        sql: &'q str,
    ) -> sqlx::query::QueryAs<'q, DB, Self, <DB as Database>::Arguments<'q>> {
        sqlx::query_as::<DB, Self>(sql)
    }

    /// Use raw SQL with fast positional mapping (select columns in model field order).
    fn raw_sql_fast<'q>(
        sql: &'q str,
    ) -> sqlx::query::QueryAs<'q, DB, crate::FastRow<DB, Self>, <DB as Database>::Arguments<'q>>
    where
        Self: Sized,
        usize: sqlx::ColumnIndex<DB::Row>,
        for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
    {
        sqlx::query_as::<DB, crate::FastRow<DB, Self>>(sql)
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

/// Convenience helpers that map sqlx errors into `PremixError`.
pub trait ModelResultExt<DB: Database>: Model<DB>
where
    DB: SqlDialect,
    for<'r> Self: FromRow<'r, DB::Row>,
{
    /// Save the model and return `PremixError` on failure.
    fn save_result<'a, E>(&'a mut self, executor: E) -> impl Future<Output = PremixResult<()>>
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Update the model and return `PremixError` on failure.
    fn update_result<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = PremixResult<UpdateResult>>
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Delete the model and return `PremixError` on failure.
    fn delete_result<'a, E>(&'a mut self, executor: E) -> impl Future<Output = PremixResult<()>>
    where
        E: IntoExecutor<'a, DB = DB>;
}

impl<T, DB> ModelResultExt<DB> for T
where
    DB: SqlDialect,
    T: Model<DB>,
    for<'r> T: FromRow<'r, DB::Row>,
{
    #[allow(clippy::manual_async_fn)]
    fn save_result<'a, E>(&'a mut self, executor: E) -> impl Future<Output = PremixResult<()>>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        async move { self.save(executor).await.map_err(PremixError::from) }
    }

    #[allow(clippy::manual_async_fn)]
    fn update_result<'a, E>(
        &'a mut self,
        executor: E,
    ) -> impl Future<Output = PremixResult<UpdateResult>>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        async move { self.update(executor).await.map_err(PremixError::from) }
    }

    #[allow(clippy::manual_async_fn)]
    fn delete_result<'a, E>(&'a mut self, executor: E) -> impl Future<Output = PremixResult<()>>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        async move { self.delete(executor).await.map_err(PremixError::from) }
    }
}
