mod conversions;
mod data_store;
mod error;
mod surrealdb_types;

// LanceDB implementation modules (for migration)
mod lance_data_store;
pub mod migration;
mod schema;

pub use data_store::{DataStore, SurrealDataStore};
pub use error::DataStoreError;
pub use lance_data_store::{LanceConfig, LanceDataStore, UniversalNode};
pub use surrealdb_types::*;
