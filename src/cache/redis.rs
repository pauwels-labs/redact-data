// use crate::storage::{Data, DataCollection};
use crate::cache::{Cacher, error::CacheError};
use async_trait::async_trait;
use std::time::Duration;
use mobc_redis::{redis, RedisConnectionManager};
use mobc::{Connection, Pool};
use mobc_redis::redis::{AsyncCommands};

pub type MobcPool = Pool<RedisConnectionManager>;
pub type MobcCon = Connection<RedisConnectionManager>;

const CACHE_POOL_MAX_OPEN: u64 = 16;
const CACHE_POOL_MAX_IDLE: u64 = 8;
const CACHE_POOL_TIMEOUT_SECONDS: u64 = 1;
const CACHE_POOL_EXPIRE_SECONDS: u64 = 60;

/// Stores an instance of a redis-backed cache
#[derive(Clone)]
pub struct RedisCacher {
    pool: MobcPool,
}

impl RedisCacher {
    pub fn new(connection_string: &str) -> Result<RedisCacher, CacheError> {
        let client = redis::Client::open(connection_string).map_err(|e| CacheError::InternalError { source: Box::new(e), })?;
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder()
            .get_timeout(Some(Duration::from_secs(CACHE_POOL_TIMEOUT_SECONDS)))
            .max_open(CACHE_POOL_MAX_OPEN)
            .max_idle(CACHE_POOL_MAX_IDLE)
            .max_lifetime(Some(Duration::from_secs(CACHE_POOL_EXPIRE_SECONDS)))
            .build(manager);
        Ok(RedisCacher { pool })
    }

    async fn get_con(pool: &MobcPool) -> Result<MobcCon, CacheError> {
        pool.get().await.map_err(|e| {
            CacheError::InternalError { source: Box::new(e), }
        })
    }
}

#[async_trait]
impl Cacher for RedisCacher {

    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let mut con = RedisCacher::get_con(&self.pool).await?;
        con.set(key, value).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn get(&self, key: &str) -> Result<String, CacheError> {
        let mut con = RedisCacher::get_con(&self.pool).await?;
        con.get(key).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut con = RedisCacher::get_con(&self.pool).await?;
        con.exists(key).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError> {
        let mut con = RedisCacher::get_con(&self.pool).await?;
        con.expire(key, seconds).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }
}

