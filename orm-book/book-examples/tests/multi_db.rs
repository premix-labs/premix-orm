#[test]
fn sqlite_pool_type_compiles() {
    let _ = std::any::TypeId::of::<premix_orm::sqlx::SqlitePool>();
}
