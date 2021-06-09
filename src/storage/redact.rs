use crate::{Data, DataCollection, DataStorer, StorageError, DataStorerError};
use async_trait::async_trait;

/// Stores an instance of a redact-backed data storer.
/// The redact-store server is an example implementation of a redact storage backing.
#[derive(Clone)]
pub struct RedactDataStorer {
    url: String,
}

impl RedactDataStorer {
    /// Instantiates a redact-backed data storer using a URL to the storage server.
    pub fn new(url: &str) -> RedactDataStorer {
        RedactDataStorer {
            url: url.to_owned(),
        }
    }
}

#[async_trait]
impl DataStorer for RedactDataStorer {
    async fn get(&self, path: &str) -> Result<Data, DataStorerError> {
        match reqwest::get(&format!("{}/data/{}", self.url, path)).await {
            Ok(r) => Ok(r
                .json::<Data>()
                .await
                .map_err(|source| DataStorerError::StorageError {
                    source: StorageError::InternalError {
                        source: Box::new(source),
                    }
                })?),
            Err(e) => Err(DataStorerError::StorageError {
                source: StorageError::InternalError {
                    source: Box::new(e)
                }
            }),
        }
    }

    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, DataStorerError> {
        match reqwest::get(&format!(
            "{}/data/{}?skip={}&page_size={}",
            self.url, path, skip, page_size
        ))
        .await
        {
            Ok(r) => Ok(r.json::<DataCollection>().await.map_err(|source| {
                DataStorerError::StorageError {
                    source: StorageError::InternalError {
                        source: Box::new(source),
                    }
                }
            })?),
            Err(e) => Err(DataStorerError::StorageError {
                source: StorageError::InternalError {
                    source: Box::new(e)
                }
            }),
        }
    }

    async fn create(&self, data: Data) -> Result<bool, DataStorerError> {
        match reqwest::Client::new()
            .post(&format!("{}/data?path={}", self.url, data.path()))
            .json(&data)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(DataStorerError::StorageError {
                source: StorageError::InternalError {
                    source: Box::new(e)
                }
            }),
        }
    }
}
