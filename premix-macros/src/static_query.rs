//! Compile-time query macro for true Zero-Overhead SQL generation.
//!
//! This module implements `premix_query!` which generates SQL at compile time,
//! achieving 0% overhead compared to raw sqlx.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitInt, LitStr, Token,
    parse::{Parse, ParseStream},
};

/// Represents a filter operation in the static query
pub enum StaticFilter {
    Eq { column: String, value: Expr },
    Ne { column: String, value: Expr },
    Gt { column: String, value: Expr },
    Lt { column: String, value: Expr },
    Gte { column: String, value: Expr },
    Lte { column: String, value: Expr },
}

impl StaticFilter {
    /// Generate the WHERE clause fragment and return the bind expression
    fn to_sql_fragment(&self, placeholder_index: usize, use_dollar: bool) -> (String, &Expr) {
        let placeholder = if use_dollar {
            format!("${}", placeholder_index)
        } else {
            "?".to_string()
        };

        match self {
            StaticFilter::Eq { column, value } => (format!("{} = {}", column, placeholder), value),
            StaticFilter::Ne { column, value } => (format!("{} != {}", column, placeholder), value),
            StaticFilter::Gt { column, value } => (format!("{} > {}", column, placeholder), value),
            StaticFilter::Lt { column, value } => (format!("{} < {}", column, placeholder), value),
            StaticFilter::Gte { column, value } => {
                (format!("{} >= {}", column, placeholder), value)
            }
            StaticFilter::Lte { column, value } => {
                (format!("{} <= {}", column, placeholder), value)
            }
        }
    }
}

/// Represents an assignment operation (SET column = value)
pub struct StaticAssignment {
    pub column: String,
    pub value: Expr,
}

/// Query operation type
#[derive(Clone, Copy, PartialEq)]
pub enum QueryOperation {
    Select,
    Insert,
    Update,
    Delete,
}

/// Parsed static query input
pub struct StaticQueryInput {
    pub model: Ident,
    pub operation: QueryOperation,
    pub filters: Vec<StaticFilter>,
    pub assignments: Vec<StaticAssignment>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Parse for StaticQueryInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse model type
        let model: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse operation (SELECT, INSERT, UPDATE, DELETE)
        let op_ident: Ident = input.parse()?;
        let operation = match op_ident.to_string().to_uppercase().as_str() {
            "SELECT" => QueryOperation::Select,
            "INSERT" => QueryOperation::Insert,
            "UPDATE" => QueryOperation::Update,
            "DELETE" => QueryOperation::Delete,
            _ => {
                return Err(syn::Error::new(
                    op_ident.span(),
                    format!(
                        "Unknown operation: {}. Supported: SELECT, INSERT, UPDATE, DELETE",
                        op_ident
                    ),
                ));
            }
        };

        let mut filters = Vec::new();
        let mut assignments = Vec::new();
        let mut limit = None;
        let mut offset = None;

        // Parse remaining arguments (filters, set, limit, offset)
        while !input.is_empty() {
            input.parse::<Token![,]>()?;

            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;
            let ident_str = ident.to_string();

            // Parse parenthesized arguments
            let content;
            syn::parenthesized!(content in input);

            match ident_str.as_str() {
                "filter_eq" | "filter_ne" | "filter_gt" | "filter_lt" | "filter_gte"
                | "filter_lte" => {
                    let column: LitStr = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let value: Expr = content.parse()?;

                    let filter = match ident_str.as_str() {
                        "filter_eq" => StaticFilter::Eq {
                            column: column.value(),
                            value,
                        },
                        "filter_ne" => StaticFilter::Ne {
                            column: column.value(),
                            value,
                        },
                        "filter_gt" => StaticFilter::Gt {
                            column: column.value(),
                            value,
                        },
                        "filter_lt" => StaticFilter::Lt {
                            column: column.value(),
                            value,
                        },
                        "filter_gte" => StaticFilter::Gte {
                            column: column.value(),
                            value,
                        },
                        "filter_lte" => StaticFilter::Lte {
                            column: column.value(),
                            value,
                        },
                        _ => unreachable!(),
                    };
                    filters.push(filter);
                }
                "set" => {
                    let column: LitStr = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let value: Expr = content.parse()?;
                    assignments.push(StaticAssignment {
                        column: column.value(),
                        value,
                    });
                }
                "limit" => {
                    let lit: LitInt = content.parse()?;
                    limit = Some(lit.base10_parse()?);
                }
                "offset" => {
                    let lit: LitInt = content.parse()?;
                    offset = Some(lit.base10_parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown clause: {}", ident_str),
                    ));
                }
            }
        }

        Ok(StaticQueryInput {
            model,
            operation,
            filters,
            assignments,
            limit,
            offset,
        })
    }
}

/// Generate the compile-time query code
pub fn generate_static_query(input: StaticQueryInput) -> TokenStream {
    let model = &input.model;
    let table_name = format!("{}s", model.to_string().to_lowercase());

    match input.operation {
        QueryOperation::Select => generate_select_query(model, &table_name, &input),
        QueryOperation::Insert => generate_insert_query(model, &table_name, &input),
        QueryOperation::Update => generate_update_query(model, &table_name, &input),
        QueryOperation::Delete => generate_delete_query(model, &table_name, &input),
    }
}

fn generate_select_query(model: &Ident, table_name: &str, input: &StaticQueryInput) -> TokenStream {
    // Build SQL string at compile time
    let mut sql = format!("SELECT * FROM {}", table_name);

    // Collect bind values
    let mut bind_exprs: Vec<&Expr> = Vec::new();
    let mut placeholder_index = 1;

    // Add WHERE clause
    if !input.filters.is_empty() {
        sql.push_str(" WHERE ");
        let mut first = true;
        for filter in &input.filters {
            if !first {
                sql.push_str(" AND ");
            }
            // Use $N placeholders (Postgres style, works with sqlx)
            let (fragment, value) = filter.to_sql_fragment(placeholder_index, true);
            sql.push_str(&fragment);
            bind_exprs.push(value);
            placeholder_index += 1;
            first = false;
        }
    }

    // Add LIMIT
    if let Some(limit_val) = input.limit {
        sql.push_str(&format!(" LIMIT {}", limit_val));
    }

    // Add OFFSET
    if let Some(offset_val) = input.offset {
        sql.push_str(&format!(" OFFSET {}", offset_val));
    }

    // Generate bind chain
    let binds = bind_exprs.iter().map(|expr| {
        quote! { .bind(#expr) }
    });

    // Output: sqlx::query_as with static SQL string
    quote! {
        {
            // SQL generated at compile time - Zero Overhead!
            const __PREMIX_SQL: &str = #sql;
            ::premix_orm::sqlx::query_as::<_, #model>(__PREMIX_SQL)
                #(#binds)*
        }
    }
}

fn generate_insert_query(model: &Ident, table_name: &str, input: &StaticQueryInput) -> TokenStream {
    let mut cols = Vec::new();
    let mut vals = Vec::new();
    let mut bind_exprs: Vec<&Expr> = Vec::new();
    let mut placeholder_index = 1;

    for assignment in &input.assignments {
        cols.push(assignment.column.clone());
        vals.push(format!("${}", placeholder_index));
        bind_exprs.push(&assignment.value);
        placeholder_index += 1;
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        table_name,
        cols.join(", "),
        vals.join(", ")
    );

    let binds = bind_exprs.iter().map(|expr| {
        quote! { .bind(#expr) }
    });

    quote! {
        {
            const __PREMIX_SQL: &str = #sql;
            ::premix_orm::sqlx::query_as::<_, #model>(__PREMIX_SQL)
                #(#binds)*
        }
    }
}

fn generate_update_query(model: &Ident, table_name: &str, input: &StaticQueryInput) -> TokenStream {
    let mut sql = format!("UPDATE {} SET ", table_name);
    let mut bind_exprs: Vec<&Expr> = Vec::new();
    let mut placeholder_index = 1;

    // SET clause
    let mut first = true;
    for assignment in &input.assignments {
        if !first {
            sql.push_str(", ");
        }
        sql.push_str(&format!("{} = ${}", assignment.column, placeholder_index));
        bind_exprs.push(&assignment.value);
        placeholder_index += 1;
        first = false;
    }

    // WHERE clause
    if !input.filters.is_empty() {
        sql.push_str(" WHERE ");
        let mut first = true;
        for filter in &input.filters {
            if !first {
                sql.push_str(" AND ");
            }
            let (fragment, value) = filter.to_sql_fragment(placeholder_index, true);
            sql.push_str(&fragment);
            bind_exprs.push(value);
            placeholder_index += 1;
            first = false;
        }
    }

    sql.push_str(" RETURNING *");

    let binds = bind_exprs.iter().map(|expr| {
        quote! { .bind(#expr) }
    });

    quote! {
        {
            const __PREMIX_SQL: &str = #sql;
            ::premix_orm::sqlx::query_as::<_, #model>(__PREMIX_SQL)
                #(#binds)*
        }
    }
}

fn generate_delete_query(model: &Ident, table_name: &str, input: &StaticQueryInput) -> TokenStream {
    let mut sql = format!("DELETE FROM {}", table_name);
    let mut bind_exprs: Vec<&Expr> = Vec::new();
    let mut placeholder_index = 1;

    // WHERE clause
    if !input.filters.is_empty() {
        sql.push_str(" WHERE ");
        let mut first = true;
        for filter in &input.filters {
            if !first {
                sql.push_str(" AND ");
            }
            let (fragment, value) = filter.to_sql_fragment(placeholder_index, true);
            sql.push_str(&fragment);
            bind_exprs.push(value);
            placeholder_index += 1;
            first = false;
        }
    }

    // RETURNING * only supported in Postgres/SQLite 3.35+, we assume availability or non-returning execution?
    // Current pattern in `query_as` implies we want to return the deleted row.
    sql.push_str(" RETURNING *");

    let binds = bind_exprs.iter().map(|expr| {
        quote! { .bind(#expr) }
    });

    quote! {
        {
            const __PREMIX_SQL: &str = #sql;
            ::premix_orm::sqlx::query_as::<_, #model>(__PREMIX_SQL)
                #(#binds)*
        }
    }
}
