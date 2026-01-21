use axum::extract::FromRef;

/// Axum state wrapper for Premix pools.
#[derive(Clone, Debug)]
pub struct PremixState<DB: sqlx::Database> {
    /// The database connection pool.
    pub pool: sqlx::Pool<DB>,
}

impl<DB: sqlx::Database> PremixState<DB> {
    /// Creates a new `PremixState` with the given pool.
    pub fn new(pool: sqlx::Pool<DB>) -> Self {
        Self { pool }
    }
}

impl<DB: sqlx::Database> FromRef<PremixState<DB>> for sqlx::Pool<DB> {
    fn from_ref(state: &PremixState<DB>) -> Self {
        state.pool.clone()
    }
}
