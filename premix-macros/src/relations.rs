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
        pub async fn #method_name<'e, E, DB>(&self, executor: E) -> Result<Vec<#child>, premix_core::sqlx::Error>
        where
            DB: premix_core::SqlDialect,
            E: premix_core::IntoExecutor<'e, DB = DB>,
            #parent: premix_core::Model<DB>,
            #child: premix_core::Model<DB>,
            for<'q> <DB as premix_core::sqlx::Database>::Arguments<'q>: premix_core::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_core::sqlx::Database>::Connection: premix_core::sqlx::Executor<'c, Database = DB>,
            i32: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB>,
        {
            let mut executor = executor.into_executor();
            let p = <DB as premix_core::SqlDialect>::placeholder(1);
            let sql = format!("SELECT * FROM {} WHERE {} = {}", #child_table, #fk, p);
            let query = premix_core::sqlx::query_as::<DB, #child>(&sql).bind(self.id);
            executor.fetch_all(query).await
        }
    }
}

fn generate_belongs_to(child: &Ident, parent: &Ident) -> TokenStream {
    let method_name = format_ident!("{}", parent.to_string().to_lowercase());
    let parent_table = format!("{}s", parent.to_string().to_lowercase());
    let fk = format_ident!("{}_id", parent.to_string().to_lowercase());

    quote! {
        pub async fn #method_name<'e, E, DB>(&self, executor: E) -> Result<Option<#parent>, premix_core::sqlx::Error>
        where
            DB: premix_core::SqlDialect,
            E: premix_core::IntoExecutor<'e, DB = DB>,
            #child: premix_core::Model<DB>,
            #parent: premix_core::Model<DB>,
            for<'q> <DB as premix_core::sqlx::Database>::Arguments<'q>: premix_core::sqlx::IntoArguments<'q, DB>,
            for<'c> &'c mut <DB as premix_core::sqlx::Database>::Connection: premix_core::sqlx::Executor<'c, Database = DB>,
            i32: premix_core::sqlx::Type<DB> + for<'q> premix_core::sqlx::Encode<'q, DB>,
        {
            let mut executor = executor.into_executor();
            let p = <DB as premix_core::SqlDialect>::placeholder(1);
            let sql = format!("SELECT * FROM {} WHERE id = {}", #parent_table, p);
            let query = premix_core::sqlx::query_as::<DB, #parent>(&sql).bind(self.#fk);
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
                                let ids: Vec<i32> = models.iter().map(|m| m.id).collect();
                                if ids.is_empty() { return Ok(()); }
                                let params = (1..=ids.len()).map(|i| <DB as premix_core::SqlDialect>::placeholder(i)).collect::<Vec<_>>().join(",");
                                let sql = format!("SELECT * FROM {} WHERE {} IN ({})", #child_table, #parent_fk_str, params);
                                let mut query = premix_core::sqlx::query_as::<DB, #child_model>(&sql);
                                for id in ids { query = query.bind(id); }

                                let children = executor.fetch_all(query).await?;

                                let mut grouped: std::collections::HashMap<i32, Vec<#child_model>> = std::collections::HashMap::new();
                                for child in children {
                                    grouped.entry(child.#parent_fk_ident).or_default().push(child);
                                }
                                for model in models.iter_mut() {
                                    if let Some(children) = grouped.remove(&model.id) {
                                        model.#field_name = Some(children);
                                    } else {
                                        model.#field_name = Some(Vec::new());
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
            _ => { println!("Warning: Relation '{}' not found", relation); }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

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
}
