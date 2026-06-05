pub trait ValueSize {
    /// Logical content size in bytes, compared against `Config::max_item_size`.
    fn value_size(&self) -> usize;
}

impl ValueSize for String {
    fn value_size(&self) -> usize {
        self.len()
    }
}

impl ValueSize for str {
    fn value_size(&self) -> usize {
        self.len()
    }
}

impl ValueSize for &str {
    fn value_size(&self) -> usize {
        self.len()
    }
}

impl ValueSize for Vec<u8> {
    fn value_size(&self) -> usize {
        self.len()
    }
}

impl ValueSize for [u8] {
    fn value_size(&self) -> usize {
        self.len()
    }
}

impl ValueSize for &[u8] {
    fn value_size(&self) -> usize {
        self.len()
    }
}

macro_rules! impl_value_size_sized {
    ($($t:ty),*) => {
        $(impl ValueSize for $t {
            fn value_size(&self) -> usize {
                std::mem::size_of::<$t>()
            }
        })*
    };
}

impl_value_size_sized!(
    bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_like_size() {
        assert_eq!(String::from("abc").value_size(), 3);
        assert_eq!("hello".value_size(), 5);
        assert_eq!(vec![1u8, 2, 3, 4].value_size(), 4);
    }

    #[test]
    fn test_scalar_size() {
        assert_eq!(42u32.value_size(), 4);
        assert_eq!(42u64.value_size(), 8);
        assert_eq!(true.value_size(), 1);
    }
}
