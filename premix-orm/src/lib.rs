//! Premix ORM Re-exports
//!
//! This crate serves as a "facade" to simplify imports for Premix ORM users.
//! Instead of depending on `premix-core` and `premix-macros` separately,
//! you can just depend on `premix` and importing everything from here.

pub use premix_core::*;
pub use premix_macros::Model;
