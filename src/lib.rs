pub mod data;
pub mod storage;

pub use data::{Data, DataCollection, DataPath};
pub use storage::{DataStorer, MongoDataStorer, StorageError};
