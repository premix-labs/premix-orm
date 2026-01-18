pub use async_trait;
pub use sqlx;

pub mod prelude {
    pub use crate::{Executor, IntoExecutor, Model, Premix, UpdateResult};
}
use sqlx::{Database, Executor as SqlxExecutor, IntoArguments};

pub struct Premix;
pub mod migrator;
pub use migrator::{Migration, Migrator};

// Chapter 18: Multi-Database Support
// We define a trait that encapsulates all the requirements for a database to work with Premix.
pub trait SqlDialect: Database + Sized + Send + Sync
where
    Self::Connection: Send,
{
    fn placeholder(n: usize) -> String;
    fn auto_increment_pk() -> &'static str;
    fn rows_affected(res: &Self::QueryResult) -> u64;
    fn last_insert_id(res: &Self::QueryResult) -> i64;

    fn current_timestamp_fn() -> &'static str {
        "CURRENT_TIMESTAMP"
    }
    fn int_type() -> &'static str {
        "INTEGER"
    }
    fn text_type() -> &'static str {
        "TEXT"
    }
    fn bool_type() -> &'static str {
        "BOOLEAN"
    }
    fn float_type() -> &'static str {
        "REAL"
    }
}

#[cfg(feature = "sqlite")]
impl SqlDialect for sqlx::Sqlite {
    fn placeholder(_n: usize) -> String {
        "?".to_string()
    }
    fn auto_increment_pk() -> &'static str {
        "INTEGER PRIMARY KEY"
    }
    fn rows_affected(res: &sqlx::sqlite::SqliteQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(res: &sqlx::sqlite::SqliteQueryResult) -> i64 {
        res.last_insert_rowid()
    }
}

#[cfg(feature = "postgres")]
impl SqlDialect for sqlx::Postgres {
    fn placeholder(n: usize) -> String {
        format!("${}", n)
    }
    fn auto_increment_pk() -> &'static str {
        "SERIAL PRIMARY KEY"
    }
    fn rows_affected(res: &sqlx::postgres::PgQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(_res: &sqlx::postgres::PgQueryResult) -> i64 {
        0
    }
}

#[cfg(feature = "mysql")]
impl SqlDialect for sqlx::MySql {
    fn placeholder(_n: usize) -> String {
        "?".to_string()
    }
    fn auto_increment_pk() -> &'static str {
        "INTEGER AUTO_INCREMENT PRIMARY KEY"
    }
    fn rows_affected(res: &sqlx::mysql::MySqlQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(res: &sqlx::mysql::MySqlQueryResult) -> i64 {
        res.last_insert_id() as i64
    }
}

// Chapter 7: Stronger Executor Abstraction
pub enum Executor<'a, DB: Database> {
    Pool(&'a sqlx::Pool<DB>),
    Conn(&'a mut DB::Connection),
}

unsafe impl<'a, DB: Database> Send for Executor<'a, DB> where DB::Connection: Send {}
unsafe impl<'a, DB: Database> Sync for Executor<'a, DB> where DB::Connection: Sync {}

impl<'a, DB: Database> From<&'a sqlx::Pool<DB>> for Executor<'a, DB> {
    fn from(pool: &'a sqlx::Pool<DB>) -> Self {
        Self::Pool(pool)
    }
}

impl<'a, DB: Database> From<&'a mut DB::Connection> for Executor<'a, DB> {
    fn from(conn: &'a mut DB::Connection) -> Self {
        Self::Conn(conn)
    }
}

pub trait IntoExecutor<'a>: Send + 'a {
    type DB: SqlDialect;
    fn into_executor(self) -> Executor<'a, Self::DB>;
}

impl<'a, DB: SqlDialect> IntoExecutor<'a> for &'a sqlx::Pool<DB> {
    type DB = DB;
    fn into_executor(self) -> Executor<'a, DB> {
        Executor::Pool(self)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> IntoExecutor<'a> for &'a mut sqlx::SqliteConnection {
    type DB = sqlx::Sqlite;
    fn into_executor(self) -> Executor<'a, Self::DB> {
        Executor::Conn(self)
    }
}

#[cfg(feature = "postgres")]
impl<'a> IntoExecutor<'a> for &'a mut sqlx::postgres::PgConnection {
    type DB = sqlx::Postgres;
    fn into_executor(self) -> Executor<'a, Self::DB> {
        Executor::Conn(self)
    }
}

impl<'a, DB: SqlDialect> IntoExecutor<'a> for Executor<'a, DB> {
    type DB = DB;
    fn into_executor(self) -> Executor<'a, DB> {
        self
    }
}

impl<'a, DB: Database> Executor<'a, DB> {
    pub async fn execute<'q, A>(
        &mut self,
        query: sqlx::query::Query<'q, DB, A>,
    ) -> Result<DB::QueryResult, sqlx::Error>
    where
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.execute(*pool).await,
            Self::Conn(conn) => query.execute(&mut **conn).await,
        }
    }

    pub async fn fetch_all<'q, T, A>(
        &mut self,
        query: sqlx::query::QueryAs<'q, DB, T, A>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.fetch_all(*pool).await,
            Self::Conn(conn) => query.fetch_all(&mut **conn).await,
        }
    }

    pub async fn fetch_optional<'q, T, A>(
        &mut self,
        query: sqlx::query::QueryAs<'q, DB, T, A>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.fetch_optional(*pool).await,
            Self::Conn(conn) => query.fetch_optional(&mut **conn).await,
        }
    }
}

// Chapter 8: Weak Hook Pattern
#[async_trait::async_trait]
pub trait ModelHooks {
    async fn before_save(&mut self) -> Result<(), sqlx::Error> {
        Ok(())
    }
    async fn after_save(&mut self) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync> ModelHooks for T {}

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

impl<T> ModelValidation for T {}

#[async_trait::async_trait]
pub trait Model<DB: Database>: Sized + Send + Sync + Unpin
where
    DB: SqlDialect,
    for<'r> Self: sqlx::FromRow<'r, DB::Row>,
{
    fn table_name() -> &'static str;
    fn create_table_sql() -> String;
    fn list_columns() -> Vec<String>;

    /// Saves the current instance to the database.
    async fn save<'a, E>(&mut self, executor: E) -> Result<(), sqlx::Error>
    where
        E: IntoExecutor<'a, DB = DB>;

    async fn update<'a, E>(&mut self, executor: E) -> Result<UpdateResult, sqlx::Error>
    where
        E: IntoExecutor<'a, DB = DB>;

    // Chapter 16: Soft Delete support
    async fn delete<'a, E>(&mut self, executor: E) -> Result<(), sqlx::Error>
    where
        E: IntoExecutor<'a, DB = DB>;
    fn has_soft_delete() -> bool;

    /// Finds a record by its Primary Key.
    async fn find_by_id<'a, E>(executor: E, id: i32) -> Result<Option<Self>, sqlx::Error>
    where
        E: IntoExecutor<'a, DB = DB>;

    /// Use raw SQL and map rows into the current model type.
    fn raw_sql<'q>(
        sql: &'q str,
    ) -> sqlx::query::QueryAs<'q, DB, Self, <DB as Database>::Arguments<'q>> {
        sqlx::query_as::<DB, Self>(sql)
    }

    async fn eager_load<'a, E>(
        _models: &mut [Self],
        _relation: &str,
        _executor: E,
    ) -> Result<(), sqlx::Error>
    where
        E: IntoExecutor<'a, DB = DB>,
    {
        Ok(())
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

pub struct QueryBuilder<'a, T, DB: Database> {
    executor: Executor<'a, DB>,
    filters: Vec<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    includes: Vec<String>,
    include_deleted: bool, // Chapter 16
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T, DB> QueryBuilder<'a, T, DB>
where
    DB: SqlDialect,
    T: Model<DB>,
{
    pub fn new(executor: Executor<'a, DB>) -> Self {
        Self {
            executor,
            filters: Vec::new(),
            limit: None,
            offset: None,
            includes: Vec::new(),
            include_deleted: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn filter(mut self, condition: impl Into<String>) -> Self {
        self.filters.push(condition.into());
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn include(mut self, relation: impl Into<String>) -> Self {
        self.includes.push(relation.into());
        self
    }

    // Chapter 16: Soft Delete toggle
    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    /// Returns the SELECT SQL that would be executed for this query.
    pub fn to_sql(&self) -> String {
        let mut sql = format!(
            "SELECT * FROM {}{}",
            T::table_name(),
            self.build_where_clause()
        );

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// Returns the UPDATE SQL that would be executed for this query.
    pub fn to_update_sql(&self, values: &serde_json::Value) -> Result<String, sqlx::Error> {
        let obj = values.as_object().ok_or_else(|| {
            sqlx::Error::Protocol("Bulk update requires a JSON object".to_string())
        })?;

        let mut i = 1;
        let set_clause = obj
            .keys()
            .map(|k| {
                let p = DB::placeholder(i);
                i += 1;
                format!("{} = {}", k, p)
            })
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!(
            "UPDATE {} SET {}{}",
            T::table_name(),
            set_clause,
            self.build_where_clause()
        ))
    }

    /// Returns the DELETE (or soft delete) SQL that would be executed for this query.
    pub fn to_delete_sql(&self) -> String {
        if T::has_soft_delete() {
            format!(
                "UPDATE {} SET deleted_at = {}{}",
                T::table_name(),
                DB::current_timestamp_fn(),
                self.build_where_clause()
            )
        } else {
            format!(
                "DELETE FROM {}{}",
                T::table_name(),
                self.build_where_clause()
            )
        }
    }

    fn build_where_clause(&self) -> String {
        let mut filters = self.filters.clone();

        // Chapter 16: Handle Soft Delete filtering
        if T::has_soft_delete() && !self.include_deleted {
            filters.push("deleted_at IS NULL".to_string());
        }

        if filters.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", filters.join(" AND "))
        }
    }
}

impl<'a, T, DB> QueryBuilder<'a, T, DB>
where
    DB: SqlDialect,
    T: Model<DB>,
    for<'q> <DB as Database>::Arguments<'q>: IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: SqlxExecutor<'c, Database = DB>,
    for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
    DB::Connection: Send,
    T: Send,
{
    pub async fn all(mut self) -> Result<Vec<T>, sqlx::Error> {
        let mut sql = format!(
            "SELECT * FROM {}{}",
            T::table_name(),
            self.build_where_clause()
        );

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut results: Vec<T> = match &mut self.executor {
            Executor::Pool(pool) => sqlx::query_as::<DB, T>(&sql).fetch_all(*pool).await?,
            Executor::Conn(conn) => sqlx::query_as::<DB, T>(&sql).fetch_all(&mut **conn).await?,
        };

        for relation in self.includes {
            match &mut self.executor {
                Executor::Pool(pool) => {
                    T::eager_load(&mut results, &relation, Executor::Pool(*pool)).await?;
                }
                Executor::Conn(conn) => {
                    T::eager_load(&mut results, &relation, Executor::Conn(&mut **conn)).await?;
                }
            }
        }

        Ok(results)
    }

    // Chapter 17: Bulk Operations
    pub async fn update(mut self, values: serde_json::Value) -> Result<u64, sqlx::Error>
    where
        String: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        i64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        f64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        bool: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Option<String>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        let obj = values.as_object().ok_or_else(|| {
            sqlx::Error::Protocol("Bulk update requires a JSON object".to_string())
        })?;

        let mut i = 1;
        let set_clause = obj
            .keys()
            .map(|k| {
                let p = DB::placeholder(i);
                i += 1;
                format!("{} = {}", k, p)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "UPDATE {} SET {}{}",
            T::table_name(),
            set_clause,
            self.build_where_clause()
        );

        let mut query = sqlx::query::<DB>(&sql);
        for val in obj.values() {
            match val {
                serde_json::Value::String(s) => query = query.bind(s.clone()),
                serde_json::Value::Number(n) => {
                    if let Some(v) = n.as_i64() {
                        query = query.bind(v);
                    } else if let Some(v) = n.as_f64() {
                        query = query.bind(v);
                    }
                }
                serde_json::Value::Bool(b) => query = query.bind(*b),
                serde_json::Value::Null => query = query.bind(Option::<String>::None),
                _ => {
                    return Err(sqlx::Error::Protocol(
                        "Unsupported type in bulk update".to_string(),
                    ));
                }
            }
        }

        match &mut self.executor {
            Executor::Pool(pool) => {
                let res = query.execute(*pool).await?;
                Ok(DB::rows_affected(&res))
            }
            Executor::Conn(conn) => {
                let res = query.execute(&mut **conn).await?;
                Ok(DB::rows_affected(&res))
            }
        }
    }

    pub async fn delete(mut self) -> Result<u64, sqlx::Error> {
        let sql = if T::has_soft_delete() {
            format!(
                "UPDATE {} SET deleted_at = {}{}",
                T::table_name(),
                DB::current_timestamp_fn(),
                self.build_where_clause()
            )
        } else {
            format!(
                "DELETE FROM {}{}",
                T::table_name(),
                self.build_where_clause()
            )
        };

        match &mut self.executor {
            Executor::Pool(pool) => {
                let res = sqlx::query::<DB>(&sql).execute(*pool).await?;
                Ok(DB::rows_affected(&res))
            }
            Executor::Conn(conn) => {
                let res = sqlx::query::<DB>(&sql).execute(&mut **conn).await?;
                Ok(DB::rows_affected(&res))
            }
        }
    }
}

impl Premix {
    pub async fn sync<DB, M>(pool: &sqlx::Pool<DB>) -> Result<(), sqlx::Error>
    where
        DB: SqlDialect,
        M: Model<DB>,
        for<'q> <DB as Database>::Arguments<'q>: IntoArguments<'q, DB>,
        for<'c> &'c mut <DB as Database>::Connection: SqlxExecutor<'c, Database = DB>,
        for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
    {
        let sql = M::create_table_sql();
        sqlx::query::<DB>(&sql).execute(pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{sqlite::SqliteRow, Sqlite, SqlitePool};

    #[derive(Debug)]
    struct SoftDeleteModel {
        id: i32,
        status: String,
        deleted_at: Option<String>,
    }

    #[derive(Debug)]
    struct HardDeleteModel {
        id: i32,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct DbModel {
        id: i32,
        status: String,
        deleted_at: Option<String>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct DbHardModel {
        id: i32,
        status: String,
    }

    impl<'r> sqlx::FromRow<'r, SqliteRow> for SoftDeleteModel {
        fn from_row(_row: &SqliteRow) -> Result<Self, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }
    }

    impl<'r> sqlx::FromRow<'r, SqliteRow> for HardDeleteModel {
        fn from_row(_row: &SqliteRow) -> Result<Self, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }
    }

    #[async_trait::async_trait]
    impl Model<Sqlite> for DbModel {
        fn table_name() -> &'static str {
            "db_users"
        }
        fn create_table_sql() -> String {
            String::new()
        }
        fn list_columns() -> Vec<String> {
            vec!["id".into(), "status".into(), "deleted_at".into()]
        }
        async fn save<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        async fn update<'a, E>(&mut self, _executor: E) -> Result<UpdateResult, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(UpdateResult::NotImplemented)
        }
        async fn delete<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            true
        }
        async fn find_by_id<'a, E>(
            _executor: E,
            _id: i32,
        ) -> Result<Option<Self>, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(None)
        }
    }

    #[async_trait::async_trait]
    impl Model<Sqlite> for DbHardModel {
        fn table_name() -> &'static str {
            "db_hard_users"
        }
        fn create_table_sql() -> String {
            String::new()
        }
        fn list_columns() -> Vec<String> {
            vec!["id".into(), "status".into()]
        }
        async fn save<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        async fn update<'a, E>(&mut self, _executor: E) -> Result<UpdateResult, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(UpdateResult::NotImplemented)
        }
        async fn delete<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            false
        }
        async fn find_by_id<'a, E>(
            _executor: E,
            _id: i32,
        ) -> Result<Option<Self>, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(None)
        }
    }

    #[async_trait::async_trait]
    impl Model<Sqlite> for SoftDeleteModel {
        fn table_name() -> &'static str {
            "users"
        }
        fn create_table_sql() -> String {
            String::new()
        }
        fn list_columns() -> Vec<String> {
            Vec::new()
        }
        async fn save<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        async fn update<'a, E>(&mut self, _executor: E) -> Result<UpdateResult, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(UpdateResult::NotImplemented)
        }
        async fn delete<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            true
        }
        async fn find_by_id<'a, E>(
            _executor: E,
            _id: i32,
        ) -> Result<Option<Self>, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(None)
        }
    }

    #[async_trait::async_trait]
    impl Model<Sqlite> for HardDeleteModel {
        fn table_name() -> &'static str {
            "hard_users"
        }
        fn create_table_sql() -> String {
            String::new()
        }
        fn list_columns() -> Vec<String> {
            Vec::new()
        }
        async fn save<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        async fn update<'a, E>(&mut self, _executor: E) -> Result<UpdateResult, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(UpdateResult::NotImplemented)
        }
        async fn delete<'a, E>(&mut self, _executor: E) -> Result<(), sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            false
        }
        async fn find_by_id<'a, E>(
            _executor: E,
            _id: i32,
        ) -> Result<Option<Self>, sqlx::Error>
        where
            E: IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn query_builder_to_sql_includes_soft_delete_filter() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = SoftDeleteModel::find_in_pool(&pool)
            .filter("age > 18")
            .limit(10)
            .offset(5);
        let sql = query.to_sql();
        assert!(sql.contains("FROM users"));
        assert!(sql.contains("age > 18"));
        assert!(sql.contains("deleted_at IS NULL"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 5"));
    }

    #[tokio::test]
    async fn query_builder_to_sql_without_filters_has_no_where_for_hard_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = HardDeleteModel::find_in_pool(&pool);
        let sql = query.to_sql();
        assert!(sql.contains("FROM hard_users"));
        assert!(!sql.contains(" WHERE "));
    }

    #[tokio::test]
    async fn query_builder_with_deleted_skips_soft_delete_filter() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = SoftDeleteModel::find_in_pool(&pool)
            .filter("age > 18")
            .with_deleted();
        let sql = query.to_sql();
        assert!(sql.contains("age > 18"));
        assert!(!sql.contains("deleted_at IS NULL"));
    }

    #[tokio::test]
    async fn query_builder_to_update_sql_includes_fields() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = SoftDeleteModel::find_in_pool(&pool).filter("status = 'inactive'");
        let sql = query
            .to_update_sql(&serde_json::json!({ "status": "active", "age": 1 }))
            .unwrap();
        assert!(sql.contains("UPDATE users SET"));
        assert!(sql.contains("status"));
        assert!(sql.contains("age"));
        assert!(sql.contains("WHERE"));
    }

    #[tokio::test]
    async fn query_builder_to_update_sql_rejects_non_object() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = SoftDeleteModel::find_in_pool(&pool);
        let err = query.to_update_sql(&serde_json::json!("bad")).unwrap_err();
        assert!(err.to_string().contains("Bulk update requires a JSON object"));
    }

    #[tokio::test]
    async fn query_builder_to_delete_sql_soft_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = SoftDeleteModel::find_in_pool(&pool).filter("id = 1");
        let sql = query.to_delete_sql();
        assert!(sql.starts_with("UPDATE users SET deleted_at"));
    }

    #[tokio::test]
    async fn query_builder_to_delete_sql_hard_delete() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let query = HardDeleteModel::find_in_pool(&pool).filter("id = 1");
        let sql = query.to_delete_sql();
        assert!(sql.starts_with("DELETE FROM hard_users"));
    }

    #[test]
    fn model_raw_sql_compiles() {
        let _query = SoftDeleteModel::raw_sql("SELECT * FROM users");
    }

    #[test]
    fn sqlite_placeholder_uses_question_mark() {
        assert_eq!(Sqlite::placeholder(1), "?");
        assert_eq!(Sqlite::placeholder(5), "?");
    }

    #[test]
    fn sqlite_timestamp_fn_is_constant() {
        assert_eq!(Sqlite::current_timestamp_fn(), "CURRENT_TIMESTAMP");
    }

    #[tokio::test]
    async fn executor_execute_and_fetch() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT);")
            .execute(&pool)
            .await
            .unwrap();

        let mut executor = Executor::Pool(&pool);
        executor
            .execute(sqlx::query("INSERT INTO items (name) VALUES ('a');"))
            .await
            .unwrap();

        let mut executor = Executor::Pool(&pool);
        let row = executor
            .fetch_optional(sqlx::query_as::<Sqlite, (i64, String)>(
                "SELECT id, name FROM items WHERE name = 'a'",
            ))
            .await
            .unwrap();
        let (id, name) = row.unwrap();
        assert_eq!(name, "a");
        assert!(id > 0);

        let mut executor = Executor::Pool(&pool);
        let rows = executor
            .fetch_all(sqlx::query_as::<Sqlite, (i64, String)>(
                "SELECT id, name FROM items",
            ))
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn query_builder_update_executes() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE db_users (id INTEGER PRIMARY KEY, status TEXT, flag INTEGER, deleted_at TEXT);",
        )
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_users (status) VALUES ('inactive');")
            .execute(&pool)
            .await
            .unwrap();

        let updated = DbModel::find_in_pool(&pool)
            .filter("status = 'inactive'")
            .update(serde_json::json!({ "status": "active" }))
            .await
            .unwrap();
        assert_eq!(updated, 1);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_users WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn query_builder_update_binds_bool_and_null() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE db_users (id INTEGER PRIMARY KEY, status TEXT, flag INTEGER, deleted_at TEXT);",
        )
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_users (status) VALUES ('inactive');")
            .execute(&pool)
            .await
            .unwrap();

        let updated = DbModel::find_in_pool(&pool)
            .filter("id = 1")
            .update(serde_json::json!({ "status": "active", "flag": true, "deleted_at": null }))
            .await
            .unwrap();
        assert_eq!(updated, 1);
    }

    #[tokio::test]
    async fn query_builder_update_rejects_unsupported_type() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE db_users (id INTEGER PRIMARY KEY, status TEXT, deleted_at TEXT);")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_users (status) VALUES ('inactive');")
            .execute(&pool)
            .await
            .unwrap();

        let err = DbModel::find_in_pool(&pool)
            .filter("id = 1")
            .update(serde_json::json!({ "meta": { "a": 1 } }))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Unsupported type in bulk update"));
    }

    #[tokio::test]
    async fn query_builder_soft_delete_executes() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE db_users (id INTEGER PRIMARY KEY, status TEXT, deleted_at TEXT);")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_users (status) VALUES ('active');")
            .execute(&pool)
            .await
            .unwrap();

        let deleted = DbModel::find_in_pool(&pool)
            .filter("status = 'active'")
            .delete()
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_users WHERE deleted_at IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn query_builder_hard_delete_executes() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE db_hard_users (id INTEGER PRIMARY KEY, status TEXT);")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_hard_users (status) VALUES ('active');")
            .execute(&pool)
            .await
            .unwrap();

        let deleted = DbHardModel::find_in_pool(&pool)
            .filter("status = 'active'")
            .delete()
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_hard_users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn query_builder_all_with_limit_offset() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE db_users (id INTEGER PRIMARY KEY, status TEXT, deleted_at TEXT);")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO db_users (status) VALUES ('a'), ('b'), ('c');")
            .execute(&pool)
            .await
            .unwrap();

        let rows = DbModel::find_in_pool(&pool)
            .include("posts")
            .limit(1)
            .offset(1)
            .all()
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn default_model_hooks_are_noops() {
        let mut model = SoftDeleteModel {
            id: 1,
            status: "active".to_string(),
            deleted_at: None,
        };
        model.before_save().await.unwrap();
        model.after_save().await.unwrap();
    }

    #[test]
    fn default_model_validation_is_ok() {
        let model = SoftDeleteModel {
            id: 1,
            status: "active".to_string(),
            deleted_at: None,
        };
        assert!(model.validate().is_ok());
    }

    #[tokio::test]
    async fn eager_load_default_is_ok() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let mut models = vec![SoftDeleteModel {
            id: 1,
            status: "active".to_string(),
            deleted_at: None,
        }];
        SoftDeleteModel::eager_load(&mut models, "posts", &pool)
            .await
            .unwrap();
    }

    #[test]
    fn test_models_use_fields() {
        let soft = SoftDeleteModel {
            id: 1,
            status: "active".to_string(),
            deleted_at: None,
        };
        let hard = HardDeleteModel { id: 2 };
        assert_eq!(soft.id, 1);
        assert_eq!(soft.status, "active");
        assert!(soft.deleted_at.is_none());
        assert_eq!(hard.id, 2);
    }
}
