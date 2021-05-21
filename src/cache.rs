pub mod error;
pub mod redact;
use crate::data::{Data, DataCollection};

use async_trait::async_trait;
use error::CacheError;
use std::{ops::Deref, sync::Arc};

/// The operations a fetch cache struct must be able to fulfill.
#[async_trait]
pub trait FetchCacher: Clone + Send + Sync {
    async fn set(&self, key: &str, value: &str) -> Result<(), RedisServiceError>;

    /// retrieves a cached value using the key
    async fn get(&self, key: &str) -> Result<String, RedisServiceError>;

    /// returns a boolean indicating whether an entry exists with a given key
    async fn exists(&self, key: &str) -> Result<bool, RedisServiceError>;

    /// sets the cache entry's expiration in seconds
    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, RedisServiceError>;


    async fn set(&self, fetch_id: &str, page_number: i64, value: &Vec<Data>, ttl_seconds: usize) -> Result<(), RedisClientError>;
    async fn get_index(&self, key: &str, index: i64) -> Result<Data, RedisClientError>;
    async fn exists_index(&self, key: &str, index: i64) -> Result<bool, RedisClientError>;
    fn get_collection_size(&self) -> u8;
}

/// Allows an `Arc<FetchCacher>` to act exactly like a `FetchCacher`, dereferencing
/// itself and passing calls through to the underlying `FetchCacher`.
#[async_trait]
impl<U> FetchCacher for Arc<U>
    where
        U: FetchCacher,
{
    async fn set(&self, key: &str, value: &str) -> Result<(), RedisServiceError> {
        self.deref().set(key, value).await
    }

    async fn get(&self, key: &str) -> Result<String, RedisServiceError> {
        self.deref().get(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, RedisServiceError> {
        self.exists().set(key).await
    }

    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, RedisServiceError> {
        self.deref().expire(key, seconds).await
    }
}

pub mod tests {
    use crate::{FetchCacher, CacheError};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub FetchCacher {}
    #[async_trait]
    impl FetchCacher for FetchCacher {
        async fn set(&self, key: &str, value: &str) -> Result<(), RedisServiceError>;
        async fn get(&self, key: &str) -> Result<String, RedisServiceError>;
        async fn exists(&self, key: &str) -> Result<bool, RedisServiceError>;
        async fn expire(&self, key: &str, seconds: usize) -> Result<bool, RedisServiceError>;
    }
    impl Clone for FetchCacher {
        fn clone(&self) -> Self;
    }
    }

}