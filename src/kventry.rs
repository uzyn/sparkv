#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvEntry {
    pub key: String,
    pub value: String,
    pub expired_at: std::time::Instant,
}

impl KvEntry {
    pub fn new(key: &str, value: &str, expiration: std::time::Duration) -> Self {
        let expired_at: std::time::Instant = std::time::Instant::now() + expiration;
        Self {
            key: String::from(key),
            value: String::from(value),
            expired_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expired_at < std::time::Instant::now()
    }
}

impl Ord for KvEntry {
    // Match in opposite direction (min-heap), so that the smallest element is at the top.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.expired_at.cmp(&other.expired_at) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for KvEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let item = KvEntry::new("key", "value", std::time::Duration::from_secs(10));
        assert_eq!(item.key, "key");
        assert_eq!(item.value, "value");
        assert!(item.expired_at > std::time::Instant::now() + std::time::Duration::from_secs(9));
        assert!(item.expired_at <= std::time::Instant::now() + std::time::Duration::from_secs(10));
    }
}
