mod config;
mod error;
mod kventry;
mod value_size;

pub use config::Config;
pub use error::Error;
pub use kventry::KvEntry;
pub use value_size::ValueSize;

pub struct SparKV<V = String> {
    pub config: Config,
    data: std::collections::HashMap<String, KvEntry<V>>,
    expiries: std::collections::BTreeMap<(std::time::Instant, String), ()>,
}

impl<V> SparKV<V> {
    pub fn new() -> Self {
        let config = Config::new();
        SparKV::with_config(config)
    }

    pub fn with_config(config: Config) -> Self {
        SparKV {
            config,
            data: std::collections::HashMap::new(),
            expiries: std::collections::BTreeMap::new(),
        }
    }

    // Only returns if it is not yet expired
    pub fn get_item(&self, key: &str) -> Option<&KvEntry<V>> {
        let item = self.data.get(key)?;
        if item.expired_at > std::time::Instant::now() {
            Some(item)
        } else {
            None
        }
    }

    pub fn delete(&mut self, key: &str) -> Option<V> {
        self.clear_expired_if_auto();
        let item = self.data.remove(key)?;
        self.expiries.remove(&(item.expired_at, item.key));
        Some(item.value)
    }

    /// Number of physically-present entries.
    ///
    /// This is an O(1) count that may include expired-but-unswept entries.
    /// For the live view use [`get`](Self::get) / [`get_item`](Self::get_item),
    /// or call [`clear_expired`](Self::clear_expired) first.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether there are no physically-present entries.
    ///
    /// O(1) and physical, mirroring [`len`](Self::len): it may still report
    /// `false` while every remaining entry is expired-but-unswept.
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    /// Whether a live (non-expired) entry exists for `key`.
    ///
    /// Expiry-aware and O(1): consistent with [`get`](Self::get), it returns
    /// `false` for an expired-but-unswept entry.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_item(key).is_some()
    }

    pub fn clear_expired(&mut self) -> usize {
        let now = std::time::Instant::now();
        let mut cleared_count: usize = 0;
        while self
            .expiries
            .first_key_value()
            .is_some_and(|((expired_at, _), _)| *expired_at < now)
        {
            if let Some(((_, key), _)) = self.expiries.pop_first() {
                self.data.remove(&key); // not self.delete() -> avoids re-entrant auto-clear recursion
                cleared_count += 1;
            }
        }
        cleared_count
    }

    fn clear_expired_if_auto(&mut self) {
        if self.config.auto_clear_expired {
            self.clear_expired();
        }
    }

    fn ensure_capacity(&self) -> Result<(), Error> {
        if self.len() >= self.config.max_items {
            return Err(Error::CapacityExceeded);
        }
        Ok(())
    }

    fn ensure_capacity_ignore_key(&self, key: &str) -> Result<(), Error> {
        // Physical presence (not the expiry-aware public `contains_key`) so
        // overwrite/capacity semantics stay identical regardless of expiry.
        if self.data.contains_key(key) {
            return Ok(());
        }
        self.ensure_capacity()
    }

    fn ensure_max_ttl(&self, ttl: std::time::Duration) -> Result<(), Error> {
        if ttl > self.config.max_ttl {
            return Err(Error::TTLTooLong);
        }
        Ok(())
    }
}

impl<V: ValueSize> SparKV<V> {
    pub fn set(&mut self, key: &str, value: V) -> Result<(), Error> {
        self.set_with_ttl(key, value, self.config.default_ttl)
    }

    pub fn set_with_ttl(
        &mut self,
        key: &str,
        value: V,
        ttl: std::time::Duration,
    ) -> Result<(), Error> {
        self.clear_expired_if_auto();
        self.ensure_capacity_ignore_key(key)?;
        self.ensure_item_size(&value)?;
        self.ensure_max_ttl(ttl)?;

        let item: KvEntry<V> = KvEntry::new(key, value, ttl);
        let exp_at = item.expired_at;
        if let Some(old) = self.data.insert(item.key.clone(), item) {
            self.expiries.remove(&(old.expired_at, key.to_string()));
        }
        self.expiries.insert((exp_at, key.to_string()), ());
        Ok(())
    }

    fn ensure_item_size(&self, value: &V) -> Result<(), Error> {
        if let Some(max_item_size) = self.config.max_item_size {
            if value.value_size() > max_item_size {
                return Err(Error::ItemSizeExceeded);
            }
        }
        Ok(())
    }
}

impl<V: Clone> SparKV<V> {
    pub fn get(&self, key: &str) -> Option<V> {
        let item = self.get_item(key)?;
        Some(item.value.clone())
    }
}

impl<V> Default for SparKV<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparkv_config() {
        let config: Config = Config::new();
        assert_eq!(config.max_items, 10_000);
        assert_eq!(config.max_item_size, Some(500_000));
        assert_eq!(config.max_ttl, std::time::Duration::from_secs(60 * 60));
    }

    #[test]
    fn test_sparkv_new_with_config() {
        let config: Config = Config::new();
        let sparkv: SparKV<String> = SparKV::with_config(config);
        assert_eq!(sparkv.config, config);
    }

    #[test]
    fn test_len_is_empty() {
        let mut sparkv = SparKV::new();
        assert_eq!(sparkv.len(), 0);
        assert!(sparkv.is_empty());

        _ = sparkv.set("keyA", String::from("value"));
        assert_eq!(sparkv.len(), 1);
        assert!(!sparkv.is_empty());
    }

    #[test]
    fn test_expiry_index_stays_bounded_to_live_set() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = false;
        let mut sparkv = SparKV::with_config(config);

        for i in 0..1000 {
            _ = sparkv.set("same-key", format!("value{i}"));
        }
        assert_eq!(sparkv.get("same-key"), Some(String::from("value999")));
        assert_eq!(sparkv.expiries.len(), 1);
        assert_eq!(sparkv.data.len(), 1);

        let deleted = sparkv.delete("same-key");
        assert_eq!(deleted, Some(String::from("value999")));
        assert_eq!(sparkv.expiries.len(), 0);
        assert_eq!(sparkv.data.len(), 0);
    }

    #[test]
    fn test_set_get() {
        let mut sparkv = SparKV::new();
        _ = sparkv.set("keyA", String::from("value"));
        assert_eq!(sparkv.get("keyA"), Some(String::from("value")));
        assert_eq!(sparkv.expiries.len(), 1);

        // Overwrite the value: the index is replaced 1:1, not appended to.
        _ = sparkv.set("keyA", String::from("value2"));
        assert_eq!(sparkv.get("keyA"), Some(String::from("value2")));
        assert_eq!(sparkv.expiries.len(), 1);

        assert!(sparkv.get("non-existent").is_none());
    }

    #[test]
    fn test_generic_value_type() {
        let mut sparkv: SparKV<i64> = SparKV::new();
        _ = sparkv.set("answer", 42);
        assert_eq!(sparkv.get("answer"), Some(42));

        _ = sparkv.set("answer", -7);
        assert_eq!(sparkv.get("answer"), Some(-7));

        assert_eq!(sparkv.delete("answer"), Some(-7));
        assert!(sparkv.get("answer").is_none());
    }

    #[test]
    fn test_generic_item_size_enforced() {
        let mut config: Config = Config::new();
        config.max_item_size = Some(4); // i64 is 8 bytes
        let mut sparkv: SparKV<i64> = SparKV::with_config(config);

        let set_result = sparkv.set("too-big", 1);
        assert_eq!(set_result.unwrap_err(), Error::ItemSizeExceeded);
    }

    #[test]
    fn test_unbounded_item_size_when_none() {
        let mut config: Config = Config::new();
        config.max_item_size = None;
        let mut sparkv = SparKV::with_config(config);

        let huge = "x".repeat(1_000_000);
        let set_result = sparkv.set("huge", huge.clone());
        assert!(set_result.is_ok());
        assert_eq!(sparkv.get("huge"), Some(huge));
    }

    #[test]
    fn test_get_item() {
        let mut sparkv = SparKV::new();
        let item = KvEntry::new(
            "keyARaw",
            String::from("value99"),
            std::time::Duration::from_secs(1),
        );
        sparkv.data.insert(item.key.clone(), item);
        let get_result = sparkv.get_item("keyARaw");
        let unwrapped = get_result.unwrap();

        assert!(get_result.is_some());
        assert_eq!(unwrapped.key, "keyARaw");
        assert_eq!(unwrapped.value, "value99");

        assert!(sparkv.get_item("non-existent").is_none());
    }

    #[test]
    fn test_get_item_return_none_if_expired() {
        let mut sparkv = SparKV::new();
        _ = sparkv.set_with_ttl(
            "kkk",
            String::from("value"),
            std::time::Duration::from_millis(50),
        );
        assert_eq!(sparkv.get("kkk"), Some(String::from("value")));

        std::thread::sleep(std::time::Duration::from_millis(60));
        assert_eq!(sparkv.get("kkk"), None);
    }

    #[test]
    fn test_set_should_fail_if_capacity_exceeded() {
        let mut config: Config = Config::new();
        config.max_items = 2;

        let mut sparkv = SparKV::with_config(config);
        let mut set_result = sparkv.set("keyA", String::from("value"));
        assert!(set_result.is_ok());
        assert_eq!(sparkv.get("keyA"), Some(String::from("value")));

        set_result = sparkv.set("keyB", String::from("value2"));
        assert!(set_result.is_ok());

        set_result = sparkv.set("keyC", String::from("value3"));
        assert!(set_result.is_err());
        assert_eq!(set_result.unwrap_err(), Error::CapacityExceeded);
        assert!(sparkv.get("keyC").is_none());

        // Overwrite existing key should not err
        set_result = sparkv.set("keyB", String::from("newValue1234"));
        assert!(set_result.is_ok());
        assert_eq!(sparkv.get("keyB"), Some(String::from("newValue1234")));
    }

    #[test]
    fn test_set_with_ttl() {
        let mut sparkv = SparKV::new();
        _ = sparkv.set("longest", String::from("value"));
        _ = sparkv.set_with_ttl(
            "longer",
            String::from("value"),
            std::time::Duration::from_secs(2),
        );
        _ = sparkv.set_with_ttl(
            "shorter",
            String::from("value"),
            std::time::Duration::from_secs(1),
        );

        assert_eq!(sparkv.get("longer"), Some(String::from("value")));
        assert_eq!(sparkv.get("shorter"), Some(String::from("value")));
        assert!(
            sparkv.get_item("longer").unwrap().expired_at
                > sparkv.get_item("shorter").unwrap().expired_at
        );
        assert!(
            sparkv.get_item("longest").unwrap().expired_at
                > sparkv.get_item("longer").unwrap().expired_at
        );
    }

    #[test]
    fn test_ensure_max_ttl() {
        let mut config: Config = Config::new();
        config.max_ttl = std::time::Duration::from_secs(3600);
        config.default_ttl = std::time::Duration::from_secs(5000);
        let mut sparkv = SparKV::with_config(config);

        let set_result_long_def =
            sparkv.set("default is longer than max", String::from("should fail"));
        assert!(set_result_long_def.is_err());
        assert_eq!(set_result_long_def.unwrap_err(), Error::TTLTooLong);

        let set_result_ok = sparkv.set_with_ttl(
            "shorter",
            String::from("ok"),
            std::time::Duration::from_secs(3599),
        );
        assert!(set_result_ok.is_ok());

        let set_result_ok_2 = sparkv.set_with_ttl(
            "exact",
            String::from("ok"),
            std::time::Duration::from_secs(3600),
        );
        assert!(set_result_ok_2.is_ok());

        let set_result_not_ok = sparkv.set_with_ttl(
            "not",
            String::from("not ok"),
            std::time::Duration::from_secs(3601),
        );
        assert!(set_result_not_ok.is_err());
        assert_eq!(set_result_not_ok.unwrap_err(), Error::TTLTooLong);
    }

    #[test]
    fn test_delete() {
        let mut sparkv = SparKV::new();
        _ = sparkv.set("keyA", String::from("value"));
        assert_eq!(sparkv.get("keyA"), Some(String::from("value")));
        assert_eq!(sparkv.expiries.len(), 1);

        let deleted_value = sparkv.delete("keyA");
        assert_eq!(deleted_value, Some(String::from("value")));
        assert!(sparkv.get("keyA").is_none());
        assert_eq!(sparkv.expiries.len(), 0); // index entry removed too
    }

    #[test]
    fn test_clear_expired() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = false;
        let mut sparkv = SparKV::with_config(config);
        _ = sparkv.set_with_ttl(
            "not-yet-expired",
            String::from("v"),
            std::time::Duration::from_secs(90),
        );
        _ = sparkv.set_with_ttl(
            "expiring",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        _ = sparkv.set_with_ttl(
            "not-expired",
            String::from("value"),
            std::time::Duration::from_secs(60),
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
        assert_eq!(sparkv.len(), 3);

        let cleared_count = sparkv.clear_expired();
        assert_eq!(cleared_count, 1);
        assert_eq!(sparkv.len(), 2);

        assert_eq!(sparkv.clear_expired(), 0);
    }

    #[test]
    fn test_clear_expired_with_overwritten_key() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = false;
        let mut sparkv = SparKV::with_config(config);
        _ = sparkv.set_with_ttl(
            "no-longer",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        _ = sparkv.set_with_ttl(
            "no-longer",
            String::from("v"),
            std::time::Duration::from_secs(90),
        );
        _ = sparkv.set_with_ttl(
            "not-expired",
            String::from("value"),
            std::time::Duration::from_secs(60),
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
        assert_eq!(sparkv.expiries.len(), 2); // overwriting key updates the index 1:1
        assert_eq!(sparkv.len(), 2);

        let cleared_count = sparkv.clear_expired();
        assert_eq!(cleared_count, 0); // no longer expiring
        assert_eq!(sparkv.expiries.len(), 2); // nothing to clear
        assert_eq!(sparkv.len(), 2);
    }

    #[test]
    fn test_contains_key_is_expiry_aware() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = false;
        let mut sparkv = SparKV::with_config(config);
        _ = sparkv.set_with_ttl(
            "expiring",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        _ = sparkv.set_with_ttl(
            "live",
            String::from("value"),
            std::time::Duration::from_secs(60),
        );
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Expired-but-unswept: contains_key agrees with get (both report absent).
        assert!(!sparkv.contains_key("expiring"));
        assert!(sparkv.get("expiring").is_none());
        // Yet it is still physically present (not swept).
        assert_eq!(sparkv.len(), 2);

        // Live key is reported present.
        assert!(sparkv.contains_key("live"));
        assert_eq!(sparkv.get("live"), Some(String::from("value")));

        assert!(!sparkv.contains_key("non-existent"));
    }

    #[test]
    fn test_clear_expired_does_not_panic_on_deleted_key() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = false;
        let mut sparkv = SparKV::with_config(config);
        _ = sparkv.set_with_ttl(
            "gone",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        // Delete removes the index entry too, so nothing is left to expire.
        assert_eq!(sparkv.delete("gone"), Some(String::from("value")));
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Clearing after a delete is a safe no-op.
        let cleared_count = sparkv.clear_expired();
        assert_eq!(cleared_count, 0);
        assert_eq!(sparkv.expiries.len(), 0); // already removed on delete
        assert!(!sparkv.contains_key("gone"));
    }

    #[test]
    fn test_clear_expired_does_not_recurse_under_default_config() {
        // Default config has auto_clear_expired = true.
        let mut sparkv = SparKV::new();
        _ = sparkv.set_with_ttl(
            "expiring",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        std::thread::sleep(std::time::Duration::from_millis(2));

        // A subsequent set triggers auto-clear; must not overflow the stack.
        _ = sparkv.set("live", String::from("value"));

        assert!(!sparkv.contains_key("expiring"));
        assert!(sparkv.get("expiring").is_none());
        assert_eq!(sparkv.get("live"), Some(String::from("value")));
        assert_eq!(sparkv.len(), 1);
    }

    #[test]
    fn test_clear_expired_with_auto_clear_expired_enabled() {
        let mut config: Config = Config::new();
        config.auto_clear_expired = true; // explicitly setting it to true
        let mut sparkv = SparKV::with_config(config);
        _ = sparkv.set_with_ttl(
            "no-longer",
            String::from("value"),
            std::time::Duration::from_millis(1),
        );
        _ = sparkv.set_with_ttl(
            "no-longer",
            String::from("v"),
            std::time::Duration::from_secs(90),
        );
        std::thread::sleep(std::time::Duration::from_millis(2));
        _ = sparkv.set_with_ttl(
            "not-expired",
            String::from("value"),
            std::time::Duration::from_secs(60),
        );
        assert_eq!(sparkv.expiries.len(), 2); // diff from above, because of auto clear
        assert_eq!(sparkv.len(), 2);

        // auto clear
        _ = sparkv.set_with_ttl(
            "new-",
            String::from("value"),
            std::time::Duration::from_secs(60),
        );
        assert_eq!(sparkv.expiries.len(), 3); // should have cleared the expiries
        assert_eq!(sparkv.len(), 3); // but not actually deleting
    }
}
