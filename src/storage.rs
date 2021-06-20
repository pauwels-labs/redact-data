pub mod error;
pub mod mongodb;
pub mod redact;

use crate::data::{Data, DataCollection};
use async_trait::async_trait;
use std::{ops::Deref, sync::Arc};
use crate::{DataCacher};
use crate::storage::error::DataStorerError;


/// The operations a storer of `Data` structs must be able to fulfill.
#[async_trait]
pub trait DataStorer: Clone + Send + Sync {
    /// Fetches one instance of a `Data` stored at that path.
    /// If the `Data` is an array, the first retrieved element is returned.
    async fn get(&self, path: &str) -> Result<Data, DataStorerError>;
    /// Serializes a piece of `Data` to the the database.
    async fn create(&self, data: Data) -> Result<bool, DataStorerError>;
}

/// Allows an `Arc<DataStorer>` to act exactly like a `DataStorer`, dereferencing
/// itself and passing calls through to the underlying `DataStorer`.
#[async_trait]
impl<U> DataStorer for Arc<U>
where
    U: DataStorer,
{
    async fn get(&self, path: &str) -> Result<Data, DataStorerError> {
        self.deref().get(path).await
    }

    async fn create(&self, value: Data) -> Result<bool, DataStorerError> {
        self.deref().create(value).await
    }
}

/// Stores an instance of a redact-backed data storer, including a cache.
#[derive(Clone)]
pub struct CachedDataStorer<T: DataStorer, V: DataCacher> {
    storer: T,
    cacher: V
}

impl<T: DataStorer, V: DataCacher> CachedDataStorer<T, V> {
    /// Instantiates a cached redact-backed data storer using an existing storer and cacher.
    pub fn new(storer: T, cacher: V) -> CachedDataStorer<T,V>
        where
            T: DataStorer,
            V: DataCacher
    {
        CachedDataStorer {
            storer,
            cacher,
        }
    }
}

#[async_trait]
impl<T: DataStorer, V: DataCacher> DataStorer for CachedDataStorer<T, V> {
    async fn get(&self, path: &str) -> Result<Data, DataStorerError> {
        let cache_hit = self.cacher.exists(path).await?;
        if cache_hit {
            self.cacher.expire(
                path,
                self.cacher.get_default_key_expiration_seconds())
                .await?;
            self.cacher.get(path).await.map_err(|source| {
                DataStorerError::CacheError {
                    source: source
                }
            })
        } else {
            let res = self.storer.get(path).await?;
            self.cacher.set(path, res.clone()).await?;
            Ok(res)
        }
    }

    async fn create(&self, value: Data) -> Result<bool, DataStorerError> {
        self.storer.create(value).await?;
        self.cacher.set(&value.path(), value.clone()).await?;
        Ok(true)
    }
}

pub mod tests {
    use crate::{Data, DataCollection, DataStorer, DataStorerError, MockDataCacher, CachedDataStorer, DataValue, UnencryptedDataValue};
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
    pub DataStorer {}
    #[async_trait]
    impl DataStorer for DataStorer {
        async fn get(&self, path: &str) -> Result<Data, DataStorerError>;
        async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
        ) -> Result<DataCollection, DataStorerError>;
        async fn create(&self, data: Data) -> Result<bool, DataStorerError>;
    }
    impl Clone for DataStorer {
        fn clone(&self) -> Self;
    }
    }

    #[tokio::test]
    async fn test_cached_data_storer_get_cache_hit() {
        let mut storer = MockDataStorer::new();
        let mut cacher = MockDataCacher::new();

        cacher.expect_exists()
            .times(1)
            .returning(|_| { Ok(true) });
        cacher.expect_expire()
            .times(1)
            .returning(|_, _| { Ok(true) });
        cacher.expect_get_default_key_expiration_seconds()
            .returning(|| {60});
        cacher.expect_get()
            .times(1)
            .returning(|_| {
                Ok( Data::new(".path.", DataValue::Unencrypted(UnencryptedDataValue::I64(1))))
            });

        storer.expect_get()
            .times(0);
        cacher.expect_set()
            .times(0);

        let cached_storer = CachedDataStorer::new(storer, cacher);
        let result = cached_storer.get("abc").await.unwrap();
        assert_eq!(".path.", result.path());
    }

    #[tokio::test]
    async fn test_cached_data_storer_get_cache_miss() {
        let mut storer = MockDataStorer::new();
        let mut cacher = MockDataCacher::new();

        cacher.expect_exists()
            .times(1)
            .returning(|_| { Ok(false) });
        storer.expect_get()
            .times(1)
            .returning(|_| {
                Ok(Data::new(".path.", DataValue::Unencrypted(UnencryptedDataValue::I64(1))))
            });
        cacher.expect_set()
            .times(1)
            .withf(|path: &str, d: &Data| d.path() == ".path." && path == ".path.")
            .returning(|_, _| Ok(()));

        cacher.expect_expire()
            .times(0);
        cacher.expect_get()
            .times(0);


        let cached_storer = CachedDataStorer::new(storer, cacher);
        let result = cached_storer.get(".path.").await.unwrap();
        assert_eq!(".path.", result.path());
    }
}
