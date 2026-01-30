use crate::dialect::SqlDialect;
use crate::executor::Executor;
use crate::model::Model;
use futures_util::StreamExt;
use smallvec::{SmallVec, smallvec};
use sqlx::{Database, IntoArguments};
use std::time::{Duration, Instant};

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum BindValue {
    String(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Uuid(uuid::Uuid),
    DateTime(chrono::DateTime<chrono::Utc>),
    Null,
}

impl BindValue {
    fn to_log_string(&self) -> String {
        match self {
            BindValue::String(v) => v.clone(),
            BindValue::I64(v) => v.to_string(),
            BindValue::F64(v) => v.to_string(),
            BindValue::Bool(v) => v.to_string(),
            BindValue::Uuid(v) => v.to_string(),
            BindValue::DateTime(v) => v.to_rfc3339(),
            BindValue::Null => "NULL".to_string(),
        }
    }
}

#[cfg(feature = "metrics")]
fn record_query_metrics(operation: &str, table: &str, elapsed: Duration) {
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let labels = [
        ("operation", operation.to_string()),
        ("table", table.to_string()),
    ];
    metrics::histogram!("premix.query.duration_ms", &labels).record(elapsed_ms);
    metrics::counter!("premix.query.count", &labels).increment(1);
}

#[cfg(not(feature = "metrics"))]
fn record_query_metrics(_operation: &str, _table: &str, _elapsed: Duration) {}

#[inline(always)]
fn bind_value_query<'q, DB>(
    query: sqlx::query::Query<'q, DB, <DB as Database>::Arguments<'q>>,
    value: BindValue,
) -> sqlx::query::Query<'q, DB, <DB as Database>::Arguments<'q>>
where
    DB: Database,
    String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    i64: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    bool: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    uuid::Uuid: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    chrono::DateTime<chrono::Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    Option<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
    match value {
        BindValue::String(v) => query.bind(v),
        BindValue::I64(v) => query.bind(v),
        BindValue::F64(v) => query.bind(v),
        BindValue::Bool(v) => query.bind(v),
        BindValue::Uuid(v) => query.bind(v),
        BindValue::DateTime(v) => query.bind(v),
        BindValue::Null => query.bind(Option::<String>::None),
    }
}

#[inline(always)]
fn bind_value_query_as<'q, DB, T>(
    query: sqlx::query::QueryAs<'q, DB, T, <DB as Database>::Arguments<'q>>,
    value: BindValue,
) -> sqlx::query::QueryAs<'q, DB, T, <DB as Database>::Arguments<'q>>
where
    DB: Database,
    String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    i64: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    bool: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    uuid::Uuid: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    chrono::DateTime<chrono::Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    Option<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
    match value {
        BindValue::String(v) => query.bind(v),
        BindValue::I64(v) => query.bind(v),
        BindValue::F64(v) => query.bind(v),
        BindValue::Bool(v) => query.bind(v),
        BindValue::Uuid(v) => query.bind(v),
        BindValue::DateTime(v) => query.bind(v),
        BindValue::Null => query.bind(Option::<String>::None),
    }
}

#[inline]
fn apply_persistent_query_as<'q, DB, T>(
    query: sqlx::query::QueryAs<'q, DB, T, <DB as Database>::Arguments<'q>>,
    prepared: bool,
) -> sqlx::query::QueryAs<'q, DB, T, <DB as Database>::Arguments<'q>>
where
    DB: Database + sqlx::database::HasStatementCache,
{
    if prepared {
        query.persistent(true)
    } else {
        query
    }
}

#[inline]
fn apply_persistent_query<'q, DB>(
    query: sqlx::query::Query<'q, DB, <DB as Database>::Arguments<'q>>,
    prepared: bool,
) -> sqlx::query::Query<'q, DB, <DB as Database>::Arguments<'q>>
where
    DB: Database + sqlx::database::HasStatementCache,
{
    if prepared {
        query.persistent(true)
    } else {
        query
    }
}

impl From<String> for BindValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for BindValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<i32> for BindValue {
    fn from(value: i32) -> Self {
        Self::I64(value as i64)
    }
}

impl From<i64> for BindValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<f64> for BindValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<bool> for BindValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Option<String>> for BindValue {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(v) => Self::String(v),
            None => Self::Null,
        }
    }
}

impl From<uuid::Uuid> for BindValue {
    fn from(value: uuid::Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for BindValue {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self::DateTime(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FilterExpr {
    Raw(String),
    Compare {
        column: ColumnRef,
        op: FilterOp,
        values: SmallVec<[BindValue; 2]>,
    },
    NullCheck {
        column: ColumnRef,
        is_null: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterOp {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    In,
}

impl FilterOp {
    fn as_str(self) -> &'static str {
        match self {
            FilterOp::Eq => "=",
            FilterOp::Ne => "!=",
            FilterOp::Lt => "<",
            FilterOp::Lte => "<=",
            FilterOp::Gt => ">",
            FilterOp::Gte => ">=",
            FilterOp::Like => "LIKE",
            FilterOp::In => "IN",
        }
    }

    fn is_in(self) -> bool {
        matches!(self, FilterOp::In)
    }
}

/// Column reference used in filters (static literals or owned names).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnRef {
    /// Static column name known at compile time.
    Static(&'static str),
    /// Owned column name created at runtime.
    Owned(String),
}

impl ColumnRef {
    /// Builds a static column reference without allocation.
    pub const fn static_str(value: &'static str) -> Self {
        ColumnRef::Static(value)
    }

    fn as_str(&self) -> &str {
        match self {
            ColumnRef::Static(value) => value,
            ColumnRef::Owned(value) => value,
        }
    }
}

impl From<&str> for ColumnRef {
    fn from(value: &str) -> Self {
        ColumnRef::Owned(value.to_string())
    }
}

impl From<String> for ColumnRef {
    fn from(value: String) -> Self {
        ColumnRef::Owned(value)
    }
}

impl From<&String> for ColumnRef {
    fn from(value: &String) -> Self {
        ColumnRef::Owned(value.clone())
    }
}

/// A type-safe SQL query builder.
///
/// `QueryBuilder` provides a fluent interface for building SELECT, UPDATE, and DELETE queries
/// with support for filtering, pagination, eager loading, and soft deletes.
pub struct QueryBuilder<'a, T, DB: Database> {
    executor: Executor<'a, DB>,
    filters: Vec<FilterExpr>,
    limit: Option<i32>,
    offset: Option<i32>,
    includes: SmallVec<[String; 2]>,
    include_deleted: bool,
    allow_unsafe: bool,
    has_raw_filter: bool,
    fast_path: bool,
    unsafe_fast: bool,
    ultra_fast: bool,
    prepared: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T, DB: Database> std::fmt::Debug for QueryBuilder<'a, T, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryBuilder")
            .field("filters", &self.filters)
            .field("limit", &self.limit)
            .field("offset", &self.offset)
            .field("includes", &self.includes)
            .field("include_deleted", &self.include_deleted)
            .field("allow_unsafe", &self.allow_unsafe)
            .field("fast_path", &self.fast_path)
            .field("unsafe_fast", &self.unsafe_fast)
            .field("ultra_fast", &self.ultra_fast)
            .field("prepared", &self.prepared)
            .finish()
    }
}

impl<'a, T, DB> QueryBuilder<'a, T, DB>
where
    DB: SqlDialect + sqlx::database::HasStatementCache,
    T: Model<DB>,
{
    /// Creates a new `QueryBuilder` using the provided [`Executor`].
    pub fn new(executor: Executor<'a, DB>) -> Self {
        Self {
            executor,
            filters: Vec::with_capacity(4), // Pre-allocate for typical queries (1-4 filters)
            limit: None,
            offset: None,
            includes: SmallVec::with_capacity(2), // Pre-allocate for typical queries (1-2 includes)
            include_deleted: false,
            allow_unsafe: false,
            has_raw_filter: false,
            fast_path: false,
            unsafe_fast: false,
            ultra_fast: false,
            prepared: true,
            _marker: std::marker::PhantomData,
        }
    }

    /// Adds a raw SQL filter condition to the query.
    ///
    /// # Safety
    /// This method is potentially unsafe and requires calling [`allow_unsafe`] for the query to execute.
    pub fn filter(mut self, condition: impl Into<String>) -> Self {
        self.filters.push(FilterExpr::Raw(condition.into()));
        self.has_raw_filter = true;
        self
    }

    /// Adds a raw SQL filter condition to the query.
    pub fn filter_raw(self, condition: impl Into<String>) -> Self {
        self.filter(condition)
    }

    /// Adds an equality filter (`column = value`).
    pub fn filter_eq(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Eq,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a not-equal filter (`column != value`).
    pub fn filter_ne(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Ne,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a less-than filter (`column < value`).
    pub fn filter_lt(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Lt,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a less-than-or-equal filter (`column <= value`).
    pub fn filter_lte(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Lte,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a greater-than filter (`column > value`).
    pub fn filter_gt(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Gt,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a greater-than-or-equal filter (`column >= value`).
    pub fn filter_gte(mut self, column: impl Into<ColumnRef>, value: impl Into<BindValue>) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Gte,
            values: smallvec![value.into()],
        });
        self
    }

    /// Adds a LIKE filter (`column LIKE value`).
    pub fn filter_like(
        mut self,
        column: impl Into<ColumnRef>,
        value: impl Into<BindValue>,
    ) -> Self {
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::Like,
            values: smallvec![value.into()],
        });
        self
    }

    /// Filters rows where the column IS NULL.
    pub fn filter_is_null(mut self, column: impl Into<ColumnRef>) -> Self {
        self.filters.push(FilterExpr::NullCheck {
            column: column.into(),
            is_null: true,
        });
        self
    }

    /// Filters rows where the column IS NOT NULL.
    pub fn filter_is_not_null(mut self, column: impl Into<ColumnRef>) -> Self {
        self.filters.push(FilterExpr::NullCheck {
            column: column.into(),
            is_null: false,
        });
        self
    }

    /// Adds an IN filter (`column IN (values...)`).
    pub fn filter_in<I, V>(mut self, column: impl Into<ColumnRef>, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<BindValue>,
    {
        let values: SmallVec<[BindValue; 2]> = values.into_iter().map(Into::into).collect();
        self.filters.push(FilterExpr::Compare {
            column: column.into(),
            op: FilterOp::In,
            values,
        });
        self
    }

    fn format_filters_for_log(&self) -> String {
        let sensitive_fields = T::sensitive_fields();
        let mut rendered = String::with_capacity(128);
        let mut first_clause = true;
        let mut append_and = |buf: &mut String| {
            if first_clause {
                first_clause = false;
            } else {
                buf.push_str(" AND ");
            }
        };
        use std::fmt::Write;

        for filter in &self.filters {
            match filter {
                FilterExpr::Raw(_) => {
                    append_and(&mut rendered);
                    rendered.push_str("RAW(<redacted>)");
                }
                FilterExpr::Compare { column, op, values } => {
                    let column_name = column.as_str();
                    let is_sensitive = sensitive_fields.contains(&column_name);
                    if op.is_in() {
                        if values.is_empty() {
                            append_and(&mut rendered);
                            rendered.push_str("1=0");
                            continue;
                        }
                        append_and(&mut rendered);
                        let _ = write!(rendered, "{} IN (", column_name);
                        for (idx, value) in values.iter().enumerate() {
                            if idx > 0 {
                                rendered.push_str(", ");
                            }
                            if is_sensitive {
                                rendered.push_str("***");
                            } else {
                                rendered.push_str(&value.to_log_string());
                            }
                        }
                        rendered.push(')');
                    } else {
                        append_and(&mut rendered);
                        let _ = write!(rendered, "{} {} ", column_name, op.as_str());
                        if is_sensitive {
                            rendered.push_str("***");
                        } else if let Some(value) = values.first() {
                            rendered.push_str(&value.to_log_string());
                        } else {
                            rendered.push_str("NULL");
                        }
                    }
                }
                FilterExpr::NullCheck { column, is_null } => {
                    append_and(&mut rendered);
                    if *is_null {
                        let _ = write!(rendered, "{} IS NULL", column.as_str());
                    } else {
                        let _ = write!(rendered, "{} IS NOT NULL", column.as_str());
                    }
                }
            }
        }

        if T::has_soft_delete() && !self.include_deleted {
            append_and(&mut rendered);
            rendered.push_str("deleted_at IS NULL");
        }

        rendered
    }

    fn estimate_bind_count(&self) -> usize {
        let mut count = 0usize;
        for filter in &self.filters {
            if let FilterExpr::Compare { op, values, .. } = filter {
                if op.is_in() {
                    count = count.saturating_add(values.len());
                } else {
                    count = count.saturating_add(1);
                }
            }
        }
        count
    }

    /// Limits the number of rows returned by the query.
    /// Sets the maximum number of rows to return.
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Skips the specified number of rows.
    /// Sets the number of rows to skip.
    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Eager loads a related model.
    pub fn include(mut self, relation: impl Into<String>) -> Self {
        self.includes.push(relation.into());
        self
    }

    /// Includes soft-deleted records in the results.
    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    /// Explicitly allows potentially unsafe raw filters.
    /// Enables execution of queries with raw SQL filters.
    pub fn allow_unsafe(mut self) -> Self {
        self.allow_unsafe = true;
        self
    }

    /// Enables a fast path that skips logging and metrics for hot queries.
    pub fn fast(mut self) -> Self {
        self.fast_path = true;
        self
    }

    /// Enables an unsafe fast path that skips logging, metrics, and safety guards.
    pub fn unsafe_fast(mut self) -> Self {
        self.fast_path = true;
        self.unsafe_fast = true;
        self.allow_unsafe = true;
        self
    }

    /// Enables the ultra-fast path: skips logging, metrics, safety guards, and eager loading.
    /// Note: Any configured includes will be ignored.
    pub fn ultra_fast(mut self) -> Self {
        self.fast_path = true;
        self.unsafe_fast = true;
        self.allow_unsafe = true;
        self.ultra_fast = true;
        self
    }

    /// Enable prepared statement caching for this query (default: enabled).
    pub fn prepared(mut self) -> Self {
        self.prepared = true;
        self
    }

    /// Disable prepared statement caching for this query.
    pub fn unprepared(mut self) -> Self {
        self.prepared = false;
        self
    }

    /// Returns the SELECT SQL that would be executed for this query.
    pub fn to_sql(&self) -> String {
        let mut sql = String::with_capacity(128); // Pre-allocate reasonable size
        use std::fmt::Write;

        sql.push_str("SELECT * FROM ");
        sql.push_str(T::table_name());

        let mut dummy_binds: SmallVec<[BindValue; 8]> = SmallVec::new();
        self.render_where_clause_into(&mut sql, &mut dummy_binds, 1);

        if let Some(limit) = self.limit {
            let _ = write!(sql, " LIMIT {}", limit);
        }

        if let Some(offset) = self.offset {
            let _ = write!(sql, " OFFSET {}", offset);
        }

        sql
    }

    /// Returns the UPDATE SQL that would be executed for this query.
    pub fn to_update_sql(&self, values: &serde_json::Value) -> Result<String, sqlx::Error> {
        let obj = values.as_object().ok_or_else(|| {
            sqlx::Error::Protocol("Bulk update requires a JSON object".to_string())
        })?;

        let mut sql = String::with_capacity(256);
        use std::fmt::Write;

        let _ = write!(sql, "UPDATE {} SET ", T::table_name());

        let mut i = 1;
        let mut first = true;

        for k in obj.keys() {
            if !first {
                sql.push_str(", ");
            }
            let p = DB::placeholder(i);
            let _ = write!(sql, "{} = {}", DB::quote_identifier(k), p);
            i += 1;
            first = false;
        }

        let mut dummy_binds: SmallVec<[BindValue; 8]> = SmallVec::new();
        self.render_where_clause_into(&mut sql, &mut dummy_binds, obj.len() + 1);
        Ok(sql)
    }

    /// Returns the DELETE (or soft delete) SQL that would be executed for this query.
    pub fn to_delete_sql(&self) -> String {
        let mut sql = String::with_capacity(128);
        use std::fmt::Write;

        if T::has_soft_delete() {
            let _ = write!(
                sql,
                "UPDATE {} SET {} = {}",
                T::table_name(),
                DB::quote_identifier("deleted_at"),
                DB::current_timestamp_fn()
            );
        } else {
            let _ = write!(sql, "DELETE FROM {}", T::table_name());
        }

        let mut dummy_binds: SmallVec<[BindValue; 8]> = SmallVec::new();
        self.render_where_clause_into(&mut sql, &mut dummy_binds, 1);
        sql
    }

    // Optimized version that writes to buffer
    #[inline(always)]
    fn render_where_clause_into(
        &self,
        sql: &mut String,
        binds: &mut SmallVec<[BindValue; 8]>,
        start_index: usize,
    ) {
        let mut idx = start_index;
        let mut first_clause = true;
        use std::fmt::Write;

        // Helper to handle AND prefix
        let mut append_and = |sql: &mut String| {
            if first_clause {
                sql.push_str(" WHERE ");
                first_clause = false;
            } else {
                sql.push_str(" AND ");
            }
        };

        for filter in &self.filters {
            match filter {
                FilterExpr::Raw(condition) => {
                    append_and(sql);
                    sql.push_str(condition);
                }
                FilterExpr::Compare { column, op, values } => {
                    if op.is_in() {
                        if values.is_empty() {
                            append_and(sql);
                            sql.push_str("1=0");
                            continue;
                        }
                        append_and(sql);
                        let _ = write!(sql, "{} IN (", DB::quote_identifier(column.as_str()));
                        let placeholders = crate::cached_placeholders_from::<DB>(idx, values.len());
                        sql.push_str(placeholders);
                        sql.push(')');
                        idx = idx.saturating_add(values.len());
                        for v in values {
                            binds.push(v.clone());
                        }
                    } else {
                        append_and(sql);
                        let _ = write!(
                            sql,
                            "{} {} {}",
                            DB::quote_identifier(column.as_str()),
                            op.as_str(),
                            DB::placeholder(idx)
                        );
                        idx += 1;
                        if let Some(v) = values.first() {
                            binds.push(v.clone());
                        }
                    }
                }
                FilterExpr::NullCheck { column, is_null } => {
                    append_and(sql);
                    if *is_null {
                        let _ = write!(sql, "{} IS NULL", DB::quote_identifier(column.as_str()));
                    } else {
                        let _ =
                            write!(sql, "{} IS NOT NULL", DB::quote_identifier(column.as_str()));
                    }
                }
            }
        }

        if T::has_soft_delete() && !self.include_deleted {
            append_and(sql);
            sql.push_str("deleted_at IS NULL");
        }
    }
}

impl<'a, T, DB> QueryBuilder<'a, T, DB>
where
    DB: SqlDialect,
    T: Model<DB>,
    for<'q> <DB as Database>::Arguments<'q>: IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
    DB::Connection: Send,
    T: Send,
    String: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    i64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    f64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    bool: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    Option<String>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    uuid::Uuid: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    chrono::DateTime<chrono::Utc>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
    fn ensure_safe_filters(&self) -> Result<(), sqlx::Error> {
        if self.unsafe_fast {
            return Ok(());
        }
        if self.has_raw_filter && !self.allow_unsafe {
            return Err(sqlx::Error::Protocol(
                "Refusing raw filter without allow_unsafe".to_string(),
            ));
        }
        Ok(())
    }

    /// Executes the query and returns a vector of results.
    ///
    /// This method will fetch all rows matching the criteria and then perform
    /// eager loading for any included relations.
    #[tracing::instrument(skip(self), fields(table = T::table_name()))]
    pub async fn all(mut self) -> Result<Vec<T>, sqlx::Error> {
        self.ensure_safe_filters()?;

        let mut sql = String::with_capacity(128);
        sql.push_str("SELECT * FROM ");
        sql.push_str(T::table_name());

        let mut where_binds: SmallVec<[BindValue; 8]> =
            SmallVec::with_capacity(self.estimate_bind_count());
        self.render_where_clause_into(&mut sql, &mut where_binds, 1);

        if let Some(limit) = self.limit {
            use std::fmt::Write;
            let _ = write!(sql, " LIMIT {}", limit);
        }

        if let Some(offset) = self.offset {
            use std::fmt::Write;
            let _ = write!(sql, " OFFSET {}", offset);
        }

        #[cfg(debug_assertions)]
        if !self.fast_path && tracing::enabled!(tracing::Level::DEBUG) {
            let filters = self.format_filters_for_log();
            tracing::debug!(
                operation = "select",
                sql = %sql,
                filters = %filters,
                "premix query"
            );
        }

        let start = Instant::now();
        let mut results: Vec<T> = match &mut self.executor {
            Executor::Pool(pool) => {
                let base = apply_persistent_query_as(sqlx::query_as::<DB, T>(&sql), self.prepared);
                let query = where_binds.into_iter().fold(base, bind_value_query_as);
                query.fetch_all(*pool).await?
            }
            Executor::Conn(conn) => {
                let base = apply_persistent_query_as(sqlx::query_as::<DB, T>(&sql), self.prepared);
                let query = where_binds.into_iter().fold(base, bind_value_query_as);
                query.fetch_all(&mut **conn).await?
            }
        };
        if !self.fast_path {
            record_query_metrics("select", T::table_name(), start.elapsed());
        }

        if self.ultra_fast {
            return Ok(results);
        }

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

    /// Executes the query and returns a stream of results.
    ///
    /// This is useful for processing large result sets without loading them all into memory.
    #[tracing::instrument(skip(self), fields(table = T::table_name()))]
    pub fn stream(
        self,
    ) -> Result<futures_util::stream::BoxStream<'a, Result<T, sqlx::Error>>, sqlx::Error>
    where
        T: 'a,
    {
        self.ensure_safe_filters()?;

        let mut sql = String::with_capacity(128);
        sql.push_str("SELECT * FROM ");
        sql.push_str(T::table_name());

        let mut where_binds: SmallVec<[BindValue; 8]> =
            SmallVec::with_capacity(self.estimate_bind_count());
        self.render_where_clause_into(&mut sql, &mut where_binds, 1);

        if let Some(limit) = self.limit {
            use std::fmt::Write;
            let _ = write!(sql, " LIMIT {}", limit);
        }

        if let Some(offset) = self.offset {
            use std::fmt::Write;
            let _ = write!(sql, " OFFSET {}", offset);
        }

        #[cfg(debug_assertions)]
        if !self.fast_path && tracing::enabled!(tracing::Level::DEBUG) {
            let filters = self.format_filters_for_log();
            tracing::debug!(
                operation = "stream",
                sql = %sql,
                filters = %filters,
                "premix query"
            );
        }

        let executor = self.executor;
        Ok(Box::pin(async_stream::try_stream! {
            let mut query = apply_persistent_query_as(sqlx::query_as::<DB, T>(&sql), self.prepared);
            for bind in where_binds {
                query = bind_value_query_as(query, bind);
            }
            let mut s = executor.fetch_stream(query);
            while let Some(row) = s.next().await {
                yield row?;
            }
        }))
    }

    /// Executes a bulk update based on the current filters.
    ///
    /// # Errors
    /// Returns an error if no filters are provided (unless `allow_unsafe` is used),
    /// or if the values cannot be mapped to the database.
    #[inline(never)]
    #[tracing::instrument(skip(self, values), fields(table = T::table_name()))]
    pub async fn update(mut self, values: serde_json::Value) -> Result<u64, sqlx::Error>
    where
        String: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        i64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        f64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        bool: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Option<String>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        uuid::Uuid: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        chrono::DateTime<chrono::Utc>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        self.ensure_safe_filters()?;
        if self.filters.is_empty() && !self.allow_unsafe && !self.unsafe_fast {
            return Err(sqlx::Error::Protocol(
                "Refusing bulk update without filters".to_string(),
            ));
        }
        let obj = values.as_object().ok_or_else(|| {
            sqlx::Error::Protocol("Bulk update requires a JSON object".to_string())
        })?;

        let mut sql = String::with_capacity(256);
        use std::fmt::Write;
        let _ = write!(sql, "UPDATE {} SET ", T::table_name());

        let mut i = 1;
        let mut first = true;
        for k in obj.keys() {
            if !first {
                sql.push_str(", ");
            }
            let p = DB::placeholder(i);
            let _ = write!(sql, "{} = {}", DB::quote_identifier(k), p);
            i += 1;
            first = false;
        }

        let mut where_binds: SmallVec<[BindValue; 8]> =
            SmallVec::with_capacity(self.estimate_bind_count());
        self.render_where_clause_into(&mut sql, &mut where_binds, obj.len() + 1);

        if !self.fast_path && tracing::enabled!(tracing::Level::DEBUG) {
            let filters = self.format_filters_for_log();
            tracing::debug!(
                operation = "bulk_update",
                sql = %sql,
                filters = %filters,
                "premix query"
            );
        }
        let mut query = apply_persistent_query(sqlx::query::<DB>(&sql), self.prepared);
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
        for bind in where_binds {
            query = bind_value_query(query, bind);
        }

        let start = Instant::now();
        let result = match &mut self.executor {
            Executor::Pool(pool) => {
                let res = query.execute(*pool).await?;
                Ok(DB::rows_affected(&res))
            }
            Executor::Conn(conn) => {
                let res = query.execute(&mut **conn).await?;
                Ok(DB::rows_affected(&res))
            }
        };
        if !self.fast_path {
            record_query_metrics("bulk_update", T::table_name(), start.elapsed());
        }
        result
    }

    /// Executes a bulk update based on the current filters. Alias for [`update`].
    pub async fn update_all(self, values: serde_json::Value) -> Result<u64, sqlx::Error>
    where
        String: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        i64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        f64: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        bool: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Option<String>: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        self.update(values).await
    }

    /// Executes a bulk delete based on the current filters.
    #[tracing::instrument(skip(self), fields(table = T::table_name()))]
    pub async fn delete(mut self) -> Result<u64, sqlx::Error> {
        self.ensure_safe_filters()?;
        if self.filters.is_empty() && !self.allow_unsafe && !self.unsafe_fast {
            return Err(sqlx::Error::Protocol(
                "Refusing bulk delete without filters".to_string(),
            ));
        }

        let mut sql = String::with_capacity(128);
        use std::fmt::Write;

        if T::has_soft_delete() {
            let _ = write!(
                sql,
                "UPDATE {} SET {} = {}",
                T::table_name(),
                DB::quote_identifier("deleted_at"),
                DB::current_timestamp_fn()
            );
        } else {
            let _ = write!(sql, "DELETE FROM {}", T::table_name());
        }

        let mut where_binds: SmallVec<[BindValue; 8]> =
            SmallVec::with_capacity(self.estimate_bind_count());
        self.render_where_clause_into(&mut sql, &mut where_binds, 1);

        if !self.fast_path && tracing::enabled!(tracing::Level::DEBUG) {
            let filters = self.format_filters_for_log();
            tracing::debug!(
                operation = "bulk_delete",
                sql = %sql,
                filters = %filters,
                "premix query"
            );
        }
        let start = Instant::now();
        let result = match &mut self.executor {
            Executor::Pool(pool) => {
                let base = apply_persistent_query(sqlx::query::<DB>(&sql), self.prepared);
                let query = where_binds.into_iter().fold(base, bind_value_query);
                let res = query.execute(*pool).await?;
                Ok(DB::rows_affected(&res))
            }
            Executor::Conn(conn) => {
                let base = apply_persistent_query(sqlx::query::<DB>(&sql), self.prepared);
                let query = where_binds.into_iter().fold(base, bind_value_query);
                let res = query.execute(&mut **conn).await?;
                Ok(DB::rows_affected(&res))
            }
        };
        if !self.fast_path {
            record_query_metrics("bulk_delete", T::table_name(), start.elapsed());
        }
        result
    }

    /// Executes a bulk delete based on the current filters. Alias for [`delete`].
    pub async fn delete_all(self) -> Result<u64, sqlx::Error> {
        self.delete().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Sqlite;

    struct DummyModel;

    impl Model<Sqlite> for DummyModel {
        fn table_name() -> &'static str {
            "users"
        }
        fn create_table_sql() -> String {
            String::new()
        }
        fn list_columns() -> Vec<String> {
            vec!["id".to_string()]
        }
        async fn save<'a, E>(&'a mut self, _e: E) -> Result<(), sqlx::Error>
        where
            E: crate::executor::IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        async fn update<'a, E>(
            &'a mut self,
            _e: E,
        ) -> Result<crate::model::UpdateResult, sqlx::Error>
        where
            E: crate::executor::IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(crate::model::UpdateResult::Success)
        }
        async fn delete<'a, E>(&'a mut self, _e: E) -> Result<(), sqlx::Error>
        where
            E: crate::executor::IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(())
        }
        fn has_soft_delete() -> bool {
            false
        }
        async fn find_by_id<'a, E>(_e: E, _id: i32) -> Result<Option<Self>, sqlx::Error>
        where
            E: crate::executor::IntoExecutor<'a, DB = Sqlite>,
        {
            Ok(None)
        }
    }

    // Dummy FromRow implementation for Sqlite
    impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for DummyModel {
        fn from_row(_row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
            Ok(DummyModel)
        }
    }

    #[tokio::test]
    async fn test_sql_injection_mitigation() {
        let pool = sqlx::Pool::<Sqlite>::connect_lazy("sqlite::memory:").unwrap();
        let qb = DummyModel::find_in_pool(&pool);

        // Malicious column name
        let qb = qb.filter_eq("id; DROP TABLE users; --", 1);
        let sql = qb.to_sql();
        println!("SQL select: {}", sql);

        // The column name should be quoted
        assert!(sql.contains("`id; DROP TABLE users; --` = ?"));
        assert!(sql.contains("SELECT * FROM users WHERE"));
    }

    #[tokio::test]
    async fn test_to_update_sql_quoting() {
        let pool = sqlx::Pool::<Sqlite>::connect_lazy("sqlite::memory:").unwrap();
        let qb = DummyModel::find_in_pool(&pool).filter_eq("id", 1);

        let values = serde_json::json!({
            "name; DROP TABLE users; --": "admin"
        });

        let sql = qb.to_update_sql(&values).unwrap();
        println!("SQL update: {}", sql);
        assert!(sql.contains("`name; DROP TABLE users; --` = ?"));
    }

    #[tokio::test]
    async fn test_stream_api() {
        use sqlx::Connection;
        let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&mut conn)
            .await
            .unwrap();
        sqlx::query("INSERT INTO users (id, name) VALUES (1, 'alice'), (2, 'bob')")
            .execute(&mut conn)
            .await
            .unwrap();

        // Use find_in_tx which is standard in Premix
        let qb = DummyModel::find_in_tx(&mut conn);

        let mut stream = qb.stream().unwrap();
        let mut count = 0;
        while let Some(row) = stream.next().await {
            let _: DummyModel = row.unwrap();
            count += 1;
        }
        assert_eq!(count, 2);
    }
}
