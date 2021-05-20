pub mod error;
pub mod mongodb;
pub mod redact;

use crate::data::{Data, DataCollection};
use async_trait::async_trait;
use error::StorageError;
use std::{ops::Deref, sync::Arc};

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

/// Allows an `Arc<DataStorer>` to act exactly like a `DataStorer`, dereferencing
/// itself and passing calls through to the underlying `DataStorer`.
#[async_trait]
impl<U> DataStorer for Arc<U>
where
    U: DataStorer,
{
    async fn get(&self, path: &str) -> Result<Data, StorageError> {
        self.deref().get(path).await
    }

    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError> {
        self.deref().get_collection(path, skip, page_size).await
    }

    async fn create(&self, value: Data) -> Result<bool, StorageError> {
        self.deref().create(value).await
    }
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
