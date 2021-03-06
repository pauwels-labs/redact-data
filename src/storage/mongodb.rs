use crate::storage::{error::StorageError, Data, DataStorer};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{bson, options::ClientOptions, options::FindOneOptions, Client, Database};
use crate::DataStorerError;

/// Stores an instance of a mongodb-backed data storer
#[derive(Clone)]
pub struct MongoDataStorer {
    url: String,
    db_name: String,
    client: Client,
    db: Database,
}

impl MongoDataStorer {
    /// Instantiates a mongo-backed data storer using a URL to the mongo cluster and the
    /// name of the DB to connect to.
    pub async fn new(url: &str, db_name: &str) -> Self {
        let db_client_options = ClientOptions::parse_with_resolver_config(
            url,
            mongodb::options::ResolverConfig::cloudflare(),
        )
        .await
        .unwrap();
        let client = Client::with_options(db_client_options).unwrap();
        let db = client.database(db_name);
        MongoDataStorer {
            url: url.to_owned(),
            db_name: db_name.to_owned(),
            client,
            db,
        }
    }
}

#[async_trait]
impl DataStorer for MongoDataStorer {
    async fn get(&self, path: &str) -> Result<Data, DataStorerError> {
        let filter_options = FindOneOptions::builder().build();
        let filter = bson::doc! { "path": path };

        match self
            .db
            .collection_with_type::<Data>("data")
            .find_one(filter, filter_options)
            .await
        {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(DataStorerError::StorageError {
                source: StorageError::NotFound
            }),
            Err(e) => Err(DataStorerError::StorageError {
                source: StorageError::InternalError {
                    source: Box::new(e)
                }
            }),
        }
    }

    async fn create(&self, data: Data) -> Result<bool, DataStorerError> {
        let filter_options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();
        let filter = bson::doc! { "path": data.path() };

        match self
            .db
            .collection_with_type::<Data>("data")
            .replace_one(filter, data, filter_options)
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
