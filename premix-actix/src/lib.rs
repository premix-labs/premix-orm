/// Actix-web helper type alias for Premix pools.
pub type PremixData<DB> = actix_web::web::Data<sqlx::Pool<DB>>;

/// Wrap a Premix pool as Actix application data.
pub fn premix_data<DB: sqlx::Database>(pool: sqlx::Pool<DB>) -> PremixData<DB> {
    actix_web::web::Data::new(pool)
}
