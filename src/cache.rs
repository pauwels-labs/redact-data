pub mod error;
pub mod redis;

use async_trait::async_trait;
use error::CacheError;
use std::{ops::Deref, sync::Arc};

/// The operations a redact cache struct must be able to fulfill.
#[async_trait]
pub trait Cacher: Clone + Send + Sync {
    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError>;

    /// retrieves a cached value using the key
    async fn get(&self, key: &str) -> Result<String, CacheError>;

    /// returns a boolean indicating whether an entry exists with a given key
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// sets the cache entry's expiration in seconds
    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError>;
}

/// Allows an `Arc<FetchCacher>` to act exactly like a `FetchCacher`, dereferencing
/// itself and passing calls through to the underlying `FetchCacher`.
#[async_trait]
impl<U> Cacher for Arc<U>
    where
        U: Cacher,
{
    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        self.deref().set(key, value).await
    }

    async fn get(&self, key: &str) -> Result<String, CacheError> {
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
    use crate::{Cacher, CacheError};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub Cacher {}
    #[async_trait]
    impl Cacher for Cacher {
        async fn set(&self, key: &str, value: &str) -> Result<(), CacheError>;
        async fn get(&self, key: &str) -> Result<String, CacheError>;
        async fn exists(&self, key: &str) -> Result<bool, CacheError>;
        async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError>;
    }
    impl Clone for Cacher {
        fn clone(&self) -> Self;
    }
    }

}