#[derive(Debug, Clone)]
pub struct KvEntry<V = String> {
    pub key: String,
    pub value: V,
    pub expired_at: std::time::Instant,
}

impl<V> KvEntry<V> {
    pub fn new(key: &str, value: V, expiration: std::time::Duration) -> Self {
        let expired_at: std::time::Instant = std::time::Instant::now() + expiration;
        Self {
            key: String::from(key),
            value,
            expired_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let item = KvEntry::new(
            "key",
            String::from("value"),
            std::time::Duration::from_secs(10),
        );
        assert_eq!(item.key, "key");
        assert_eq!(item.value, "value");
        assert!(item.expired_at > std::time::Instant::now() + std::time::Duration::from_secs(9));
        assert!(item.expired_at <= std::time::Instant::now() + std::time::Duration::from_secs(10));
    }
}
