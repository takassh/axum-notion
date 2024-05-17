use redis::{Commands, RedisResult};

#[derive(Clone, Debug)]
pub struct SessionRepository {
    pub redis: redis::Client,
}

impl SessionRepository {
    pub fn new(redis: redis::Client) -> Self {
        Self { redis }
    }
}

impl SessionRepository {
    pub fn get(&self, user_id: &str) -> anyhow::Result<i32> {
        let mut con = self.redis.get_connection()?;
        let result: RedisResult<i32> = con.get(user_id);

        Ok(result?)
    }

    pub fn increment(&self, user_id: &str) -> anyhow::Result<i32> {
        let mut con = self.redis.get_connection()?;
        let count: RedisResult<String> = con.get(user_id);
        let count = match count {
            Ok(count) => Ok(count),
            Err(err) => match err.kind() {
                redis::ErrorKind::TypeError => Ok("0".to_string()),
                _ => Err(err),
            },
        }?;
        let count = count.parse::<i32>()?;
        con.set_ex(user_id, count + 1, 60 * 60)?;

        Ok(count + 1)
    }
}
