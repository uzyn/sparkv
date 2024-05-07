#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Config {
    max_items: usize,
    max_item_size: usize,
    max_ttl: std::time::Duration,
    default_ttl: std::time::Duration,
    auto_clear_expired: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            max_items: 10_000,
            max_item_size: 500_000,
            max_ttl: std::time::Duration::from_secs(60 * 60),
            default_ttl: std::time::Duration::from_secs(5 * 60), // 5 minutes
            auto_clear_expired: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let config: Config = Config::new();
        assert_eq!(config.max_items, 10_000);
        assert_eq!(config.max_item_size, 500_000);
        assert_eq!(config.max_ttl, std::time::Duration::from_secs(60 * 60));
        assert_eq!(config.default_ttl, std::time::Duration::from_secs(5 * 60));
        assert_eq!(config.auto_clear_expired, true);
    }
}
