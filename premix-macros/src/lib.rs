use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, Ident, LitStr, Token, parse_macro_input,
    punctuated::Punctuated,
};

mod relations;
mod static_query;

/// Compile-time query macro for true Zero-Overhead SQL generation.
///
/// This macro generates SQL at compile time, achieving 0% overhead compared to raw sqlx.
///
/// # Example
///
/// ```ignore
/// use premix_orm::prelude::*;
///
/// // FIND (SELECT + LIMIT 1)
/// let user = premix_query!(User, FIND, filter_eq("id", 1))
///     .fetch_one(&pool).await?;
///
/// // INSERT
/// let new_user = premix_query!(
///     User, INSERT,
///     set("name", "Bob"),
///     set("age", 30)
/// ).fetch_one(&pool).await?;
///
/// // UPDATE
/// premix_query!(
///     User, UPDATE,
///     set("age", 31),
///     filter_eq("name", "Bob")
/// ).execute(&pool).await?;
///
/// // DELETE
/// premix_query!(
///     User, DELETE,
///     filter_eq("id", 1)
/// ).execute(&pool).await?;
///
/// // UPDATE + RETURNING *
/// let updated = premix_query!(
///     User, UPDATE,
///     set("age", 32),
///     filter_eq("name", "Bob"),
///     returning_all()
/// ).fetch_one(&pool).await?;
/// ```
///
/// # Supported Operations
///
/// - `SELECT` - Generate SELECT query
/// - `FIND` - Generate SELECT + LIMIT 1 query
/// - `INSERT` - Generate INSERT query (with `set` assignments)
/// - `UPDATE` - Generate UPDATE query (with `set` assignments and filters)
/// - `DELETE` - Generate DELETE query (with filters)
///
/// # Supported Clauses
///
/// - `filter_eq/ne/gt/lt/gte/lte("col", val)` - WHERE clause conditions
/// - `set("col", val)` - SET/VALUES clause for INSERT/UPDATE
/// - `limit(N)` - LIMIT N
/// - `offset(N)` - OFFSET N
/// - `returning_all()` - Append `RETURNING *` for UPDATE/DELETE
#[proc_macro]
pub fn premix_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as static_query::StaticQueryInput);
    TokenStream::from(static_query::generate_static_query(input))
}

#[proc_macro_derive(Model, attributes(has_many, belongs_to, premix))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_model_impl(&input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn derive_model_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let impl_block = generate_generic_impl(input)?;
    let rel_block = relations::impl_relations(input)?;
    Ok(quote! {
        #impl_block
        #rel_block
    })
}

fn generate_generic_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let table_name = struct_name.to_string().to_lowercase() + "s";
    let custom_hooks = has_premix_flag(&input.attrs, "custom_hooks");
    let custom_validation = has_premix_flag(&input.attrs, "custom_validation");

    let all_fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            return Err(syn::Error::new_spanned(
                &data.fields,
                "Premix Model only supports structs with named fields",
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            input,
            "Premix Model only supports structs",
        ));
    };

    let mut db_fields = Vec::new();
    let mut ignored_field_idents = Vec::new();

    for field in all_fields {
        if is_ignored(field) {
            ignored_field_idents.push(field.ident.as_ref().unwrap());
        } else {
            db_fields.push(field);
        }
    }

    let field_idents: Vec<_> = db_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = db_fields.iter().map(|f| &f.ty).collect();
    let _field_indices: Vec<_> = (0..db_fields.len()).collect();
    let field_names: Vec<_> = field_idents.iter().map(|id| id.to_string()).collect();
    let field_names_no_id: Vec<_> = field_names
        .iter()
        .filter(|name| *name != "id")
        .cloned()
        .collect();
    let field_names_no_id_len = field_names_no_id.len();
    // all_columns_joined and no_id_columns_joined removed as part of Zero-Overhead optimization (replaced by concat!)

    // Prepare head/tail for concat! (to avoid trailing commas and handle separators)
    let all_cols_head = field_names.first().cloned().unwrap_or_default();
    let all_cols_tail: Vec<_> = field_names.iter().skip(1).cloned().collect();

    let no_id_cols_head = field_names_no_id.first().cloned().unwrap_or_default();
    let no_id_cols_tail: Vec<_> = field_names_no_id.iter().skip(1).cloned().collect();

    let field_idents_len = field_idents.len();
    let field_nullables: Vec<_> = db_fields.iter().map(|f| is_option_type(&f.ty)).collect();
    let field_primary_keys: Vec<_> = field_names.iter().map(|n| n == "id").collect();
    let field_sql_types: Vec<_> = db_fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap().to_string();
            sql_type_for_field(&name, &field.ty).to_string()
        })
        .collect();
    let field_sql_type_exprs: Vec<_> = db_fields
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap().to_string();
            sql_type_expr_for_field(&name, &field.ty)
        })
        .collect();
    let sensitive_field_literals: Vec<LitStr> = db_fields
        .iter()
        .filter(|f| is_sensitive(f))
        .map(|f| {
            LitStr::new(
                &f.ident.as_ref().unwrap().to_string(),
                f.ident.as_ref().unwrap().span(),
            )
        })
        .collect();

    let relation_meta = relations::collect_relation_metadata(input)?;
    let relation_names: Vec<LitStr> = relation_meta
        .iter()
        .map(|meta| LitStr::new(&meta.relation_name, proc_macro2::Span::call_site()))
        .collect();
    let eager_relation_names: Vec<LitStr> = relation_meta
        .iter()
        .filter(|meta| meta.eager)
        .map(|meta| LitStr::new(&meta.relation_name, proc_macro2::Span::call_site()))
        .collect();
    let eager_load_body = relations::generate_eager_load_body(input)?;
    let (index_specs, foreign_key_specs) = collect_schema_specs(all_fields, &table_name)?;
    let index_tokens: Vec<_> = index_specs
        .iter()
        .map(|spec| {
            let name = &spec.name;
            let columns = &spec.columns;
            let unique = spec.unique;
            quote! {
                premix_orm::schema::SchemaIndex {
                    name: #name.to_string(),
                    columns: vec![#(#columns.to_string()),*],
                    unique: #unique,
                }
            }
        })
        .collect();
    let foreign_key_tokens: Vec<_> = foreign_key_specs
        .iter()
        .map(|spec| {
            let column = &spec.column;
            let ref_table = &spec.ref_table;
            let ref_column = &spec.ref_column;
            quote! {
                premix_orm::schema::SchemaForeignKey {
                    column: #column.to_string(),
                    ref_table: #ref_table.to_string(),
                    ref_column: #ref_column.to_string(),
                }
            }
        })
        .collect();
    let has_version = field_names.contains(&"version".to_string());
    let has_soft_delete = field_names.contains(&"deleted_at".to_string());

    let save_update_block = if has_version {
        quote! {
            if self.id != 0 {
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let ver_p = <DB as premix_orm::SqlDialect>::placeholder(2 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                        table_name,
                        set_clause,
                        id_p,
                        ver_p
                    );
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "update",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = executor.execute(query).await?;
                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    static EXISTS_SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                    let exists_sql = EXISTS_SQL.get_or_init(|| {
                        let exists_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                        let mut exists_sql = String::with_capacity(table_name.len() + 32);
                        use ::std::fmt::Write;
                        let _ = write!(exists_sql, "SELECT id FROM {} WHERE id = {}", table_name, exists_p);
                        exists_sql
                    });
                    let exists_query =
                        premix_orm::sqlx::query_as::<DB, (i32,)>(exists_sql)
                            .persistent(true)
                            .bind(&self.id);
                    let exists = executor.fetch_optional(exists_query).await?;
                    if exists.is_some() {
                        return Err(premix_orm::sqlx::Error::Protocol(
                            "premix save failed: version conflict".into(),
                        ));
                    }
                } else {
                    self.version += 1;
                    self.after_save().await?;
                    return Ok(());
                }
            }
        }
    } else {
        quote! {
            if self.id != 0 {
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 32);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "UPDATE {} SET {} WHERE id = {}", table_name, set_clause, id_p);
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "update",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = executor.execute(query).await?;
                if <DB as premix_orm::SqlDialect>::rows_affected(&result) > 0 {
                    self.after_save().await?;
                    return Ok(());
                }
            }
        }
    };

    let save_fast_update_block = if has_version {
        quote! {
            if self.id != 0 {
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let ver_p = <DB as premix_orm::SqlDialect>::placeholder(2 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                        table_name,
                        set_clause,
                        id_p,
                        ver_p
                    );
                    sql
                });

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = executor.execute(query).await?;
                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    static EXISTS_SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                    let exists_sql = EXISTS_SQL.get_or_init(|| {
                        let exists_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                        let mut exists_sql = String::with_capacity(table_name.len() + 32);
                        use ::std::fmt::Write;
                        let _ = write!(exists_sql, "SELECT id FROM {} WHERE id = {}", table_name, exists_p);
                        exists_sql
                    });
                    let exists_query =
                        premix_orm::sqlx::query_as::<DB, (i32,)>(exists_sql)
                            .persistent(true)
                            .bind(&self.id);
                    let exists = executor.fetch_optional(exists_query).await?;
                    if exists.is_some() {
                        return Err(premix_orm::sqlx::Error::Protocol(
                            "premix save failed: version conflict".into(),
                        ));
                    }
                } else {
                    self.version += 1;
                    return Ok(());
                }
            }
        }
    } else {
        quote! {
            if self.id != 0 {
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 32);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "UPDATE {} SET {} WHERE id = {}", table_name, set_clause, id_p);
                    sql
                });

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = executor.execute(query).await?;
                if <DB as premix_orm::SqlDialect>::rows_affected(&result) > 0 {
                    return Ok(());
                }
            }
        }
    };

    let update_impl = if has_version {
        quote! {
            fn update<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let ver_p = <DB as premix_orm::SqlDialect>::placeholder(2 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                        table_name,
                        set_clause,
                        id_p,
                        ver_p
                    );
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "update",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = executor.execute(query).await?;

                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    static EXISTS_SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                    let exists_sql = EXISTS_SQL.get_or_init(|| {
                        let exists_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                        let mut exists_sql = String::with_capacity(table_name.len() + 32);
                        use ::std::fmt::Write;
                        let _ = write!(exists_sql, "SELECT id FROM {} WHERE id = {}", table_name, exists_p);
                        exists_sql
                    });
                    let exists_query =
                        premix_orm::sqlx::query_as::<DB, (i32,)>(exists_sql)
                            .persistent(true)
                            .bind(&self.id);
                    let exists = executor.fetch_optional(exists_query).await?;

                    if exists.is_none() {
                        Ok(premix_orm::UpdateResult::NotFound)
                    } else {
                        Ok(premix_orm::UpdateResult::VersionConflict)
                    }
                } else {
                    self.version += 1;
                    Ok(premix_orm::UpdateResult::Success)
                }
                }
            }

            fn update_fast<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let ver_p = <DB as premix_orm::SqlDialect>::placeholder(2 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                        table_name,
                        set_clause,
                        id_p,
                        ver_p
                    );
                    sql
                });

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = executor.execute(query).await?;

                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    static EXISTS_SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                    let exists_sql = EXISTS_SQL.get_or_init(|| {
                        let exists_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                        let mut exists_sql = String::with_capacity(table_name.len() + 32);
                        use ::std::fmt::Write;
                        let _ = write!(exists_sql, "SELECT id FROM {} WHERE id = {}", table_name, exists_p);
                        exists_sql
                    });
                    let exists_query =
                        premix_orm::sqlx::query_as::<DB, (i32,)>(exists_sql)
                            .persistent(true)
                            .bind(&self.id);
                    let exists = executor.fetch_optional(exists_query).await?;

                    if exists.is_none() {
                        Ok(premix_orm::UpdateResult::NotFound)
                    } else {
                        Ok(premix_orm::UpdateResult::VersionConflict)
                    }
                } else {
                    self.version += 1;
                    Ok(premix_orm::UpdateResult::Success)
                }
                }
            }
            fn update_ultra<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move { self.update_fast(executor).await }
            }
        }
    } else {
        quote! {
            fn update<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 32);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "UPDATE {} SET {} WHERE id = {}", table_name, set_clause, id_p);
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "update",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = executor.execute(query).await?;

                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    Ok(premix_orm::UpdateResult::NotFound)
                } else {
                    Ok(premix_orm::UpdateResult::Success)
                }
                }
            }

            fn update_fast<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let mut set_clause = String::with_capacity(#field_idents_len * 8);
                    let mut i = 1usize;
                    #(
                        if i > 1 {
                            set_clause.push_str(", ");
                        }
                        set_clause.push_str(#field_names);
                        set_clause.push_str(" = ");
                        set_clause.push_str(&<DB as premix_orm::SqlDialect>::placeholder(i));
                        i += 1;
                    )*
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1 + #field_idents_len);
                    let mut sql = String::with_capacity(set_clause.len() + table_name.len() + 32);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "UPDATE {} SET {} WHERE id = {}", table_name, set_clause, id_p);
                    sql
                });

                let mut query = premix_orm::sqlx::query::<DB>(sql).persistent(true)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = executor.execute(query).await?;

                if <DB as premix_orm::SqlDialect>::rows_affected(&result) == 0 {
                    Ok(premix_orm::UpdateResult::NotFound)
                } else {
                    Ok(premix_orm::UpdateResult::Success)
                }
                }
            }
            fn update_ultra<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<
                Output = Result<premix_orm::UpdateResult, premix_orm::sqlx::Error>,
            > + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move { self.update_fast(executor).await }
            }
        }
    };

    let delete_impl = if has_soft_delete {
        quote! {
            fn delete<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                    let mut sql = String::with_capacity(table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET deleted_at = {} WHERE id = {}",
                        table_name,
                        <DB as premix_orm::SqlDialect>::current_timestamp_fn(),
                        id_p
                    );
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "delete",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let query = premix_orm::sqlx::query::<DB>(sql).persistent(true).bind(&self.id);
                executor.execute(query).await?;

                self.deleted_at = Some("DELETED".to_string());
                Ok(())
                }
            }
            fn delete_fast<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                    let mut sql = String::with_capacity(table_name.len() + 64);
                    use ::std::fmt::Write;
                    let _ = write!(
                        sql,
                        "UPDATE {} SET deleted_at = {} WHERE id = {}",
                        table_name,
                        <DB as premix_orm::SqlDialect>::current_timestamp_fn(),
                        id_p
                    );
                    sql
                });

                let query = premix_orm::sqlx::query::<DB>(sql).persistent(true).bind(&self.id);
                executor.execute(query).await?;

                self.deleted_at = Some("DELETED".to_string());
                Ok(())
                }
            }
            fn delete_ultra<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move { self.delete_fast(executor).await }
            }
            fn has_soft_delete() -> bool { true }
        }
    } else {
        quote! {
            fn delete<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                    let mut sql = String::with_capacity(table_name.len() + 24);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "DELETE FROM {} WHERE id = {}", table_name, id_p);
                    sql
                });

                premix_orm::tracing::debug!(
                    operation = "delete",
                    table = table_name,
                    sql = %sql,
                    "premix query"
                );

                let query = premix_orm::sqlx::query::<DB>(sql).persistent(true).bind(&self.id);
                executor.execute(query).await?;

                Ok(())
                }
            }
            fn delete_fast<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                static SQL: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
                let sql = SQL.get_or_init(|| {
                    let id_p = <DB as premix_orm::SqlDialect>::placeholder(1);
                    let mut sql = String::with_capacity(table_name.len() + 24);
                    use ::std::fmt::Write;
                    let _ = write!(sql, "DELETE FROM {} WHERE id = {}", table_name, id_p);
                    sql
                });

                let query = premix_orm::sqlx::query::<DB>(sql).persistent(true).bind(&self.id);
                executor.execute(query).await?;

                Ok(())
                }
            }
            fn delete_ultra<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move { self.delete_fast(executor).await }
            }
            fn has_soft_delete() -> bool { false }
        }
    };

    let mut related_model_bounds = Vec::new();
    for field in all_fields {
        for attr in &field.attrs {
            if attr.path().is_ident("has_many")
                && let Ok(related_ident) = attr.parse_args::<syn::Ident>()
            {
                related_model_bounds.push(quote! { #related_ident: premix_orm::Model<DB> });
            } else if attr.path().is_ident("belongs_to")
                && let Ok(related_ident) = attr.parse_args::<syn::Ident>()
            {
                related_model_bounds.push(quote! { #related_ident: premix_orm::Model<DB> + Clone });
            }
        }
    }

    let hooks_impl = if custom_hooks {
        quote! {}
    } else {
        quote! {
            impl premix_orm::ModelHooks for #struct_name {}
        }
    };

    let validation_impl = if custom_validation {
        quote! {}
    } else {
        quote! {
            impl premix_orm::ModelValidation for #struct_name {}
        }
    };

    let col_consts: Vec<_> = field_names
        .iter()
        .zip(field_idents.iter())
        .map(|(name, ident)| {
            let const_name = syn::Ident::new(&ident.to_string().to_uppercase(), ident.span());
            quote! {
                pub const #const_name: &str = #name;
            }
        })
        .collect();

    let columns_mod_ident = syn::Ident::new(
        &format!("columns_{}", struct_name.to_string().to_lowercase()),
        struct_name.span(),
    );

    // Generic Implementation
    Ok(quote! {
        // Generate column constants
        #[allow(non_snake_case)]
        pub mod #columns_mod_ident {
             #( #col_consts )*
        }

        impl<'r, R> premix_orm::sqlx::FromRow<'r, R> for #struct_name
        where
            R: premix_orm::sqlx::Row,
            R::Database: premix_orm::sqlx::Database,
            #(
                #field_types: premix_orm::sqlx::Type<R::Database> + premix_orm::sqlx::Decode<'r, R::Database>,
            )*
            for<'c> &'c str: premix_orm::sqlx::ColumnIndex<R>,
        {
            fn from_row(row: &'r R) -> Result<Self, premix_orm::sqlx::Error> {
                use premix_orm::sqlx::Row;
                Ok(Self {
                    #(
                        #field_idents: row.try_get(#field_names)?,
                    )*
                    #(
                        #ignored_field_idents: None,
                    )*
                })
            }
        }


        impl<DB> premix_orm::Model<DB> for #struct_name
        where
            DB: premix_orm::SqlDialect,
            for<'c> &'c str: premix_orm::sqlx::ColumnIndex<DB::Row>,
            usize: premix_orm::sqlx::ColumnIndex<DB::Row>,
            for<'q> <DB as premix_orm::sqlx::Database>::Arguments<'q>: premix_orm::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_orm::sqlx::Database>::Connection: premix_orm::sqlx::Executor<'c, Database = DB>,
            i32: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB> + for<'r> premix_orm::sqlx::Decode<'r, DB>,
            i64: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB> + for<'r> premix_orm::sqlx::Decode<'r, DB>,
            String: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB> + for<'r> premix_orm::sqlx::Decode<'r, DB>,
            bool: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB> + for<'r> premix_orm::sqlx::Decode<'r, DB>,
            Option<String>: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB> + for<'r> premix_orm::sqlx::Decode<'r, DB>,
            #( #related_model_bounds, )*
        {
            fn table_name() -> &'static str {
                #table_name
            }

            fn create_table_sql() -> String {
                let mut cols = vec!["id ".to_string() + <DB as premix_orm::SqlDialect>::auto_increment_pk()];
                #(
                    if #field_names != "id" {
                        let sql_type = #field_sql_type_exprs;
                        cols.push(format!("{} {}", #field_names, sql_type));
                    }
                )*
                format!("CREATE TABLE IF NOT EXISTS {} ({})", #table_name, cols.join(", "))
            }

            fn list_columns() -> ::std::vec::Vec<::std::string::String> {
                vec![ #( #field_names.to_string() ),* ]
            }

            fn sensitive_fields() -> &'static [&'static str] {
                &[ #( #sensitive_field_literals ),* ]
            }

            fn relation_names() -> &'static [&'static str] {
                &[ #( #relation_names ),* ]
            }

            fn default_includes() -> &'static [&'static str] {
                &[ #( #eager_relation_names ),* ]
            }

            fn from_row_fast(row: &<DB as premix_orm::sqlx::Database>::Row) -> Result<Self, premix_orm::sqlx::Error>
            where
                usize: premix_orm::sqlx::ColumnIndex<<DB as premix_orm::sqlx::Database>::Row>,
                for<'c> &'c str: premix_orm::sqlx::ColumnIndex<<DB as premix_orm::sqlx::Database>::Row>,
            {
                use premix_orm::sqlx::Row;
                let mut idx: usize = 0;
                #(
                    let #field_idents = row.try_get(idx)?;
                    idx += 1;
                )*
                Ok(Self {
                    #( #field_idents, )*
                    #( #ignored_field_idents: None, )*
                })
            }

            fn save<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                use premix_orm::ModelHooks;
                self.before_save().await?;

                #save_update_block

                // CONSTANT column lists to avoid runtime joining/allocation
                // We use head/tail pattern to insert ", " separator without trailing comma
                const ALL_COLUMNS_LIST: &str = concat!(#all_cols_head, #( ", ", #all_cols_tail ),*);
                const NO_ID_COLUMNS_LIST: &str = concat!(#no_id_cols_head, #( ", ", #no_id_cols_tail ),*);

                let supports_returning = <DB as premix_orm::SqlDialect>::supports_returning();
                if supports_returning {
                    let sql = if self.id == 0 {
                        static INSERT_NO_ID_RETURNING_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_NO_ID_RETURNING_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_names_no_id_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
                                #table_name,
                                NO_ID_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    } else {
                        static INSERT_WITH_ID_RETURNING_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_WITH_ID_RETURNING_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_idents_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
                                #table_name,
                                ALL_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    };

                    premix_orm::tracing::debug!(
                        operation = "insert",
                        table = #table_name,
                        sql = %sql,
                        "premix query"
                    );

                    let mut query = premix_orm::sqlx::query_as::<DB, (i32,)>(sql.as_str())
                        .persistent(true);
                    #(
                        if #field_names != "id" {
                            query = query.bind(&self.#field_idents);
                        } else if self.id != 0 {
                            query = query.bind(&self.id);
                        }
                    )*

                    if let Some((id,)) = executor.fetch_optional(query).await? {
                        self.id = id;
                    }
                } else {
                    let sql = if self.id == 0 {
                        static INSERT_NO_ID_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_NO_ID_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_names_no_id_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                NO_ID_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    } else {
                        static INSERT_WITH_ID_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_WITH_ID_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_idents_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                ALL_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    };

                    premix_orm::tracing::debug!(
                        operation = "insert",
                        table = #table_name,
                        sql = %sql,
                        "premix query"
                    );

                    let mut query = premix_orm::sqlx::query::<DB>(sql.as_str()).persistent(true);
                    #(
                        if #field_names != "id" {
                            query = query.bind(&self.#field_idents);
                        } else if self.id != 0 {
                            query = query.bind(&self.id);
                        }
                    )*

                    let result = executor.execute(query).await?;
                    let last_id = <DB as premix_orm::SqlDialect>::last_insert_id(&result);
                    if last_id > 0 {
                        self.id = last_id as i32;
                    }
                }

                self.after_save().await?;
                Ok(())
                }
            }

            fn save_fast<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();

                #save_fast_update_block

                // CONSTANT column lists to avoid runtime joining/allocation
                // We use head/tail pattern to insert ", " separator without trailing comma
                const ALL_COLUMNS_LIST: &str = concat!(#all_cols_head, #( ", ", #all_cols_tail ),*);
                const NO_ID_COLUMNS_LIST: &str = concat!(#no_id_cols_head, #( ", ", #no_id_cols_tail ),*);

                let supports_returning = <DB as premix_orm::SqlDialect>::supports_returning();
                if supports_returning {
                    let sql = if self.id == 0 {
                        static INSERT_NO_ID_RETURNING_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_NO_ID_RETURNING_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_names_no_id_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
                                #table_name,
                                NO_ID_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    } else {
                        static INSERT_WITH_ID_RETURNING_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_WITH_ID_RETURNING_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_idents_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
                                #table_name,
                                ALL_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    };

                    let mut query = premix_orm::sqlx::query_as::<DB, (i32,)>(sql.as_str())
                        .persistent(true);
                    #(
                        if #field_names != "id" {
                            query = query.bind(&self.#field_idents);
                        } else if self.id != 0 {
                            query = query.bind(&self.id);
                        }
                    )*

                    if let Some((id,)) = executor.fetch_optional(query).await? {
                        self.id = id;
                    }
                } else {
                    let sql = if self.id == 0 {
                        static INSERT_NO_ID_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_NO_ID_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_names_no_id_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                NO_ID_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    } else {
                        static INSERT_WITH_ID_SQL: std::sync::OnceLock<String> =
                            std::sync::OnceLock::new();
                        INSERT_WITH_ID_SQL.get_or_init(|| {
                            let placeholders =
                                premix_orm::cached_placeholders::<DB>(#field_idents_len);
                            format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                ALL_COLUMNS_LIST,
                                placeholders
                            )
                        })
                    };

                    let mut query = premix_orm::sqlx::query::<DB>(sql.as_str()).persistent(true);
                    #(
                        if #field_names != "id" {
                            query = query.bind(&self.#field_idents);
                        } else if self.id != 0 {
                            query = query.bind(&self.id);
                        }
                    )*

                    let result = executor.execute(query).await?;
                    let last_id = <DB as premix_orm::SqlDialect>::last_insert_id(&result);
                    if last_id > 0 {
                        self.id = last_id as i32;
                    }
                }

                Ok(())
                }
            }
            fn save_ultra<'a, E>(
                &'a mut self,
                executor: E,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();

                // CONSTANT column lists to avoid runtime joining/allocation
                // We use head/tail pattern to insert ", " separator without trailing comma
                const ALL_COLUMNS_LIST: &str = concat!(#all_cols_head, #( ", ", #all_cols_tail ),*);
                const NO_ID_COLUMNS_LIST: &str = concat!(#no_id_cols_head, #( ", ", #no_id_cols_tail ),*);

                let column_list: &str = if self.id == 0 { NO_ID_COLUMNS_LIST } else { ALL_COLUMNS_LIST };

                // We still need to calculate placeholders at runtime because they depend on the count and DB dialect
                let count = if self.id == 0 { #field_names_no_id_len } else { #field_idents_len };
                let placeholders = premix_orm::cached_placeholders::<DB>(count);

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    #table_name,
                    column_list,
                    placeholders
                );

                let mut query = premix_orm::sqlx::query::<DB>(&sql).persistent(true);
                #(
                    if #field_names != "id" {
                        query = query.bind(&self.#field_idents);
                    } else if self.id != 0 {
                        query = query.bind(&self.id);
                    }
                )*

                let result = executor.execute(query).await?;
                let last_id = <DB as premix_orm::SqlDialect>::last_insert_id(&result);
                if last_id > 0 {
                    self.id = last_id as i32;
                }

                Ok(())
                }
            }

            #update_impl
            #delete_impl

            fn find_by_id<'a, E>(
                executor: E,
                id: i32,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<::std::option::Option<Self>, premix_orm::sqlx::Error>>
            + Send
            where
                E: premix_orm::IntoExecutor<'a, DB = DB>
            {
                async move {
                let mut executor = executor.into_executor();
                let p = <DB as premix_orm::SqlDialect>::placeholder(1);

                // Optimization: Pre-calculate the base SQL string
                let sql = if Self::has_soft_delete() {
                    format!("SELECT * FROM {} WHERE id = {} AND deleted_at IS NULL LIMIT 1", #table_name, p)
                } else {
                    format!("SELECT * FROM {} WHERE id = {} LIMIT 1", #table_name, p)
                };

                premix_orm::tracing::debug!(
                    operation = "select",
                    table = #table_name,
                    sql = %sql,
                    "premix query"
                );
                let query = premix_orm::sqlx::query_as::<DB, Self>(&sql)
                    .persistent(true)
                    .bind(id);
                executor.fetch_optional(query).await
                }
            }

            fn eager_load<'a>(
                models: &mut [Self],
                relation: &str,
                executor: premix_orm::Executor<'a, DB>,
            ) -> impl ::std::future::Future<Output = Result<(), premix_orm::sqlx::Error>> + Send
            {
                async move {
                    let mut executor = executor;
                    #eager_load_body
                }
            }
        }

        #hooks_impl
        #validation_impl

        impl premix_orm::ModelSchema for #struct_name {
            fn schema() -> premix_orm::schema::SchemaTable {
                let columns = vec![
                    #(
                        premix_orm::schema::SchemaColumn {
                            name: #field_names.to_string(),
                            sql_type: #field_sql_types.to_string(),
                            nullable: #field_nullables,
                            primary_key: #field_primary_keys,
                        }
                    ),*
                ];
                let indexes = vec![
                    #(#index_tokens),*
                ];
                let foreign_keys = vec![
                    #(#foreign_key_tokens),*
                ];
                premix_orm::schema::SchemaTable {
                    name: #table_name.to_string(),
                    columns,
                    indexes,
                    foreign_keys,
                    create_sql: None,
                }
            }
        }
    })
}

fn has_premix_field_flag(field: &Field, flag: &str) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("premix")
            && let Ok(meta) = attr.parse_args::<syn::Ident>()
            && meta == flag
        {
            return true;
        }
    }
    false
}

fn is_ignored(field: &Field) -> bool {
    has_premix_field_flag(field, "ignore")
}

fn is_sensitive(field: &Field) -> bool {
    has_premix_field_flag(field, "sensitive")
}

struct IndexSpec {
    name: String,
    columns: Vec<String>,
    unique: bool,
}

struct ForeignKeySpec {
    column: String,
    ref_table: String,
    ref_column: String,
}

fn collect_schema_specs(
    fields: &syn::punctuated::Punctuated<Field, Token![,]>,
    table_name: &str,
) -> syn::Result<(Vec<IndexSpec>, Vec<ForeignKeySpec>)> {
    let mut indexes = Vec::new();
    let mut foreign_keys = Vec::new();

    for field in fields {
        if is_ignored(field) {
            continue;
        }
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have an ident"))?
            .to_string();

        for attr in &field.attrs {
            if !attr.path().is_ident("premix") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("index") || meta.path.is_ident("unique") {
                    let unique = meta.path.is_ident("unique");
                    let mut name = None;
                    if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|nested| {
                            if nested.path.is_ident("name") {
                                let lit: LitStr = nested.value()?.parse()?;
                                name = Some(lit.value());
                                Ok(())
                            } else {
                                Err(nested.error("unsupported index option"))
                            }
                        })?;
                    }
                    let index_name =
                        name.unwrap_or_else(|| format!("idx_{}_{}", table_name, field_name));
                    indexes.push(IndexSpec {
                        name: index_name,
                        columns: vec![field_name.clone()],
                        unique,
                    });
                } else if meta.path.is_ident("foreign_key") {
                    let mut ref_table = None;
                    let mut ref_column = None;
                    meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("table") {
                            let lit: LitStr = nested.value()?.parse()?;
                            ref_table = Some(lit.value());
                            Ok(())
                        } else if nested.path.is_ident("column") {
                            let lit: LitStr = nested.value()?.parse()?;
                            ref_column = Some(lit.value());
                            Ok(())
                        } else {
                            Err(nested.error("unsupported foreign_key option"))
                        }
                    })?;

                    let ref_table = ref_table.ok_or_else(|| {
                        syn::Error::new_spanned(attr, "foreign_key requires table = \"...\"")
                    })?;
                    let ref_column = ref_column.unwrap_or_else(|| "id".to_string());
                    foreign_keys.push(ForeignKeySpec {
                        column: field_name.clone(),
                        ref_table,
                        ref_column,
                    });
                }
                Ok(())
            })?;
        }
    }

    Ok((indexes, foreign_keys))
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(path) = ty {
        if let Some(seg) = path.path.segments.last() {
            return seg.ident == "Option";
        }
    }
    false
}

fn has_premix_flag(attrs: &[Attribute], flag: &str) -> bool {
    for attr in attrs {
        if attr.path().is_ident("premix") {
            let args = attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated);
            if let Ok(args) = args {
                if args.iter().any(|ident| ident == flag) {
                    return true;
                }
            }
        }
    }
    false
}

fn type_name_for_field(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(path) = ty {
        let segment = path.path.segments.last()?;
        let ident = segment.ident.to_string();
        if ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in args.args.iter() {
                    if let syn::GenericArgument::Type(inner) = arg {
                        return type_name_for_field(inner);
                    }
                }
            }
            None
        } else if ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    if let Some(inner_ident) = type_name_for_field(inner) {
                        return Some(format!("Vec<{}>", inner_ident));
                    }
                }
            }
            Some("Vec".to_string())
        } else {
            Some(ident)
        }
    } else {
        None
    }
}

fn sql_type_for_field(name: &str, ty: &syn::Type) -> &'static str {
    let type_name = type_name_for_field(ty);
    match type_name.as_deref() {
        Some("i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "usize") => "INTEGER",
        Some("i64" | "u64") => "BIGINT",
        Some("f32" | "f64") => "REAL",
        Some("bool") => "BOOLEAN",
        Some("String" | "str") => "TEXT",
        Some("Uuid" | "DateTime" | "NaiveDateTime" | "NaiveDate") => "TEXT",
        Some("Vec<u8>") => "BLOB",
        _ => {
            if name == "id" || name.ends_with("_id") {
                "INTEGER"
            } else {
                "TEXT"
            }
        }
    }
}

fn sql_type_expr_for_field(name: &str, ty: &syn::Type) -> proc_macro2::TokenStream {
    let type_name = type_name_for_field(ty);
    match type_name.as_deref() {
        Some("i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "usize") => {
            quote! { <DB as premix_orm::SqlDialect>::int_type() }
        }
        Some("i64" | "u64") => quote! { <DB as premix_orm::SqlDialect>::bigint_type() },
        Some("f32" | "f64") => quote! { <DB as premix_orm::SqlDialect>::float_type() },
        Some("bool") => quote! { <DB as premix_orm::SqlDialect>::bool_type() },
        Some("String" | "str") => quote! { <DB as premix_orm::SqlDialect>::text_type() },
        Some("Uuid" | "DateTime" | "NaiveDateTime" | "NaiveDate") => {
            quote! { <DB as premix_orm::SqlDialect>::text_type() }
        }
        Some("Vec<u8>") => quote! { <DB as premix_orm::SqlDialect>::blob_type() },
        _ => {
            if name == "id" || name.ends_with("_id") {
                quote! { <DB as premix_orm::SqlDialect>::int_type() }
            } else {
                quote! { <DB as premix_orm::SqlDialect>::text_type() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn generate_generic_impl_includes_table_and_columns() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
                version: i32,
                deleted_at: Option<String>,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("CREATE TABLE IF NOT EXISTS"));
        assert!(tokens.contains("users"));
        assert!(tokens.contains("deleted_at"));
        assert!(tokens.contains("version"));
    }

    #[test]
    fn generate_generic_impl_rejects_tuple_struct() {
        let input: DeriveInput = parse_quote! {
            struct User(i32, String);
        };
        let err = generate_generic_impl(&input).unwrap_err();
        assert!(err.to_string().contains("named fields"));
    }

    #[test]
    fn generate_generic_impl_rejects_non_struct() {
        let input: DeriveInput = parse_quote! {
            enum User {
                A,
                B,
            }
        };
        let err = generate_generic_impl(&input).unwrap_err();
        assert!(err.to_string().contains("only supports structs"));
    }

    #[test]
    fn generate_generic_impl_version_update_branch() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                version: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("version = version + 1"));
    }

    #[test]
    fn generate_generic_impl_no_version_branch() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(!tokens.contains("version = version + 1"));
    }

    #[test]
    fn generate_generic_impl_includes_default_hooks_and_validation() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("ModelHooks"));
        assert!(tokens.contains("ModelValidation"));
    }

    #[test]
    fn generate_generic_impl_includes_schema_impl() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("ModelSchema"));
        assert!(tokens.contains("SchemaColumn"));
    }

    #[test]
    fn generate_generic_impl_includes_index_and_foreign_key_metadata() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                #[premix(index)]
                name: String,
                #[premix(unique(name = "users_email_uidx"))]
                email: String,
                #[premix(foreign_key(table = "accounts", column = "id"))]
                account_id: i32,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("SchemaIndex"));
        assert!(tokens.contains("idx_users_name"));
        assert!(tokens.contains("users_email_uidx"));
        assert!(tokens.contains("SchemaForeignKey"));
        assert!(tokens.contains("accounts"));
        assert!(tokens.contains("account_id"));
    }

    #[test]
    fn generate_generic_impl_includes_sensitive_fields() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                #[premix(sensitive)]
                email: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("sensitive_fields"));
        assert!(tokens.contains("\"email\""));
    }

    #[test]
    fn generate_generic_impl_skips_custom_hooks_and_validation() {
        let input: DeriveInput = parse_quote! {
            #[premix(custom_hooks, custom_validation)]
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(!tokens.contains("impl premix_orm :: ModelHooks"));
        assert!(!tokens.contains("impl premix_orm :: ModelValidation"));
    }

    #[test]
    fn is_ignored_detects_attribute() {
        let field: Field = parse_quote! {
            #[premix(ignore)]
            ignored: Option<String>
        };
        assert!(is_ignored(&field));
    }

    #[test]
    fn is_ignored_false_for_other_attrs() {
        let field: Field = parse_quote! {
            #[serde(skip)]
            name: String
        };
        assert!(!is_ignored(&field));
    }

    #[test]
    fn is_ignored_false_for_premix_other_arg() {
        let field: Field = parse_quote! {
            #[premix(skip)]
            name: String
        };
        assert!(!is_ignored(&field));
    }

    #[test]
    fn is_sensitive_detects_attribute() {
        let field: Field = parse_quote! {
            #[premix(sensitive)]
            secret: String
        };
        assert!(is_sensitive(&field));
    }

    #[test]
    fn is_sensitive_false_for_other_attrs() {
        let field: Field = parse_quote! {
            #[serde(skip)]
            secret: String
        };
        assert!(!is_sensitive(&field));
    }

    #[test]
    fn is_ignored_false_when_premix_has_no_args() {
        let field: Field = parse_quote! {
            #[premix]
            name: String
        };
        assert!(!is_ignored(&field));
    }

    #[test]
    fn derive_model_impl_emits_tokens() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = derive_model_impl(&input).unwrap().to_string();
        assert!(tokens.contains("impl"));
    }

    #[test]
    fn derive_model_impl_propagates_error() {
        let input: DeriveInput = parse_quote! {
            enum User {
                A,
            }
        };
        let err = derive_model_impl(&input).unwrap_err();
        assert!(err.to_string().contains("only supports structs"));
    }

    #[test]
    fn generate_generic_impl_includes_soft_delete_delete_impl() {
        let input: DeriveInput = parse_quote! {
            struct AuditLog {
                id: i32,
                deleted_at: Option<String>,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("deleted_at ="));
        assert!(tokens.contains("has_soft_delete"));
    }

    #[test]
    fn generate_generic_impl_ignores_marked_fields() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
                #[premix(ignore)]
                temp: Option<String>,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("temp : None"));
        assert!(!tokens.contains("\"temp\""));
    }

    #[test]
    fn generate_generic_impl_adds_relation_bounds() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                #[has_many(Post)]
                posts: Vec<Post>,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("Post : premix_orm :: Model < DB >"));
    }

    #[test]
    fn generate_generic_impl_records_field_names() {
        let input: DeriveInput = parse_quote! {
            struct Account {
                id: i32,
                user_id: i32,
                is_active: bool,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("\"user_id\""));
        assert!(tokens.contains("\"is_active\""));
    }

    #[test]
    fn generate_generic_impl_creates_column_constants() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(tokens.contains("pub mod columns_user"));
        assert!(tokens.contains("pub const ID : & str = \"id\""));
        assert!(tokens.contains("pub const NAME : & str = \"name\""));
    }
}
