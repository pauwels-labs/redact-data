pub mod error;
pub mod mongodb;
pub mod redact;

use crate::data::{SealedData, SealedDataCollection};
use async_trait::async_trait;
use error::StorageError;
use std::{ops::Deref, sync::Arc};

/// The operations a storer of `Data` structs must be able to fulfill.
#[async_trait]
pub trait SealedDataStorer: Clone + Send + Sync {
    /// Fetches one instance of a `Data` stored at that path.
    /// If the `Data` is an array, the first retrieved element is returned.
    async fn get(&self, path: &str) -> Result<SealedData, StorageError>;
    /// Fetches all the instances of `Data` stored at that path.
    /// Use this if retrieving an array of `Data`.
    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<SealedDataCollection, StorageError>;
    /// Serializes a piece of `Data` to the the database.
    async fn create(&self, data: SealedData) -> Result<bool, StorageError>;
}

/// Allows an `Arc<DataStorer>` to act exactly like a `DataStorer`, dereferencing
/// itself and passing calls through to the underlying `DataStorer`.
#[async_trait]
impl<U> SealedDataStorer for Arc<U>
where
    U: SealedDataStorer,
{
    async fn get(&self, path: &str) -> Result<SealedData, StorageError> {
        self.deref().get(path).await
    }

    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<SealedDataCollection, StorageError> {
        self.deref().get_collection(path, skip, page_size).await
    }

    async fn create(&self, value: SealedData) -> Result<bool, StorageError> {
        self.deref().create(value).await
    }
}

pub mod tests {
    use crate::{SealedData, SealedDataCollection, SealedDataStorer, StorageError};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub SealedDataStorer {}
    #[async_trait]
    impl SealedDataStorer for SealedDataStorer {
        async fn get(&self, path: &str) -> Result<SealedData, StorageError>;
        async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
        ) -> Result<SealedDataCollection, StorageError>;
        async fn create(&self, data: SealedData) -> Result<bool, StorageError>;
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
