use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, Ident, Token, parse_macro_input,
    punctuated::Punctuated,
};

mod relations;

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
    fn generate_generic_impl_skips_custom_hooks_and_validation() {
        let input: DeriveInput = parse_quote! {
            #[premix(custom_hooks, custom_validation)]
            struct User {
                id: i32,
                name: String,
            }
        };
        let tokens = generate_generic_impl(&input).unwrap().to_string();
        assert!(!tokens.contains("impl premix_core :: ModelHooks"));
        assert!(!tokens.contains("impl premix_core :: ModelValidation"));
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
        assert!(tokens.contains("Post : premix_core :: Model < DB >"));
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
    let field_indices: Vec<_> = (0..db_fields.len()).collect();
    let field_names: Vec<_> = field_idents.iter().map(|id| id.to_string()).collect();
    let field_idents_len = field_idents.len();

    let eager_load_body = relations::generate_eager_load_body(input)?;
    let has_version = field_names.contains(&"version".to_string());
    let has_soft_delete = field_names.contains(&"deleted_at".to_string());

    let update_impl = if has_version {
        quote! {
            async fn update<'a, E>(&mut self, executor: E) -> Result<premix_core::UpdateResult, premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                let set_clause = vec![ #( format!("{} = {}", #field_names, <DB as premix_core::SqlDialect>::placeholder(1 + #field_indices)) ),* ].join(", ");
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1 + #field_idents_len);
                let ver_p = <DB as premix_core::SqlDialect>::placeholder(2 + #field_idents_len);
                let sql = format!(
                    "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                    table_name, set_clause, id_p, ver_p
                );

                let mut query = premix_core::sqlx::query::<DB>(&sql)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = executor.execute(query).await?;

                if <DB as premix_core::SqlDialect>::rows_affected(&result) == 0 {
                    let exists_p = <DB as premix_core::SqlDialect>::placeholder(1);
                    let exists_sql = format!("SELECT id FROM {} WHERE id = {}", table_name, exists_p);
                    let exists_query = premix_core::sqlx::query_as::<DB, (i32,)>(&exists_sql).bind(&self.id);
                    let exists = executor.fetch_optional(exists_query).await?;

                    if exists.is_none() {
                        Ok(premix_core::UpdateResult::NotFound)
                    } else {
                        Ok(premix_core::UpdateResult::VersionConflict)
                    }
                } else {
                    self.version += 1;
                    Ok(premix_core::UpdateResult::Success)
                }
            }
        }
    } else {
        quote! {
            async fn update<'a, E>(&mut self, executor: E) -> Result<premix_core::UpdateResult, premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                let set_clause = vec![ #( format!("{} = {}", #field_names, <DB as premix_core::SqlDialect>::placeholder(1 + #field_indices)) ),* ].join(", ");
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1 + #field_idents_len);
                let sql = format!("UPDATE {} SET {} WHERE id = {}", table_name, set_clause, id_p);

                let mut query = premix_core::sqlx::query::<DB>(&sql)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = executor.execute(query).await?;

                if <DB as premix_core::SqlDialect>::rows_affected(&result) == 0 {
                    Ok(premix_core::UpdateResult::NotFound)
                } else {
                    Ok(premix_core::UpdateResult::Success)
                }
            }
        }
    };

    let delete_impl = if has_soft_delete {
        quote! {
            async fn delete<'a, E>(&mut self, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1);
                let sql = format!("UPDATE {} SET deleted_at = {} WHERE id = {}", table_name, <DB as premix_core::SqlDialect>::current_timestamp_fn(), id_p);

                let query = premix_core::sqlx::query::<DB>(&sql).bind(&self.id);
                executor.execute(query).await?;

                self.deleted_at = Some("DELETED".to_string());
                Ok(())
            }
            fn has_soft_delete() -> bool { true }
        }
    } else {
        quote! {
            async fn delete<'a, E>(&mut self, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                let table_name = Self::table_name();
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1);
                let sql = format!("DELETE FROM {} WHERE id = {}", table_name, id_p);

                let query = premix_core::sqlx::query::<DB>(&sql).bind(&self.id);
                executor.execute(query).await?;

                Ok(())
            }
            fn has_soft_delete() -> bool { false }
        }
    };

    let mut related_model_bounds = Vec::new();
    for field in all_fields {
        for attr in &field.attrs {
            if (attr.path().is_ident("has_many") || attr.path().is_ident("belongs_to"))
                && let Ok(related_ident) = attr.parse_args::<syn::Ident>()
            {
                related_model_bounds.push(quote! { #related_ident: premix_core::Model<DB> });
            }
        }
    }

    let hooks_impl = if custom_hooks {
        quote! {}
    } else {
        quote! {
            #[premix_core::async_trait::async_trait]
            impl premix_core::ModelHooks for #struct_name {}
        }
    };

    let validation_impl = if custom_validation {
        quote! {}
    } else {
        quote! {
            impl premix_core::ModelValidation for #struct_name {}
        }
    };

    // Generic Implementation
    Ok(quote! {
        impl<'r, R> premix_core::sqlx::FromRow<'r, R> for #struct_name
        where
            R: premix_core::sqlx::Row,
            R::Database: premix_core::sqlx::Database,
            #(
                #field_types: premix_core::sqlx::Type<R::Database> + premix_core::sqlx::Decode<'r, R::Database>,
            )*
            for<'c> &'c str: premix_core::sqlx::ColumnIndex<R>,
        {
            fn from_row(row: &'r R) -> Result<Self, premix_core::sqlx::Error> {
                use premix_core::sqlx::Row;
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

        #[premix_core::async_trait::async_trait]
        impl<DB> premix_core::Model<DB> for #struct_name
        where
            DB: premix_core::SqlDialect,
            for<'c> &'c str: premix_core::sqlx::ColumnIndex<DB::Row>,
            usize: premix_core::sqlx::ColumnIndex<DB::Row>,
            for<'q> <DB as premix_core::sqlx::Database>::Arguments<'q>: premix_core::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_core::sqlx::Database>::Connection: premix_core::sqlx::Executor<'c, Database = DB>,
            i32: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB> + for<'r> premix_core::sqlx::Decode<'r, DB>,
            i64: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB> + for<'r> premix_core::sqlx::Decode<'r, DB>,
            String: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB> + for<'r> premix_core::sqlx::Decode<'r, DB>,
            bool: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB> + for<'r> premix_core::sqlx::Decode<'r, DB>,
            Option<String>: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB> + for<'r> premix_core::sqlx::Decode<'r, DB>,
            #( #related_model_bounds, )*
        {
            fn table_name() -> &'static str {
                #table_name
            }

            fn create_table_sql() -> String {
                let mut cols = vec!["id ".to_string() + <DB as premix_core::SqlDialect>::auto_increment_pk()];
                #(
                    if #field_names != "id" {
                        let field_name: &str = #field_names;
                        let sql_type = if field_name.ends_with("_id") {
                            <DB as premix_core::SqlDialect>::int_type()
                        } else {
                            match field_name {
                                "name" | "title" | "status" | "email" | "role" => <DB as premix_core::SqlDialect>::text_type(),
                                "age" | "version" | "price" | "balance" => <DB as premix_core::SqlDialect>::int_type(),
                                "is_active" => <DB as premix_core::SqlDialect>::bool_type(),
                                "deleted_at" => <DB as premix_core::SqlDialect>::text_type(),
                                _ => <DB as premix_core::SqlDialect>::text_type(),
                            }
                        };
                        cols.push(format!("{} {}", #field_names, sql_type));
                    }
                )*
                format!("CREATE TABLE IF NOT EXISTS {} ({})", #table_name, cols.join(", "))
            }

            fn list_columns() -> Vec<String> {
                vec![ #( #field_names.to_string() ),* ]
            }

            async fn save<'a, E>(&mut self, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                use premix_core::ModelHooks;
                self.before_save().await?;

                // Filter out 'id' and 'version' for INSERT
                let columns: Vec<&str> = vec![ #( #field_names ),* ]
                    .into_iter()
                    .filter(|&c| {
                        if c == "id" { return self.id != 0; }
                        true
                    })
                    .collect();

                let placeholders = (1..=columns.len())
                    .map(|i| <DB as premix_core::SqlDialect>::placeholder(i))
                    .collect::<Vec<_>>()
                    .join(", ");

                let sql = format!("INSERT INTO {} ({}) VALUES ({})", #table_name, columns.join(", "), placeholders);

                let mut query = premix_core::sqlx::query::<DB>(&sql);

                // Bind only non-id/version fields
                #(
                    if #field_names != "id" {
                        query = query.bind(&self.#field_idents);
                    } else {
                        if self.id != 0 {
                            query = query.bind(&self.id);
                        }
                    }
                )*

                let result = executor.execute(query).await?;

                // Sync the ID from Database
                let last_id = <DB as premix_core::SqlDialect>::last_insert_id(&result);
                if last_id > 0 {
                     self.id = last_id as i32;
                }

                self.after_save().await?;
                Ok(())
            }

            #update_impl
            #delete_impl

            async fn find_by_id<'a, E>(executor: E, id: i32) -> Result<Option<Self>, premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                let p = <DB as premix_core::SqlDialect>::placeholder(1);
                let mut where_clause = format!("WHERE id = {}", p);
                if Self::has_soft_delete() {
                    where_clause.push_str(" AND deleted_at IS NULL");
                }
                let sql = format!("SELECT * FROM {} {} LIMIT 1", #table_name, where_clause);
                let query = premix_core::sqlx::query_as::<DB, Self>(&sql).bind(id);

                executor.fetch_optional(query).await
            }

            async fn eager_load<'a, E>(models: &mut [Self], relation: &str, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::IntoExecutor<'a, DB = DB>
            {
                let mut executor = executor.into_executor();
                #eager_load_body
            }
        }

        #hooks_impl
        #validation_impl
    })
}

fn is_ignored(field: &Field) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("premix")
            && let Ok(meta) = attr.parse_args::<syn::Ident>()
            && meta == "ignore"
        {
            return true;
        }
    }
    false
}

fn has_premix_flag(attrs: &[Attribute], flag: &str) -> bool {
    for attr in attrs {
        if attr.path().is_ident("premix") {
            let args =
                attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated);
            if let Ok(args) = args {
                if args.iter().any(|ident| ident == flag) {
                    return true;
                }
            }
        }
    }
    false
}
