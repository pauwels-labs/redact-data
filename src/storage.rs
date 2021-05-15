pub mod error;
pub mod mongodb;

pub use self::mongodb::MongoDataStorer;
use crate::data::{Data, DataCollection};
use async_trait::async_trait;
pub use error::StorageError;

#[async_trait]
pub trait DataStorer: Clone + Send + Sync {
    async fn get(&self, path: &str) -> Result<Data, StorageError>;
    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError>;
    async fn create(&self, data: Data) -> Result<bool, StorageError>;
}
