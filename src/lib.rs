mod conversions;
mod data_store;
mod error;
mod surrealdb_types;

// LanceDB implementation modules (for migration)
mod lance_data_store_simple;
pub mod migration;
mod schema;

pub use data_store::{DataStore, SurrealDataStore};
pub use error::DataStoreError;
pub use lance_data_store_simple::LanceDataStore;
pub use surrealdb_types::*;
