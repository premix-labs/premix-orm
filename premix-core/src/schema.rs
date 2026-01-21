use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "postgres")]
use sqlx::PgPool;
use sqlx::SqlitePool;

/// Metadata about a database column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaColumn {
    /// The name of the column.
    pub name: String,
    /// The SQL type of the column (e.g., "INTEGER", "TEXT").
    pub sql_type: String,
    /// Whether the column can contain NULL values.
    pub nullable: bool,
    /// Whether the column is part of the Primary Key.
    pub primary_key: bool,
}

impl SchemaColumn {
    fn normalized_type(&self) -> String {
        normalize_sql_type(&self.sql_type)
    }
}

/// Metadata about a database index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIndex {
    /// The name of the index.
    pub name: String,
    /// The columns included in the index.
    pub columns: Vec<String>,
    /// Whether the index is UNIQUE.
    pub unique: bool,
}

/// Metadata about a foreign key relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaForeignKey {
    /// The column in the current table.
    pub column: String,
    /// The table being referenced.
    pub ref_table: String,
    /// The column being referenced in the target table.
    pub ref_column: String,
}

/// Metadata about a database table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaTable {
    /// The name of the table.
    pub name: String,
    /// The columns in the table.
    pub columns: Vec<SchemaColumn>,
    /// The indexes on the table.
    pub indexes: Vec<SchemaIndex>,
    /// The foreign keys in the table.
    pub foreign_keys: Vec<SchemaForeignKey>,
    /// The original CREATE TABLE SQL (if available).
    pub create_sql: Option<String>,
}

impl SchemaTable {
    /// Returns a column by name if it exists in the table.
    pub fn column(&self, name: &str) -> Option<&SchemaColumn> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Generates a `CREATE TABLE` SQL statement for this table.
    pub fn to_create_sql(&self) -> String {
        if let Some(sql) = &self.create_sql {
            return sql.clone();
        }

        let mut cols = Vec::new();
        for col in &self.columns {
            if col.primary_key {
                cols.push(format!("{} INTEGER PRIMARY KEY", col.name));
                continue;
            }
            let mut def = format!("{} {}", col.name, col.sql_type);
            if !col.nullable {
                def.push_str(" NOT NULL");
            }
            cols.push(def);
        }

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            self.name,
            cols.join(", ")
        )
    }
}

/// A trait for models that can provide their own schema metadata.
pub trait ModelSchema {
    /// Returns the schema metadata for this model.
    fn schema() -> SchemaTable;
}

/// Helper macro to collect schema metadata from multiple models.
#[macro_export]
macro_rules! schema_models {
    ($($model:ty),+ $(,)?) => {
        vec![$(<$model as $crate::schema::ModelSchema>::schema()),+]
    };
}

/// Represents a change in a column (addition or removal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDiff {
    /// The table containing the column.
    pub table: String,
    /// The name of the column.
    pub column: String,
    /// The SQL type of the column.
    pub sql_type: Option<String>,
}

/// Represents a mismatch in column types between models and database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnTypeDiff {
    /// The table containing the column.
    pub table: String,
    /// The name of the column.
    pub column: String,
    /// The SQL type expected by the model.
    pub expected: String,
    /// The actual SQL type found in the database.
    pub actual: String,
}

/// Represents a mismatch in column nullability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnNullabilityDiff {
    /// The table containing the column.
    pub table: String,
    /// The name of the column.
    pub column: String,
    /// Whether the model expects the column to be nullable.
    pub expected_nullable: bool,
    /// Whether the column is actually nullable in the database.
    pub actual_nullable: bool,
}

/// Represents a mismatch in Primary Key status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnPrimaryKeyDiff {
    /// The table containing the column.
    pub table: String,
    /// The name of the column.
    pub column: String,
    /// Whether the model expects the column to be a Primary Key.
    pub expected_primary_key: bool,
    /// Whether the column is actually a Primary Key in the database.
    pub actual_primary_key: bool,
}

/// Represents the differences between two database schemas.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaDiff {
    /// Tables expected but missing in the actual database.
    pub missing_tables: Vec<String>,
    /// Tables present in the database but not in the models.
    pub extra_tables: Vec<String>,
    /// Columns missing in existing tables.
    pub missing_columns: Vec<ColumnDiff>,
    /// Columns present in the database but not in the models.
    pub extra_columns: Vec<ColumnDiff>,
    /// Columns with different types than expected.
    pub type_mismatches: Vec<ColumnTypeDiff>,
    /// Columns with different nullability than expected.
    pub nullability_mismatches: Vec<ColumnNullabilityDiff>,
    /// Columns with different Primary Key status than expected.
    pub primary_key_mismatches: Vec<ColumnPrimaryKeyDiff>,
    /// Indexes missing in the actual database.
    pub missing_indexes: Vec<(String, SchemaIndex)>,
    /// Indexes present in the database but not in the models.
    pub extra_indexes: Vec<(String, SchemaIndex)>,
    /// Foreign keys missing in the actual database.
    pub missing_foreign_keys: Vec<(String, SchemaForeignKey)>,
    /// Foreign keys present in the database but not in the models.
    pub extra_foreign_keys: Vec<(String, SchemaForeignKey)>,
}

impl SchemaDiff {
    /// Returns true if there are no differences.
    pub fn is_empty(&self) -> bool {
        self.missing_tables.is_empty()
            && self.extra_tables.is_empty()
            && self.missing_columns.is_empty()
            && self.extra_columns.is_empty()
            && self.type_mismatches.is_empty()
            && self.nullability_mismatches.is_empty()
            && self.primary_key_mismatches.is_empty()
            && self.missing_indexes.is_empty()
            && self.extra_indexes.is_empty()
            && self.missing_foreign_keys.is_empty()
            && self.extra_foreign_keys.is_empty()
    }
}

/// Formats a [`SchemaDiff`] into a human-readable summary.
pub fn format_schema_diff_summary(diff: &SchemaDiff) -> String {
    if diff.is_empty() {
        return "Schema diff: no changes".to_string();
    }

    let mut lines = Vec::new();
    lines.push("Schema diff summary:".to_string());
    lines.push(format!("  missing tables: {}", diff.missing_tables.len()));
    lines.push(format!("  extra tables: {}", diff.extra_tables.len()));
    lines.push(format!("  missing columns: {}", diff.missing_columns.len()));
    lines.push(format!("  extra columns: {}", diff.extra_columns.len()));
    lines.push(format!("  type mismatches: {}", diff.type_mismatches.len()));
    lines.push(format!(
        "  nullability mismatches: {}",
        diff.nullability_mismatches.len()
    ));
    lines.push(format!(
        "  primary key mismatches: {}",
        diff.primary_key_mismatches.len()
    ));
    lines.push(format!("  missing indexes: {}", diff.missing_indexes.len()));
    lines.push(format!("  extra indexes: {}", diff.extra_indexes.len()));
    lines.push(format!(
        "  missing foreign keys: {}",
        diff.missing_foreign_keys.len()
    ));
    lines.push(format!(
        "  extra foreign keys: {}",
        diff.extra_foreign_keys.len()
    ));

    if !diff.missing_tables.is_empty() {
        lines.push(format!(
            "  missing tables list: {}",
            diff.missing_tables.join(", ")
        ));
    }
    if !diff.extra_tables.is_empty() {
        lines.push(format!(
            "  extra tables list: {}",
            diff.extra_tables.join(", ")
        ));
    }

    lines.join("\n")
}

/// Introspects the schema of a SQLite database.
pub async fn introspect_sqlite_schema(pool: &SqlitePool) -> Result<Vec<SchemaTable>, sqlx::Error> {
    let table_names: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_premix_migrations' ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for name in table_names {
        let pragma_sql = format!("PRAGMA table_info({})", name);
        let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
            sqlx::query_as(&pragma_sql).fetch_all(pool).await?;

        if rows.is_empty() {
            continue;
        }

        let columns = rows
            .into_iter()
            .map(|(_cid, col_name, col_type, notnull, _default, pk)| {
                let is_pk = pk > 0;
                SchemaColumn {
                    name: col_name,
                    sql_type: col_type,
                    nullable: !is_pk && notnull == 0,
                    primary_key: is_pk,
                }
            })
            .collect();

        let indexes = introspect_sqlite_indexes(pool, &name).await?;
        let foreign_keys = introspect_sqlite_foreign_keys(pool, &name).await?;

        tables.push(SchemaTable {
            name,
            columns,
            indexes,
            foreign_keys,
            create_sql: None,
        });
    }

    Ok(tables)
}

#[cfg(feature = "postgres")]
/// Introspects the schema of a PostgreSQL database.
pub async fn introspect_postgres_schema(pool: &PgPool) -> Result<Vec<SchemaTable>, sqlx::Error> {
    let table_names: Vec<String> = sqlx::query_scalar(
        "SELECT table_name FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE' AND table_name != '_premix_migrations' ORDER BY table_name",
    )
    .fetch_all(pool)
    .await?;

    let mut tables = Vec::new();
    for name in table_names {
        let pk_cols: Vec<String> = sqlx::query_scalar(
            "SELECT a.attname FROM pg_index i JOIN pg_attribute a ON a.attrelid=i.indrelid AND a.attnum = ANY(i.indkey) WHERE i.indrelid=$1::regclass AND i.indisprimary",
        )
        .bind(&name)
        .fetch_all(pool)
        .await?;
        let pk_set: BTreeSet<String> = pk_cols.into_iter().collect();

        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT column_name, data_type, udt_name, is_nullable FROM information_schema.columns WHERE table_schema='public' AND table_name=$1 ORDER BY ordinal_position",
        )
        .bind(&name)
        .fetch_all(pool)
        .await?;

        if rows.is_empty() {
            continue;
        }

        let columns = rows
            .into_iter()
            .map(|(col_name, data_type, udt_name, is_nullable)| {
                let is_pk = pk_set.contains(&col_name);
                let sql_type = if data_type.eq_ignore_ascii_case("ARRAY") {
                    let base = udt_name.trim_start_matches('_');
                    format!("{}[]", base)
                } else if data_type.eq_ignore_ascii_case("USER-DEFINED") {
                    udt_name
                } else {
                    data_type
                };
                SchemaColumn {
                    name: col_name,
                    sql_type,
                    nullable: !is_pk && is_nullable.eq_ignore_ascii_case("YES"),
                    primary_key: is_pk,
                }
            })
            .collect();

        let indexes = introspect_postgres_indexes(pool, &name).await?;
        let foreign_keys = introspect_postgres_foreign_keys(pool, &name).await?;

        tables.push(SchemaTable {
            name,
            columns,
            indexes,
            foreign_keys,
            create_sql: None,
        });
    }

    Ok(tables)
}

/// Compares the actual SQLite schema with an expected list of tables.
pub async fn diff_sqlite_schema(
    pool: &SqlitePool,
    expected: &[SchemaTable],
) -> Result<SchemaDiff, sqlx::Error> {
    let actual = introspect_sqlite_schema(pool).await?;
    Ok(diff_schema(expected, &actual))
}

/// Compares the actual PostgreSQL schema with an expected list of tables.
#[cfg(feature = "postgres")]
pub async fn diff_postgres_schema(
    pool: &PgPool,
    expected: &[SchemaTable],
) -> Result<SchemaDiff, sqlx::Error> {
    let actual = introspect_postgres_schema(pool).await?;
    Ok(diff_schema(expected, &actual))
}

/// Calculates the difference between two sets of table metadata.
pub fn diff_schema(expected: &[SchemaTable], actual: &[SchemaTable]) -> SchemaDiff {
    let mut diff = SchemaDiff::default();

    let expected_map: BTreeMap<_, _> = expected.iter().map(|t| (&t.name, t)).collect();
    let actual_map: BTreeMap<_, _> = actual.iter().map(|t| (&t.name, t)).collect();

    for name in expected_map.keys() {
        if !actual_map.contains_key(name) {
            diff.missing_tables.push((*name).to_string());
        }
    }
    for name in actual_map.keys() {
        if !expected_map.contains_key(name) {
            diff.extra_tables.push((*name).to_string());
        }
    }

    for (name, expected_table) in &expected_map {
        let Some(actual_table) = actual_map.get(name) else {
            continue;
        };

        let expected_cols: BTreeMap<_, _> = expected_table
            .columns
            .iter()
            .map(|c| (&c.name, c))
            .collect();
        let actual_cols: BTreeMap<_, _> =
            actual_table.columns.iter().map(|c| (&c.name, c)).collect();

        for col in expected_cols.keys() {
            if !actual_cols.contains_key(col) {
                let sql_type = expected_cols.get(col).map(|c| c.normalized_type());
                diff.missing_columns.push(ColumnDiff {
                    table: (*name).to_string(),
                    column: (*col).to_string(),
                    sql_type,
                });
            }
        }
        for col in actual_cols.keys() {
            if !expected_cols.contains_key(col) {
                let sql_type = actual_cols.get(col).map(|c| c.normalized_type());
                diff.extra_columns.push(ColumnDiff {
                    table: (*name).to_string(),
                    column: (*col).to_string(),
                    sql_type,
                });
            }
        }

        for (col_name, expected_col) in &expected_cols {
            let Some(actual_col) = actual_cols.get(col_name) else {
                continue;
            };

            let expected_type = expected_col.normalized_type();
            let actual_type = actual_col.normalized_type();
            if expected_type != actual_type {
                diff.type_mismatches.push(ColumnTypeDiff {
                    table: (*name).to_string(),
                    column: (*col_name).to_string(),
                    expected: expected_col.sql_type.clone(),
                    actual: actual_col.sql_type.clone(),
                });
            }

            if expected_col.nullable != actual_col.nullable {
                diff.nullability_mismatches.push(ColumnNullabilityDiff {
                    table: (*name).to_string(),
                    column: (*col_name).to_string(),
                    expected_nullable: expected_col.nullable,
                    actual_nullable: actual_col.nullable,
                });
            }

            if expected_col.primary_key != actual_col.primary_key {
                diff.primary_key_mismatches.push(ColumnPrimaryKeyDiff {
                    table: (*name).to_string(),
                    column: (*col_name).to_string(),
                    expected_primary_key: expected_col.primary_key,
                    actual_primary_key: actual_col.primary_key,
                });
            }
        }

        let expected_indexes = index_map(&expected_table.indexes);
        let actual_indexes = index_map(&actual_table.indexes);
        for key in expected_indexes.keys() {
            if !actual_indexes.contains_key(key) {
                if let Some(index) = expected_indexes.get(key) {
                    diff.missing_indexes
                        .push(((*name).to_string(), (*index).clone()));
                }
            }
        }
        for key in actual_indexes.keys() {
            if !expected_indexes.contains_key(key) {
                if let Some(index) = actual_indexes.get(key) {
                    diff.extra_indexes
                        .push(((*name).to_string(), (*index).clone()));
                }
            }
        }

        let expected_fks = foreign_key_map(&expected_table.foreign_keys);
        let actual_fks = foreign_key_map(&actual_table.foreign_keys);
        for key in expected_fks.keys() {
            if !actual_fks.contains_key(key) {
                if let Some(fk) = expected_fks.get(key) {
                    diff.missing_foreign_keys
                        .push(((*name).to_string(), (*fk).clone()));
                }
            }
        }
        for key in actual_fks.keys() {
            if !expected_fks.contains_key(key) {
                if let Some(fk) = actual_fks.get(key) {
                    diff.extra_foreign_keys
                        .push(((*name).to_string(), (*fk).clone()));
                }
            }
        }
    }

    diff.missing_tables.sort();
    diff.extra_tables.sort();

    diff
}

/// Generates SQLite migration SQL based on the provided schema differences.
pub fn sqlite_migration_sql(expected: &[SchemaTable], diff: &SchemaDiff) -> Vec<String> {
    let expected_map: BTreeMap<String, &SchemaTable> =
        expected.iter().map(|t| (t.name.clone(), t)).collect();
    let mut statements = Vec::new();

    for table in &diff.missing_tables {
        if let Some(schema) = expected_map.get(table) {
            statements.push(schema.to_create_sql());
            for index in &schema.indexes {
                statements.push(sqlite_create_index_sql(&schema.name, index));
            }
        } else {
            statements.push(format!("-- Missing schema for table {}", table));
        }
    }

    let mut missing_by_table: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for col in &diff.missing_columns {
        missing_by_table
            .entry(col.table.clone())
            .or_default()
            .insert(col.column.clone());
    }

    for (table, columns) in missing_by_table {
        let Some(schema) = expected_map.get(&table) else {
            continue;
        };
        for col_name in columns {
            let Some(col) = schema.column(&col_name) else {
                continue;
            };

            // Heuristic: Check for potential rename from extra columns
            // Simple check: Look for an extra column in the same table with the same type
            // that isn't already "claimed" (though here we just suggest)
            let potential_rename = diff.extra_columns.iter().find(|e| {
                e.table == table && e.sql_type.as_deref() == Some(&col.normalized_type())
            });

            if let Some(old) = potential_rename {
                statements.push(format!(
                    "-- SUGESTION: Potential rename from column '{}' to '{}'?",
                    old.column, col.name
                ));
            }

            if col.primary_key {
                statements.push(format!(
                    "-- TODO: add primary key column {}.{} manually",
                    table, col_name
                ));
                continue;
            }

            if !col.nullable {
                statements.push(format!(
                    "-- WARNING: Adding NOT NULL column '{}.{}' without a default value will fail if table contains rows.",
                    table, col.name
                ));
            }

            let mut stmt = format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table, col.name, col.sql_type
            );
            if !col.nullable {
                stmt.push_str(" NOT NULL");
            }
            statements.push(stmt);
        }
    }

    for mismatch in &diff.type_mismatches {
        statements.push(format!(
            "-- TODO: column type mismatch {}.{} (expected {}, actual {})",
            mismatch.table, mismatch.column, mismatch.expected, mismatch.actual
        ));
    }
    for mismatch in &diff.nullability_mismatches {
        statements.push(format!(
            "-- TODO: column nullability mismatch {}.{} (expected nullable {}, actual nullable {})",
            mismatch.table, mismatch.column, mismatch.expected_nullable, mismatch.actual_nullable
        ));
    }
    for mismatch in &diff.primary_key_mismatches {
        statements.push(format!(
            "-- TODO: column primary key mismatch {}.{} (expected pk {}, actual pk {})",
            mismatch.table,
            mismatch.column,
            mismatch.expected_primary_key,
            mismatch.actual_primary_key
        ));
    }
    for (table, index) in &diff.missing_indexes {
        statements.push(sqlite_create_index_sql(table, index));
    }
    for (table, index) in &diff.extra_indexes {
        statements.push(format!(
            "-- TODO: extra index {}.{} ({})",
            table,
            index.name,
            index.columns.join(", ")
        ));
    }
    for (table, fk) in &diff.missing_foreign_keys {
        statements.push(format!(
            "-- TODO: add foreign key {}.{} -> {}({}) (requires table rebuild)",
            table, fk.column, fk.ref_table, fk.ref_column
        ));
    }
    for (table, fk) in &diff.extra_foreign_keys {
        statements.push(format!(
            "-- TODO: extra foreign key {}.{} -> {}({})",
            table, fk.column, fk.ref_table, fk.ref_column
        ));
    }
    for extra in &diff.extra_columns {
        statements.push(format!(
            "-- TODO: extra column {}.{} not in models",
            extra.table, extra.column
        ));
    }
    for table in &diff.extra_tables {
        statements.push(format!("-- TODO: extra table {} not in models", table));
    }

    statements
}

fn normalize_sql_type(sql_type: &str) -> String {
    let t = sql_type.trim().to_lowercase();
    if t.is_empty() {
        return t;
    }
    if t.contains("int") || t.contains("serial") {
        return "integer".to_string();
    }
    if t.contains("char") || t.contains("text") || t.contains("clob") {
        return "text".to_string();
    }
    if t.contains("real")
        || t.contains("floa")
        || t.contains("doub")
        || t.contains("numeric")
        || t.contains("decimal")
    {
        return "real".to_string();
    }
    if t.contains("bool") {
        return "boolean".to_string();
    }
    if t.contains("time") || t.contains("date") || t.contains("uuid") || t.contains("json") {
        return "text".to_string();
    }
    t
}

#[cfg(feature = "postgres")]
/// Generates PostgreSQL migration SQL for a given schema difference.
pub fn postgres_migration_sql(expected: &[SchemaTable], diff: &SchemaDiff) -> Vec<String> {
    let expected_map: BTreeMap<String, &SchemaTable> =
        expected.iter().map(|t| (t.name.clone(), t)).collect();
    let mut statements = Vec::new();

    for table in &diff.missing_tables {
        if let Some(schema) = expected_map.get(table) {
            statements.push(schema.to_create_sql());
            for index in &schema.indexes {
                statements.push(postgres_create_index_sql(&schema.name, index));
            }
        } else {
            statements.push(format!("-- Missing schema for table {}", table));
        }
    }

    let mut missing_by_table: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for col in &diff.missing_columns {
        missing_by_table
            .entry(col.table.clone())
            .or_default()
            .insert(col.column.clone());
    }

    for (table, columns) in missing_by_table {
        let Some(schema) = expected_map.get(&table) else {
            continue;
        };
        for col_name in columns {
            let Some(col) = schema.column(&col_name) else {
                continue;
            };

            // Heuristic: Check for potential rename
            let potential_rename = diff.extra_columns.iter().find(|e| {
                e.table == table && e.sql_type.as_deref() == Some(&col.normalized_type())
            });

            if let Some(old) = potential_rename {
                statements.push(format!(
                    "-- SUGESTION: Potential rename from column '{}' to '{}'?",
                    old.column, col.name
                ));
            }

            if col.primary_key {
                statements.push(format!(
                    "-- TODO: add primary key column {}.{} manually",
                    table, col_name
                ));
                continue;
            }

            if !col.nullable {
                statements.push(format!(
                    "-- WARNING: Adding NOT NULL column '{}.{}' without a default value will fail if table contains rows.",
                    table, col.name
                ));
            }

            let mut stmt = format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table, col.name, col.sql_type
            );
            if !col.nullable {
                stmt.push_str(" NOT NULL");
            }
            statements.push(stmt);
        }
    }

    for mismatch in &diff.type_mismatches {
        statements.push(format!(
            "-- TODO: column type mismatch {}.{} (expected {}, actual {})",
            mismatch.table, mismatch.column, mismatch.expected, mismatch.actual
        ));
    }
    for mismatch in &diff.nullability_mismatches {
        statements.push(format!(
            "-- TODO: column nullability mismatch {}.{} (expected nullable {}, actual nullable {})",
            mismatch.table, mismatch.column, mismatch.expected_nullable, mismatch.actual_nullable
        ));
    }
    for mismatch in &diff.primary_key_mismatches {
        statements.push(format!(
            "-- TODO: column primary key mismatch {}.{} (expected pk {}, actual pk {})",
            mismatch.table,
            mismatch.column,
            mismatch.expected_primary_key,
            mismatch.actual_primary_key
        ));
    }
    for (table, index) in &diff.missing_indexes {
        statements.push(postgres_create_index_sql(table, index));
    }
    for (table, index) in &diff.extra_indexes {
        statements.push(format!(
            "-- TODO: extra index {}.{} ({})",
            table,
            index.name,
            index.columns.join(", ")
        ));
    }
    for (table, fk) in &diff.missing_foreign_keys {
        let fk_name = format!("fk_{}_{}", table, fk.column);
        statements.push(format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}({})",
            table, fk_name, fk.column, fk.ref_table, fk.ref_column
        ));
    }
    for (table, fk) in &diff.extra_foreign_keys {
        statements.push(format!(
            "-- TODO: extra foreign key {}.{} -> {}({})",
            table, fk.column, fk.ref_table, fk.ref_column
        ));
    }
    for extra in &diff.extra_columns {
        statements.push(format!(
            "-- TODO: extra column {}.{} not in models",
            extra.table, extra.column
        ));
    }
    for table in &diff.extra_tables {
        statements.push(format!("-- TODO: extra table {} not in models", table));
    }

    statements
}

fn index_key(index: &SchemaIndex) -> (String, String, bool) {
    let name = index.name.clone();
    let cols = index.columns.join(",");
    (name, cols, index.unique)
}

fn index_map(indexes: &[SchemaIndex]) -> BTreeMap<(String, String, bool), &SchemaIndex> {
    indexes.iter().map(|i| (index_key(i), i)).collect()
}

fn foreign_key_key(fk: &SchemaForeignKey) -> (String, String, String) {
    (
        fk.column.clone(),
        fk.ref_table.clone(),
        fk.ref_column.clone(),
    )
}

fn foreign_key_map(
    fks: &[SchemaForeignKey],
) -> BTreeMap<(String, String, String), &SchemaForeignKey> {
    fks.iter().map(|f| (foreign_key_key(f), f)).collect()
}

async fn introspect_sqlite_indexes(
    pool: &SqlitePool,
    table: &str,
) -> Result<Vec<SchemaIndex>, sqlx::Error> {
    let sql = format!("PRAGMA index_list({})", table);
    let rows: Vec<(i64, String, i64, String, i64)> = sqlx::query_as(&sql).fetch_all(pool).await?;

    let mut indexes = Vec::new();
    for (_seq, name, unique, origin, _partial) in rows {
        if origin == "pk" || name.starts_with("sqlite_autoindex") {
            continue;
        }
        let info_sql = format!("PRAGMA index_info({})", name);
        let info_rows: Vec<(i64, i64, String)> = sqlx::query_as(&info_sql).fetch_all(pool).await?;
        let columns = info_rows.into_iter().map(|(_seq, _cid, col)| col).collect();
        indexes.push(SchemaIndex {
            name,
            columns,
            unique: unique != 0,
        });
    }
    Ok(indexes)
}

async fn introspect_sqlite_foreign_keys(
    pool: &SqlitePool,
    table: &str,
) -> Result<Vec<SchemaForeignKey>, sqlx::Error> {
    let sql = format!("PRAGMA foreign_key_list({})", table);
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, i64, String, String, String, String, String, String)> =
        sqlx::query_as(&sql).fetch_all(pool).await?;

    let mut fks = Vec::new();
    for (_id, _seq, ref_table, from, to, _on_update, _on_delete, _match) in rows {
        fks.push(SchemaForeignKey {
            column: from,
            ref_table,
            ref_column: to,
        });
    }
    Ok(fks)
}

fn sqlite_create_index_sql(table: &str, index: &SchemaIndex) -> String {
    let unique = if index.unique { "UNIQUE " } else { "" };
    let name = if index.name.is_empty() {
        format!("idx_{}_{}", table, index.columns.join("_"))
    } else {
        index.name.clone()
    };
    format!(
        "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
        unique,
        name,
        table,
        index.columns.join(", ")
    )
}

#[cfg(feature = "postgres")]
fn postgres_create_index_sql(table: &str, index: &SchemaIndex) -> String {
    let unique = if index.unique { "UNIQUE " } else { "" };
    let name = if index.name.is_empty() {
        format!("idx_{}_{}", table, index.columns.join("_"))
    } else {
        index.name.clone()
    };
    format!(
        "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
        unique,
        name,
        table,
        index.columns.join(", ")
    )
}

#[cfg(feature = "postgres")]
async fn introspect_postgres_indexes(
    pool: &PgPool,
    table: &str,
) -> Result<Vec<SchemaIndex>, sqlx::Error> {
    let rows: Vec<(String, bool, Vec<String>)> = sqlx::query_as(
        "SELECT i.relname AS index_name, ix.indisunique, array_agg(a.attname ORDER BY x.n) AS columns
         FROM pg_class t
         JOIN pg_index ix ON t.oid = ix.indrelid
         JOIN pg_class i ON i.oid = ix.indexrelid
         JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS x(attnum, n) ON true
         JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = x.attnum
         WHERE t.relname = $1 AND t.relkind = 'r' AND NOT ix.indisprimary
         GROUP BY i.relname, ix.indisunique
         ORDER BY i.relname",
    )
    .bind(table)
    .fetch_all(pool)
    .await?;

    let indexes = rows
        .into_iter()
        .map(|(name, unique, columns)| SchemaIndex {
            name,
            columns,
            unique,
        })
        .collect();
    Ok(indexes)
}

#[cfg(feature = "postgres")]
async fn introspect_postgres_foreign_keys(
    pool: &PgPool,
    table: &str,
) -> Result<Vec<SchemaForeignKey>, sqlx::Error> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT kcu.column_name, ccu.table_name, ccu.column_name
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
         JOIN information_schema.constraint_column_usage ccu
           ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema
         WHERE tc.constraint_type = 'FOREIGN KEY'
           AND tc.table_schema = 'public'
           AND tc.table_name = $1
         ORDER BY kcu.ordinal_position",
    )
    .bind(table)
    .fetch_all(pool)
    .await?;

    let fks = rows
        .into_iter()
        .map(|(column, ref_table, ref_column)| SchemaForeignKey {
            column,
            ref_table,
            ref_column,
        })
        .collect();

    Ok(fks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_introspect_and_diff_empty() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, deleted_at TEXT)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let expected = vec![SchemaTable {
            name: "users".to_string(),
            columns: vec![
                SchemaColumn {
                    name: "id".to_string(),
                    sql_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                },
                SchemaColumn {
                    name: "name".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                SchemaColumn {
                    name: "deleted_at".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: true,
                    primary_key: false,
                },
            ],
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            create_sql: None,
        }];

        let actual = introspect_sqlite_schema(&pool).await.unwrap();
        let diff = diff_schema(&expected, &actual);
        assert!(diff.is_empty());
    }

    #[tokio::test]
    async fn sqlite_diff_reports_missing_column() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let expected = vec![SchemaTable {
            name: "users".to_string(),
            columns: vec![
                SchemaColumn {
                    name: "id".to_string(),
                    sql_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                },
                SchemaColumn {
                    name: "name".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                SchemaColumn {
                    name: "status".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: true,
                    primary_key: false,
                },
            ],
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            create_sql: None,
        }];

        let actual = introspect_sqlite_schema(&pool).await.unwrap();
        let diff = diff_schema(&expected, &actual);
        assert_eq!(diff.missing_columns.len(), 1);

        let summary = format_schema_diff_summary(&diff);
        assert!(summary.contains("missing columns: 1"));

        let sql = sqlite_migration_sql(&expected, &diff);
        assert!(
            sql.iter()
                .any(|stmt| stmt.contains("ALTER TABLE users ADD COLUMN status"))
        );
    }

    #[tokio::test]
    async fn sqlite_diff_reports_missing_index() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let expected = vec![SchemaTable {
            name: "users".to_string(),
            columns: vec![
                SchemaColumn {
                    name: "id".to_string(),
                    sql_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                },
                SchemaColumn {
                    name: "name".to_string(),
                    sql_type: "TEXT".to_string(),
                    nullable: false,
                    primary_key: false,
                },
            ],
            indexes: vec![SchemaIndex {
                name: "idx_users_name".to_string(),
                columns: vec!["name".to_string()],
                unique: false,
            }],
            foreign_keys: Vec::new(),
            create_sql: None,
        }];

        let actual = introspect_sqlite_schema(&pool).await.unwrap();
        let diff = diff_schema(&expected, &actual);
        assert_eq!(diff.missing_indexes.len(), 1);

        let sql = sqlite_migration_sql(&expected, &diff);
        assert!(
            sql.iter()
                .any(|stmt| stmt.contains("CREATE INDEX IF NOT EXISTS idx_users_name"))
        );
    }

    #[cfg(feature = "postgres")]
    fn pg_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:admin123@localhost:5432/premix_bench".to_string()
        })
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn postgres_introspect_and_diff() {
        let url = pg_url();
        let pool = match PgPool::connect(&url).await {
            Ok(pool) => pool,
            Err(_) => return,
        };

        sqlx::query("DROP TABLE IF EXISTS schema_posts")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DROP TABLE IF EXISTS schema_users")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("CREATE TABLE schema_users (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE schema_posts (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL, CONSTRAINT fk_schema_posts_user_id FOREIGN KEY (user_id) REFERENCES schema_users(id))",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("CREATE INDEX idx_schema_posts_user_id ON schema_posts (user_id)")
            .execute(&pool)
            .await
            .unwrap();

        let expected = vec![
            SchemaTable {
                name: "schema_posts".to_string(),
                columns: vec![
                    SchemaColumn {
                        name: "id".to_string(),
                        sql_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    SchemaColumn {
                        name: "user_id".to_string(),
                        sql_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                    SchemaColumn {
                        name: "title".to_string(),
                        sql_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                ],
                indexes: vec![SchemaIndex {
                    name: "idx_schema_posts_user_id".to_string(),
                    columns: vec!["user_id".to_string()],
                    unique: false,
                }],
                foreign_keys: vec![SchemaForeignKey {
                    column: "user_id".to_string(),
                    ref_table: "schema_users".to_string(),
                    ref_column: "id".to_string(),
                }],
                create_sql: None,
            },
            SchemaTable {
                name: "schema_users".to_string(),
                columns: vec![
                    SchemaColumn {
                        name: "id".to_string(),
                        sql_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    },
                    SchemaColumn {
                        name: "name".to_string(),
                        sql_type: "TEXT".to_string(),
                        nullable: false,
                        primary_key: false,
                    },
                ],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                create_sql: None,
            },
        ];

        let actual = introspect_postgres_schema(&pool).await.unwrap();
        let expected_names: BTreeSet<String> =
            expected.iter().map(|table| table.name.clone()).collect();
        let actual = actual
            .into_iter()
            .filter(|table| expected_names.contains(&table.name))
            .collect::<Vec<_>>();
        let diff = diff_schema(&expected, &actual);
        assert!(diff.is_empty(), "postgres schema diff: {diff:?}");

        sqlx::query("DROP TABLE IF EXISTS schema_posts")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DROP TABLE IF EXISTS schema_users")
            .execute(&pool)
            .await
            .unwrap();
    }
}
