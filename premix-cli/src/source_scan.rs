use std::error::Error;
use std::path::Path;

use premix_core::schema::{SchemaColumn, SchemaForeignKey, SchemaIndex, SchemaTable};
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Fields, Item, Token};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum DbKind {
    Sqlite,
    Postgres,
    Mysql,
}

impl DbKind {
    pub fn from_url(db_url: &str) -> Self {
        if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
            return DbKind::Postgres;
        }
        if db_url.starts_with("mysql://") {
            return DbKind::Mysql;
        }
        DbKind::Sqlite
    }
}

pub fn scan_models_schema(
    src_dir: &Path,
    db_kind: DbKind,
) -> Result<Vec<SchemaTable>, Box<dyn Error + Send + Sync>> {
    if !src_dir.exists() {
        return Ok(Vec::new());
    }

    let mut tables = Vec::new();
    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if should_skip_path(entry.path(), src_dir) {
            continue;
        }
        let source = std::fs::read_to_string(entry.path())?;
        let parsed = syn::parse_file(&source)?;
        collect_tables_from_items(&parsed.items, db_kind, &mut tables)?;
    }

    Ok(tables)
}

fn should_skip_path(path: &Path, src_dir: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if matches!(name, "premix-sync.rs" | "premix-schema.rs") {
            return true;
        }
    }
    if let Ok(rel) = path.strip_prefix(src_dir) {
        if let Some(first) = rel.components().next() {
            if first
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("bin")
            {
                return true;
            }
        }
    }
    false
}

fn collect_tables_from_items(
    items: &[Item],
    db_kind: DbKind,
    tables: &mut Vec<SchemaTable>,
) -> Result<(), syn::Error> {
    for item in items {
        match item {
            Item::Struct(item_struct) => {
                if !has_derive_model(&item_struct.attrs) {
                    continue;
                }
                let table = build_schema_table(item_struct, db_kind)?;
                tables.push(table);
            }
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &item_mod.content {
                    collect_tables_from_items(items, db_kind, tables)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn has_derive_model(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let paths: Result<Punctuated<syn::Path, Token![,]>, _> =
            attr.parse_args_with(Punctuated::parse_terminated);
        if let Ok(paths) = paths {
            return paths.iter().any(|path| {
                path.segments
                    .last()
                    .map(|seg| seg.ident == "Model")
                    .unwrap_or(false)
            });
        }
        false
    })
}

fn build_schema_table(item: &syn::ItemStruct, db_kind: DbKind) -> Result<SchemaTable, syn::Error> {
    let struct_name = item.ident.to_string();
    let table_name = format!("{}s", struct_name.to_lowercase());

    let fields = match &item.fields {
        Fields::Named(named) => &named.named,
        Fields::Unit => return Err(syn::Error::new_spanned(item, "Model must have fields")),
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(item, "Model must use named fields"));
        }
    };

    let (index_specs, foreign_key_specs) = collect_schema_specs(fields, &table_name)?;

    let mut columns = Vec::new();
    for field in fields {
        if is_ignored(field) {
            continue;
        }
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have an ident"))?;
        let name = ident.to_string();
        let primary_key = name == "id";
        let nullable = !primary_key && is_option_type(&field.ty);
        let sql_type = sql_type_for_field(&name, &field.ty, db_kind).to_string();
        columns.push(SchemaColumn {
            name,
            sql_type,
            nullable,
            primary_key,
        });
    }

    let indexes = index_specs
        .into_iter()
        .map(|spec| SchemaIndex {
            name: spec.name,
            columns: spec.columns,
            unique: spec.unique,
        })
        .collect::<Vec<_>>();

    let foreign_keys = foreign_key_specs
        .into_iter()
        .map(|spec| SchemaForeignKey {
            column: spec.column,
            ref_table: spec.ref_table,
            ref_column: spec.ref_column,
        })
        .collect::<Vec<_>>();

    Ok(SchemaTable {
        name: table_name,
        columns,
        indexes,
        foreign_keys,
        create_sql: None,
    })
}

fn has_premix_flag(field: &Field, flag: &str) -> bool {
    for attr in &field.attrs {
        if attr.path().is_ident("premix") {
            let args = attr.parse_args_with(Punctuated::<syn::Ident, Token![,]>::parse_terminated);
            if let Ok(args) = args {
                if args.iter().any(|ident| ident == flag) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_ignored(field: &Field) -> bool {
    has_premix_flag(field, "ignore")
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
) -> Result<(Vec<IndexSpec>, Vec<ForeignKeySpec>), syn::Error> {
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
                                let lit: syn::LitStr = nested.value()?.parse()?;
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
                            let lit: syn::LitStr = nested.value()?.parse()?;
                            ref_table = Some(lit.value());
                            Ok(())
                        } else if nested.path.is_ident("column") {
                            let lit: syn::LitStr = nested.value()?.parse()?;
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

fn sql_type_for_field(name: &str, ty: &syn::Type, db_kind: DbKind) -> &'static str {
    let type_name = type_name_for_field(ty);
    match type_name.as_deref() {
        Some("i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "usize") => "INTEGER",
        Some("i64" | "u64") => "BIGINT",
        Some("f32" | "f64") => match db_kind {
            DbKind::Postgres => "DOUBLE PRECISION",
            _ => "REAL",
        },
        Some("bool") => "BOOLEAN",
        Some("String" | "str") => "TEXT",
        Some("Uuid" | "DateTime" | "NaiveDateTime" | "NaiveDate") => "TEXT",
        Some("Vec<u8>") => match db_kind {
            DbKind::Postgres => "BYTEA",
            DbKind::Mysql => "LONGBLOB",
            DbKind::Sqlite => "BLOB",
        },
        _ => {
            if name == "id" || name.ends_with("_id") {
                "INTEGER"
            } else {
                "TEXT"
            }
        }
    }
}
