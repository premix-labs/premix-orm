use premix_core::{Executor, Model, Premix, UpdateResult};
use premix_macros::Model;

#[derive(Model, Debug, Clone)]
struct Product {
    id: i32,
    name: String,
    price: i32,
    version: i32, // Version field for optimistic locking
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

    // Setup
    Premix::sync::<sqlx::Sqlite, Product>(&pool).await?;

    // 1. Create initial product
    let mut product = Product {
        id: 1,
        name: "Gaming Mouse".to_string(),
        price: 50,
        version: 0,
    };
    product.save(&pool).await?;
    println!("[OK] Created: {:?}", product);

    // 2. Simulate two users fetching the same product
    let mut user_a = Product::find_by_id(&pool, 1).await?.unwrap();
    let mut user_b = Product::find_by_id(&pool, 1).await?.unwrap();

    println!(
        "\n[USER] User A fetched: price={}, version={}",
        user_a.price, user_a.version
    );
    println!(
        "[USER] User B fetched: price={}, version={}",
        user_b.price, user_b.version
    );

    // 3. User A updates the product
    user_a.price = 60;
    match user_a.update(Executor::Pool(&pool)).await? {
        UpdateResult::Success => println!("\n[OK] User A updated price to 60"),
        UpdateResult::VersionConflict => println!("\n[FAIL] User A: Version conflict!"),
        UpdateResult::NotFound => println!("\n[FAIL] User A: Product not found!"),
        _ => {}
    }

    // 4. User B tries to update (should detect conflict)
    user_b.price = 55;
    match user_b.update(Executor::Pool(&pool)).await? {
        UpdateResult::Success => {
            println!("[OK] User B updated price to 55 (would conflict with full version check)")
        }
        UpdateResult::VersionConflict => {
            println!("[FAIL] User B: Version conflict detected! [DONE]")
        }
        UpdateResult::NotFound => println!("[FAIL] User B: Product not found!"),
        _ => {}
    }

    // 5. Verify final state
    let final_product = Product::find_by_id(&pool, 1).await?.unwrap();
    println!(
        "\n[DATA] Final product state: price={}, version={}",
        final_product.price, final_product.version
    );

    println!("\n[DONE] Optimistic Locking Demo Complete!");

    Ok(())
}
