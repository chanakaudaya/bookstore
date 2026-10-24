use anyhow::Result;
use redis::{aio::ConnectionManager, Client};

pub type RedisClient = ConnectionManager;

pub async fn create_client(redis_url: &str) -> Result<ConnectionManager> {
    let client = Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}

/// Cache a JSON-serializable value with a TTL (seconds).
pub async fn cache_set<T: serde::Serialize>(
    conn: &mut ConnectionManager,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> Result<()> {
    let json = serde_json::to_string(value)?;
    redis::cmd("SET")
        .arg(key)
        .arg(json)
        .arg("EX")
        .arg(ttl_secs)
        .query_async::<_, ()>(conn)
        .await?;
    Ok(())
}

/// Fetch a cached JSON value.
pub async fn cache_get<T: serde::de::DeserializeOwned>(
    conn: &mut ConnectionManager,
    key: &str,
) -> Result<Option<T>> {
    let result: Option<String> = redis::cmd("GET").arg(key).query_async::<_, Option<String>>(conn).await?;
    match result {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

/// Delete a cached key.
pub async fn cache_del(conn: &mut ConnectionManager, key: &str) -> Result<()> {
    redis::cmd("DEL").arg(key).query_async::<_, ()>(conn).await?;
    Ok(())
}
