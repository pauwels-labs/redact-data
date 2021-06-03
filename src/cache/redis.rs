use crate::cache::{DataCacher, error::CacheError};
use async_trait::async_trait;
use std::time::Duration;
use mobc_redis::{redis, RedisConnectionManager};
use mobc::{Connection, Pool};
use mobc_redis::redis::{AsyncCommands, ToRedisArgs, FromRedisValue, RedisWrite, RedisResult, Value, from_redis_value, RedisError};
use crate::Data;
use std::io::{Error, ErrorKind};

pub type MobcPool = Pool<RedisConnectionManager>;
pub type MobcCon = Connection<RedisConnectionManager>;

const CACHE_POOL_MAX_OPEN: u64 = 16;
const CACHE_POOL_MAX_IDLE: u64 = 8;
const CACHE_POOL_TIMEOUT_SECONDS: u64 = 1;
const CACHE_POOL_EXPIRE_SECONDS: u64 = 60;

/// Stores an instance of a redis-backed cache
#[derive(Clone)]
pub struct RedisDataCacher {
    pool: MobcPool,
}

impl RedisDataCacher {
    pub fn new(connection_string: &str) -> Result<RedisDataCacher, CacheError> {
        let client = redis::Client::open(connection_string).map_err(|e| CacheError::InternalError { source: Box::new(e), })?;
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder()
            .get_timeout(Some(Duration::from_secs(CACHE_POOL_TIMEOUT_SECONDS)))
            .max_open(CACHE_POOL_MAX_OPEN)
            .max_idle(CACHE_POOL_MAX_IDLE)
            .max_lifetime(Some(Duration::from_secs(CACHE_POOL_EXPIRE_SECONDS)))
            .build(manager);
        Ok(RedisDataCacher { pool })
    }

    async fn get_con(pool: &MobcPool) -> Result<MobcCon, CacheError> {
        pool.get().await.map_err(|e| {
            CacheError::InternalError { source: Box::new(e), }
        })
    }
}

impl ToRedisArgs for Data {
    fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + RedisWrite,
    {
        out.write_arg(self.to_string().as_bytes())
    }
}

impl FromRedisValue for Data {
    fn from_redis_value(v: &Value) -> RedisResult<Data> {
        let s: String = from_redis_value(v)?;
        let d: Data = serde_json::from_str(&s).map_err(|_e| Error::new(ErrorKind::Other, "deserialization error!"))?;
        Ok(d)
    }
}

#[async_trait]
impl DataCacher for RedisDataCacher {

    async fn set(&self, key: &str, value: Data) -> Result<(), CacheError> {
        let mut con = RedisDataCacher::get_con(&self.pool).await?;
        con.set(key, value).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn get(&self, key: &str) -> Result<Data, CacheError> {
        let mut con = RedisDataCacher::get_con(&self.pool).await?;
        con.get(key).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut con = RedisDataCacher::get_con(&self.pool).await?;
        con.exists(key).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }

    async fn expire(&self, key: &str, seconds: usize) -> Result<bool, CacheError> {
        let mut con = RedisDataCacher::get_con(&self.pool).await?;
        con.expire(key, seconds).await.map_err(|e| CacheError::InternalError { source: Box::new(e), })
    }
}

