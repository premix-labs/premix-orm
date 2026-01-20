use premix_core::{Model, Premix};
use premix_macros::Model;
use tracing::info;

#[derive(Debug, Default, Clone, Model)]
struct Product {
    id: i32,
    name: String,
    price: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Tracing Subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false) // cleaner output
        .init();

    info!("Starting Tracing App...");

    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    sqlx::query(&<Product as Model<sqlx::Sqlite>>::create_table_sql())
        .execute(&pool)
        .await?;

    // 2. Instrument Save
    let mut p = Product {
        id: 1,
        name: "Gaming Mouse".to_string(),
        price: 50,
    };
    p.save(&pool).await?; // Should show "Executing INSERT"

    // 3. Instrument Find
    let found = Product::find_in_pool(&pool)
        .filter_gt("price", 10)
        .all()
        .await?;
    info!("Found {} products", found.len()); // Should show "Executing SELECT ALL"

    // 4. Instrument Transaction (Manual Pattern)
    {
        let mut tx = pool.begin().await?;
        info!("Transaction started");

        let mut p2 = Product {
            id: 2,
            name: "Keyboard".to_string(),
            price: 100,
        };
        p2.save(&mut *tx).await?; // Should show transaction logs

        tx.commit().await?;
        info!("Transaction committed");
    }

    info!("Finished!");
    Ok(())
}
