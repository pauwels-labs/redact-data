pub mod error;
pub mod redis;

use async_trait::async_trait;
use error::CacheError;
use std::{ops::Deref, sync::Arc};
use crate::Data;

/// The operations a redact cache struct must be able to fulfill.
#[async_trait]
pub trait DataCacher: Clone + Send + Sync {
    async fn set(&self, key: &str, value: Data) -> Result<(), CacheError>;

    /// retrieves a cached value using the key
    async fn get(&self, key: &str) -> Result<Data, CacheError>;

    /// returns a boolean indicating whether an entry exists with a given key
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// sets the cache entry's expiration in seconds
    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError>;
}

/// Allows an `Arc<DataCacher>` to act exactly like a `DataCacher`, dereferencing
/// itself and passing calls through to the underlying `DataCacher`.
#[async_trait]
impl<U> DataCacher for Arc<U>
    where
        U: DataCacher,
{
    async fn set(&self, key: &str, value: Data) -> Result<(), CacheError> {
        self.deref().set(key, value).await
    }

    async fn get(&self, key: &str) -> Result<Data, CacheError> {
        self.deref().get(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        self.deref().exists(key).await
    }

    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError> {
        self.deref().expire(key, seconds).await
    }
}

pub mod tests {
    use crate::{DataCacher, CacheError, Data};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub DataCacher {}
    #[async_trait]
    impl DataCacher for DataCacher {
        async fn set(&self, key: &str, value: Data) -> Result<(), CacheError>;
        async fn get(&self, key: &str) -> Result<Data, CacheError>;
        async fn exists(&self, key: &str) -> Result<bool, CacheError>;
        async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError>;
    }
    impl Clone for DataCacher {
        fn clone(&self) -> Self;
    }
    }

}