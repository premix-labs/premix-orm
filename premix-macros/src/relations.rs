use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn impl_relations(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let mut methods = TokenStream::new();

    for attr in &input.attrs {
        if attr.path().is_ident("has_many") {
            let child = attr.parse_args::<syn::Ident>()?;
            methods.extend(generate_has_many(struct_name, &child));
        } else if attr.path().is_ident("belongs_to") {
            let parent = attr.parse_args::<syn::Ident>()?;
            methods.extend(generate_belongs_to(struct_name, &parent));
        }
    }

    if !methods.is_empty() {
        Ok(quote! {
            impl #struct_name {
                #methods
            }
        })
    } else {
        Ok(TokenStream::new())
    }
}

fn generate_has_many(parent: &Ident, child: &Ident) -> TokenStream {
    let method_name = format_ident!("{}s_lazy", child.to_string().to_lowercase());
    let child_table = format!("{}s", child.to_string().to_lowercase());
    let fk = format!("{}_id", parent.to_string().to_lowercase());

    quote! {
        pub async fn #method_name<'e, E, DB>(&self, executor: E) -> Result<Vec<#child>, premix_orm::sqlx::Error>
        where
            DB: premix_orm::SqlDialect,
            E: premix_orm::IntoExecutor<'e, DB = DB>,
            #parent: premix_orm::Model<DB>,
            #child: premix_orm::Model<DB>,
            for<'q> <DB as premix_orm::sqlx::Database>::Arguments<'q>: premix_orm::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_orm::sqlx::Database>::Connection: premix_orm::sqlx::Executor<'c, Database = DB>,
            i32: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB>,
        {
            let mut executor = executor.into_executor();
            let p = <DB as premix_orm::SqlDialect>::placeholder(1);
            let sql = format!("SELECT * FROM {} WHERE {} = {}", #child_table, #fk, p);
            let query = premix_orm::sqlx::query_as::<DB, #child>(&sql).bind(self.id);
            executor.fetch_all(query).await
        }
    }
}

fn generate_belongs_to(child: &Ident, parent: &Ident) -> TokenStream {
    let method_name = format_ident!("{}", parent.to_string().to_lowercase());
    let parent_table = format!("{}s", parent.to_string().to_lowercase());
    let fk = format_ident!("{}_id", parent.to_string().to_lowercase());

    quote! {
        pub async fn #method_name<'e, E, DB>(&self, executor: E) -> Result<Option<#parent>, premix_orm::sqlx::Error>
        where
            DB: premix_orm::SqlDialect,
            E: premix_orm::IntoExecutor<'e, DB = DB>,
            #child: premix_orm::Model<DB>,
            #parent: premix_orm::Model<DB>,
            for<'q> <DB as premix_orm::sqlx::Database>::Arguments<'q>: premix_orm::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_orm::sqlx::Database>::Connection: premix_orm::sqlx::Executor<'c, Database = DB>,
            i32: premix_orm::sqlx::Type<DB> + for<'q> premix_orm::sqlx::Encode<'q, DB>,
        {
            let mut executor = executor.into_executor();
            let p = <DB as premix_orm::SqlDialect>::placeholder(1);
            let sql = format!("SELECT * FROM {} WHERE id = {}", #parent_table, p);
            let query = premix_orm::sqlx::query_as::<DB, #parent>(&sql).bind(self.#fk);
            executor.fetch_optional(query).await
        }
    }
}

pub fn generate_eager_load_body(input: &DeriveInput) -> syn::Result<TokenStream> {
    let parent_struct = &input.ident;
    let mut arms = TokenStream::new();

    if let syn::Data::Struct(data) = &input.data
        && let syn::Fields::Named(fields) = &data.fields
    {
        for field in &fields.named {
            let field_name = &field.ident;

            for attr in &field.attrs {
                if attr.path().is_ident("has_many") {
                    let child_model = attr.parse_args::<Ident>()?;
                    let relation_name = field_name.as_ref().unwrap().to_string();

                    let child_table = format!("{}s", child_model.to_string().to_lowercase());
                    let parent_fk_str = format!("{}_id", parent_struct.to_string().to_lowercase());
                    let parent_fk_ident = format_ident!("{}", parent_fk_str);

                    arms.extend(quote! {
                            #relation_name => {
                                let mut ids: Vec<i32> = models.iter().map(|m| m.id).collect();
                                if ids.is_empty() { return Ok(()); }
                                ids.sort_unstable();
                                ids.dedup();

                                // Use Vec instead of HashMap for better cache locality (O(log N) binary search)
                                let mut grouped: Vec<(i32, Vec<#child_model>)> =
                                    Vec::with_capacity(models.len());

                                const CHUNK_SIZE: usize = 500;
                                for chunk in ids.chunks(CHUNK_SIZE) {
                                    let params = premix_orm::build_placeholders::<DB>(1, chunk.len());
                                    let sql = format!(
                                        "SELECT * FROM {} WHERE {} IN ({})",
                                        #child_table,
                                        #parent_fk_str,
                                        params
                                    );
                                    let mut query = premix_orm::sqlx::query_as::<DB, #child_model>(&sql);
                                    for id in chunk {
                                        query = query.bind(*id);
                                    }
                                    let children = executor.fetch_all(query).await?;
                                    for child in children {
                                        let fk = child.#parent_fk_ident;
                                        // Use binary search to find insertion point (grouped is kept sorted)
                                        match grouped.binary_search_by_key(&fk, |item| item.0) {
                                            Ok(pos) => grouped[pos].1.push(child),
                                            Err(pos) => grouped.insert(pos, (fk, vec![child])),
                                        }
                                    }
                                }

                                for model in models.iter_mut() {
                                    // Binary search for model.id in sorted grouped Vec
                                    if let Ok(idx) = grouped.binary_search_by_key(&model.id, |item| item.0) {
                                        // Move Vec directly instead of cloning
                                        let children = std::mem::take(&mut grouped[idx].1);
                                        model.#field_name = Some(children);
                                    } else {
                                        model.#field_name = Some(Vec::new());
                                    }
                                }
                            },
                        });
                } else if attr.path().is_ident("belongs_to") {
                    let parent_model = attr.parse_args::<Ident>()?;
                    let relation_name = field_name.as_ref().unwrap().to_string();
                    let parent_table = format!("{}s", parent_model.to_string().to_lowercase());
                    let fk_str = format!("{}_id", parent_model.to_string().to_lowercase());
                    let fk_ident = format_ident!("{}", fk_str);

                    arms.extend(quote! {
                            #relation_name => {
                                let mut ids: Vec<i32> = models.iter().map(|m| m.#fk_ident).collect();
                                if ids.is_empty() { return Ok(()); }
                                ids.sort_unstable();
                                ids.dedup();

                                // Use Vec instead of HashMap for better cache locality (O(log N) binary search)
                                let mut grouped: Vec<(i32, Option<#parent_model>)> =
                                    Vec::with_capacity(ids.len());

                                const CHUNK_SIZE: usize = 500;
                                for chunk in ids.chunks(CHUNK_SIZE) {
                                    let params = premix_orm::build_placeholders::<DB>(1, chunk.len());
                                    let sql = format!(
                                        "SELECT * FROM {} WHERE id IN ({})",
                                        #parent_table,
                                        params
                                    );
                                    let mut query = premix_orm::sqlx::query_as::<DB, #parent_model>(&sql);
                                    for id in chunk {
                                        query = query.bind(*id);
                                    }
                                    let parents = executor.fetch_all(query).await?;
                                    for parent in parents {
                                        // Use binary search to find insertion point (grouped is kept sorted)
                                        match grouped.binary_search_by_key(&parent.id, |item| item.0) {
                                            Ok(_) => {} // Skip duplicates (shouldn't happen with id)
                                            Err(pos) => grouped.insert(pos, (parent.id, Some(parent))),
                                        }
                                    }
                                }

                                for model in models.iter_mut() {
                                    // Binary search for model's foreign key in sorted grouped Vec
                                    if let Ok(idx) = grouped.binary_search_by_key(&model.#fk_ident, |item| item.0) {
                                        // Move value instead of cloning
                                        model.#field_name = grouped[idx].1.take();
                                    } else {
                                        model.#field_name = None;
                                    }
                                }
                            },
                        });
                }
            }
        }
    }

    Ok(quote! {
        match relation {
            #arms
            _ => {
                premix_orm::tracing::warn!("premix relation '{}' not found", relation);
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn impl_relations_generates_methods() {
        let input: DeriveInput = parse_quote! {
            #[has_many(Post)]
            #[belongs_to(Account)]
            struct User {
                id: i32,
            }
        };
        let tokens = impl_relations(&input).unwrap().to_string();
        assert!(tokens.contains("posts_lazy"));
        assert!(tokens.contains("account"));
    }

    #[test]
    fn generate_eager_load_body_includes_relation_arm() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
                #[has_many(Post)]
                posts: Option<Vec<Post>>,
            }
        };
        let tokens = generate_eager_load_body(&input).unwrap().to_string();
        assert!(tokens.contains("posts"));
        assert!(tokens.contains("WHERE"));
        assert!(tokens.contains("IN"));
    }

    #[test]
    fn generate_eager_load_body_includes_belongs_to_arm() {
        let input: DeriveInput = parse_quote! {
            struct Post {
                id: i32,
                user_id: i32,
                #[belongs_to(User)]
                user: Option<User>,
            }
        };
        let tokens = generate_eager_load_body(&input).unwrap().to_string();
        assert!(tokens.contains("user"));
        assert!(tokens.contains("user_id"));
        assert!(tokens.contains("WHERE id IN"));
    }

    #[test]
    fn impl_relations_returns_empty_when_no_attrs() {
        let input: DeriveInput = parse_quote! {
            struct User {
                id: i32,
            }
        };
        let tokens = impl_relations(&input).unwrap().to_string();
        assert!(tokens.is_empty());
    }
}
