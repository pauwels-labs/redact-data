pub mod error;
pub mod mongodb;
pub mod redact;

use crate::data::{Data, DataCollection};
use async_trait::async_trait;
use error::StorageError;

/// The operations a storer of `Data` structs must be able to fulfill.
#[async_trait]
pub trait DataStorer: Clone + Send + Sync {
    /// Fetches one instance of a `Data` stored at that path.
    /// If the `Data` is an array, the first retrieved element is returned.
    async fn get(&self, path: &str) -> Result<Data, StorageError>;
    /// Fetches all the instances of `Data` stored at that path.
    /// Use this if retrieving an array of `Data`.
    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError>;
    /// Serializes a piece of `Data` to the the database.
    async fn create(&self, data: Data) -> Result<bool, StorageError>;
}

pub mod tests {
    use crate::{Data, DataCollection, DataStorer, StorageError};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub DataStorer {}
    #[async_trait]
    impl DataStorer for DataStorer {
        async fn get(&self, path: &str) -> Result<Data, StorageError>;
        async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
        ) -> Result<DataCollection, StorageError>;
        async fn create(&self, data: Data) -> Result<bool, StorageError>;
    }
    impl Clone for DataStorer {
        fn clone(&self) -> Self;
    }
    }

    #[test]
    fn test_unit() {
        assert!(true);
    }
}
