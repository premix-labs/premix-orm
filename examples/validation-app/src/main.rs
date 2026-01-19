use premix_core::{ModelValidation, ValidationError};
use premix_macros::Model;

#[derive(Model, Debug, Clone)]
#[premix(custom_validation)]
struct User {
    id: i32,
    email: String,
    name: String,
    age: i32,
}

impl ModelValidation for User {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if !self.email.contains('@') {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "email must contain '@'".to_string(),
            });
        }

        if self.name.trim().len() < 3 {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "name must be at least 3 characters".to_string(),
            });
        }

        if self.age < 0 {
            errors.push(ValidationError {
                field: "age".to_string(),
                message: "age must be >= 0".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(">> Validation Demo\n");

    // Valid user
    let valid_user = User {
        id: 1,
        email: "test@example.com".to_string(),
        name: "John Doe".to_string(),
        age: 25,
    };

    match valid_user.validate() {
        Ok(_) => println!("[OK] Valid user: {:?}", valid_user),
        Err(errors) => {
            println!("[FAIL] Validation errors:");
            for e in errors {
                println!("  - {}: {}", e.field, e.message);
            }
        }
    }

    // Invalid user
    let invalid_user = User {
        id: 2,
        email: "not-an-email".to_string(),
        name: "A".to_string(),
        age: -5,
    };

    match invalid_user.validate() {
        Ok(_) => println!("\n[OK] User passed validation"),
        Err(errors) => {
            println!("\n[FAIL] Validation errors:");
            for e in errors {
                println!("  - {}: {}", e.field, e.message);
            }
        }
    }

    println!("\n[DONE] Validation Demo Complete!");

    Ok(())
}
