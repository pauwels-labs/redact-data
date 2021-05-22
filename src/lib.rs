//! # redact-data
//!
//! The `redact-data` crate contains all of the interfaces, data structures,
//! and abstractions necessary to work with a unit of data in the redact world.
//! It also contains implementations of the storage interface for storing and
//! retrieving redact data with a variety of sources.
//!
//! File directory:
//! - data.rs: data definitions and conversions
//! - storage.rs: trait for a data type that stores Data
//! - storage/error.rs: error types for the storage abstractions
//! - storage/mongodb.rs: storage implentation for mongodb
//! - storage/redact.rs: storage implementation for a redact-store server

mod data;
pub mod storage;
pub mod cache;

pub use data::{Data, DataCollection, DataPath, UnencryptedDataValue};
pub use storage::{
    error::StorageError, mongodb::MongoDataStorer, redact::RedactDataStorer, DataStorer,
};pub use cache::{
    error::CacheError
}
