use crate::{Data, DataCollection, DataStorer, StorageError};
use async_trait::async_trait;

#[derive(Clone)]
pub struct RedactDataStorer {
    url: String,
}

impl RedactDataStorer {
    pub fn new(url: &str) -> RedactDataStorer {
        RedactDataStorer {
            url: url.to_owned(),
        }
    }
}

#[async_trait]
impl DataStorer for RedactDataStorer {
    async fn get(&self, path: &str) -> Result<Data, StorageError> {
        match reqwest::get(&format!("{}/data/{}", self.url, path)).await {
            Ok(r) => Ok(r
                .json::<Data>()
                .await
                .map_err(|source| StorageError::InternalError {
                    source: Box::new(source),
                })?),
            Err(source) => Err(StorageError::InternalError {
                source: Box::new(source),
            }),
        }
    }

    async fn get_collection(
        &self,
        path: &str,
        skip: i64,
        page_size: i64,
    ) -> Result<DataCollection, StorageError> {
        match reqwest::get(&format!(
            "{}/data/{}?skip={}&page_size={}",
            self.url, path, skip, page_size
        ))
        .await
        {
            Ok(r) => Ok(r.json::<DataCollection>().await.map_err(|source| {
                StorageError::InternalError {
                    source: Box::new(source),
                }
            })?),
            Err(source) => Err(StorageError::InternalError {
                source: Box::new(source),
            }),
        }
    }

    async fn create(&self, data: Data) -> Result<bool, StorageError> {
        match reqwest::Client::new()
            .post(&format!("{}/data?path={}", self.url, data.path))
            .json(&data)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(source) => Err(StorageError::InternalError {
                source: Box::new(source),
            }),
        }
    }
}
