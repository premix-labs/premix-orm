use premix_core::ModelValidation;
use premix_macros::Model;

#[derive(Model, Debug, Clone)]
struct User {
    id: i32,
    email: String,
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validation Demo\n");

    // Valid user
    let valid_user = User {
        id: 1,
        email: "test@example.com".to_string(),
        name: "John Doe".to_string(),
        age: 25,
    };

    match valid_user.validate() {
        Ok(_) => println!("✅ Valid user: {:?}", valid_user),
        Err(errors) => {
            println!("❌ Validation errors:");
            for e in errors {
                println!("  - {}: {}", e.field, e.message);
            }
        }
    }

    // Invalid user (demonstration - actual validation rules will be added via attributes)
    let invalid_user = User {
        id: 2,
        email: "not-an-email".to_string(), // Would fail email validation
        name: "A".to_string(),             // Would fail min_len=3
        age: -5,                           // Would fail min=0
    };

    match invalid_user.validate() {
        Ok(_) => println!("\n✅ User passed validation (rules not yet implemented via attributes)"),
        Err(errors) => {
            println!("\n❌ Validation errors:");
            for e in errors {
                println!("  - {}: {}", e.field, e.message);
            }
        }
    }

    println!("\n🎉 Validation Demo Complete!");
    println!("\nNote: Add validation rules using attributes like:");
    println!("  #[premix(email)]");
    println!("  #[premix(min_len = 3, max_len = 50)]");
    println!("  #[premix(min = 0, max = 150)]");

    Ok(())
}
