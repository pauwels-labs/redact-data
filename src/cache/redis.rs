use crate::cache::{DataCacher, error::CacheError};
use async_trait::async_trait;
use std::time::Duration;
use mobc_redis::{redis, RedisConnectionManager};
use mobc::{Connection, Pool};
use mobc_redis::redis::{AsyncCommands, ToRedisArgs, FromRedisValue, RedisWrite, RedisResult, Value, from_redis_value, ErrorKind};
use crate::Data;

pub type MobcPool = Pool<RedisConnectionManager>;
pub type MobcCon = Connection<RedisConnectionManager>;

/// Stores an instance of a redis-backed cache
#[derive(Clone)]
pub struct RedisDataCacher {
    pool: MobcPool,
    cache_default_key_espiration_seconds: u64
}

/// Stores the configuration values used to construct a RedisDataCacher
pub struct RedisCacheConfig<'a> {
    connection_string: &'a str,
    cache_pool_timeout_seconds: u64,
    cache_pool_max_open: u64,
    cache_pool_max_idle: u64,
    cache_pool_expire_seconds: u64,
    cache_default_key_expiration_seconds: u64
}

impl RedisDataCacher {
    pub fn new(config: RedisCacheConfig) -> Result<RedisDataCacher, CacheError> {
        let client = redis::Client::open(config.connection_string).map_err(|e| CacheError::InternalError { source: Box::new(e), })?;
        let manager = RedisConnectionManager::new(client);
        let pool = Pool::builder()
            .get_timeout(Some(Duration::from_secs(config.cache_pool_timeout_seconds)))
            .max_open(config.cache_pool_max_open)
            .max_idle(config.cache_pool_max_idle)
            .max_lifetime(Some(Duration::from_secs(config.cache_pool_expire_seconds)))
            .build(manager);
        Ok(RedisDataCacher {
            pool,
            cache_default_key_espiration_seconds: config.cache_default_key_expiration_seconds
        })
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
        let s: String = serde_json::to_string(self).unwrap();
        out.write_arg(s.as_bytes())
    }
}

impl FromRedisValue for Data {
    fn from_redis_value(v: &Value) -> RedisResult<Data> {
        let s: String = from_redis_value(v)?;
        let d: Data = serde_json::from_str(&s).map_err(|_e| (ErrorKind::TypeError, "deserialization error!"))?;
        Ok(d)
    }
}

#[async_trait]
impl DataCacher for RedisDataCacher {

    async fn set(&self, key: &str, value: Data) -> Result<(), CacheError> {
        let mut con = RedisDataCacher::get_con(&self.pool).await?;
        con.set_ex(key, value, self.get_default_key_expiration_seconds())
            .await
            .map_err(|e| CacheError::InternalError { source: Box::new(e), })
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

    fn get_default_key_expiration_seconds(&self) -> usize {
        self.cache_default_key_espiration_seconds as usize
    }
}
