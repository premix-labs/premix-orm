use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

mod relations;

#[proc_macro_derive(Model, attributes(has_many, belongs_to, premix))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let impl_block = match generate_generic_impl(&input) {
        Ok(tokens) => tokens,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let rel_block = match relations::impl_relations(&input) {
        Ok(tokens) => tokens,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    TokenStream::from(quote! {
        #impl_block
        #rel_block
    })
}

fn generate_generic_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let table_name = struct_name.to_string().to_lowercase() + "s";

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
            async fn update(&mut self, mut executor: premix_core::Executor<'_, DB>) -> Result<premix_core::UpdateResult, premix_core::sqlx::Error> {
                let set_clause = vec![ #( format!("{} = {}", #field_names, <DB as premix_core::SqlDialect>::placeholder(1 + #field_indices)) ),* ].join(", ");
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1 + #field_idents_len);
                let ver_p = <DB as premix_core::SqlDialect>::placeholder(2 + #field_idents_len);
                let sql = format!(
                    "UPDATE {} SET {}, version = version + 1 WHERE id = {} AND version = {}",
                    #table_name, set_clause, id_p, ver_p
                );

                let mut query = premix_core::sqlx::query::<DB>(&sql)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id)
                    .bind(&self.version);

                let result = match &mut executor {
                    premix_core::Executor::Pool(pool) => query.execute(*pool).await?,
                    premix_core::Executor::Conn(conn) => query.execute(&mut **conn).await?,
                };

                if <DB as premix_core::SqlDialect>::rows_affected(&result) == 0 {
                    let exists_p = <DB as premix_core::SqlDialect>::placeholder(1);
                    let exists_sql = format!("SELECT id FROM {} WHERE id = {}", #table_name, exists_p);
                    let exists_query = premix_core::sqlx::query::<DB>(&exists_sql).bind(&self.id);
                    let exists = match &mut executor {
                        premix_core::Executor::Pool(pool) => exists_query.fetch_optional(*pool).await?,
                        premix_core::Executor::Conn(conn) => exists_query.fetch_optional(&mut **conn).await?,
                    };

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
            async fn update(&mut self, mut executor: premix_core::Executor<'_, DB>) -> Result<premix_core::UpdateResult, premix_core::sqlx::Error> {
                let set_clause = vec![ #( format!("{} = {}", #field_names, <DB as premix_core::SqlDialect>::placeholder(1 + #field_indices)) ),* ].join(", ");
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1 + #field_idents_len);
                let sql = format!("UPDATE {} SET {} WHERE id = {}", #table_name, set_clause, id_p);

                let mut query = premix_core::sqlx::query::<DB>(&sql)
                    #( .bind(&self.#field_idents) )*
                    .bind(&self.id);

                let result = match &mut executor {
                    premix_core::Executor::Pool(pool) => query.execute(*pool).await?,
                    premix_core::Executor::Conn(conn) => query.execute(&mut **conn).await?,
                };

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
            async fn delete(&mut self, mut executor: premix_core::Executor<'_, DB>) -> Result<(), premix_core::sqlx::Error> {
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1);
                let sql = format!("UPDATE {} SET deleted_at = {} WHERE id = {}", #table_name, <DB as premix_core::SqlDialect>::current_timestamp_fn(), id_p);
                match &mut executor {
                    premix_core::Executor::Pool(pool) => {
                        premix_core::sqlx::query::<DB>(&sql).bind(&self.id).execute(*pool).await?;
                    }
                    premix_core::Executor::Conn(conn) => {
                        premix_core::sqlx::query::<DB>(&sql).bind(&self.id).execute(&mut **conn).await?;
                    }
                }
                self.deleted_at = Some("DELETED".to_string());
                Ok(())
            }
            fn has_soft_delete() -> bool { true }
        }
    } else {
        quote! {
            async fn delete(&mut self, mut executor: premix_core::Executor<'_, DB>) -> Result<(), premix_core::sqlx::Error> {
                let id_p = <DB as premix_core::SqlDialect>::placeholder(1);
                let sql = format!("DELETE FROM {} WHERE id = {}", #table_name, id_p);
                match &mut executor {
                    premix_core::Executor::Pool(pool) => {
                        premix_core::sqlx::query::<DB>(&sql).bind(&self.id).execute(*pool).await?;
                    }
                    premix_core::Executor::Conn(conn) => {
                        premix_core::sqlx::query::<DB>(&sql).bind(&self.id).execute(&mut **conn).await?;
                    }
                }
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

        impl<DB> premix_core::Model<DB> for #struct_name
        where
            DB: premix_core::SqlDialect,
            for<'c> &'c str: premix_core::sqlx::ColumnIndex<DB::Row>,
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

            async fn save<'e, E>(&mut self, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::sqlx::Executor<'e, Database = DB>
            {
                use premix_core::ModelHooks;
                self.before_save().await?;

                let binds = (1..=#field_idents_len).map(|i| <DB as premix_core::SqlDialect>::placeholder(i)).collect::<Vec<_>>().join(", ");
                let sql = format!("INSERT INTO {} ({}) VALUES ({})", #table_name, vec![ #( #field_names ),* ].join(", "), binds);

                premix_core::sqlx::query::<DB>(&sql)
                    #( .bind(&self.#field_idents) )*
                    .execute(executor).await?;

                self.after_save().await?;
                Ok(())
            }

            #update_impl
            #delete_impl

            async fn find_by_id<'e, E>(executor: E, id: i32) -> Result<Option<Self>, premix_core::sqlx::Error>
            where
                E: premix_core::sqlx::Executor<'e, Database = DB>
            {
                let p = <DB as premix_core::SqlDialect>::placeholder(1);
                let mut where_clause = format!("WHERE id = {}", p);
                if Self::has_soft_delete() {
                    where_clause.push_str(" AND deleted_at IS NULL");
                }
                let sql = format!("SELECT * FROM {} {} LIMIT 1", #table_name, where_clause);
                premix_core::sqlx::query_as::<DB, Self>(&sql)
                    .bind(id)
                    .fetch_optional(executor)
                    .await
            }

            async fn eager_load<'e, E>(models: &mut [Self], relation: &str, executor: E) -> Result<(), premix_core::sqlx::Error>
            where
                E: premix_core::sqlx::Executor<'e, Database = DB>
            {
                #eager_load_body
            }
        }
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
