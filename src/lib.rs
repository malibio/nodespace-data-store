mod conversions;
mod data_store;
mod error;
mod lance_data_store_simple;

pub use data_store::{DataStore, SurrealDataStore};
pub use error::DataStoreError;
pub use lance_data_store_simple::LanceDataStore;
