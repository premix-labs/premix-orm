use premix_core::{Model, Premix};
use premix_macros::Model;

#[derive(Model, Debug, Clone)]
struct Account {
    id: i32,
    name: String,
    balance: i32,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

    // Setup
    Premix::sync::<sqlx::Sqlite, Account>(&pool).await?;

    let mut acc1 = Account {
        id: 1,
        name: "Alice".to_string(),
        balance: 1000,
    };
    let mut acc2 = Account {
        id: 2,
        name: "Bob".to_string(),
        balance: 500,
    };
    acc1.save(&pool).await?;
    acc2.save(&pool).await?;
    println!("[OK] Initial State: Alice=1000, Bob=500");

    // 1. Successful Transaction (Manual Pattern - Recommended by sqlx)
    println!("\n>> Executing Transfer (100 from Alice to Bob)...");
    {
        let mut tx = pool.begin().await?;

        // Read in transaction
        let alice_q: Option<Account> =
            sqlx::query_as("SELECT * FROM accounts WHERE name = 'Alice'")
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(mut _alice) = alice_q {
            _alice.balance -= 100;
            // In real app, we'd update the database here
        }

        // Insert Log
        let mut log = Account {
            id: 3,
            name: "Log".to_string(),
            balance: 0,
        };
        log.save(&mut *tx).await?;

        tx.commit().await?;
    }
    println!("[OK] Transaction Committed");

    // Verify
    let log = Account::find_by_id(&pool, 3).await?;
    assert!(log.is_some(), "Log should exist");
    println!("[OK] Verified: Log exists.");

    // 2. Rollback Transaction (Error causes automatic rollback on drop)
    println!("\n>> Executing Faulty Transaction (Should Rollback)...");
    let result: Result<(), sqlx::Error> = async {
        let mut tx = pool.begin().await?;

        let mut log_fail = Account {
            id: 4,
            name: "FailLog".to_string(),
            balance: 0,
        };
        log_fail.save(&mut *tx).await?;

        // Simulate Error - tx will be dropped without commit, causing rollback
        Err(sqlx::Error::RowNotFound)
    }
    .await;

    assert!(result.is_err());
    println!("[OK] Transaction Error Caught");

    // Verify Rollback
    let log_fail = Account::find_by_id(&pool, 4).await?;
    assert!(log_fail.is_none(), "FailLog should NOT exist");
    println!("[OK] Verified: FailLog rolled back.");

    Ok(())
}
