#[derive(Debug, PartialEq)]
pub enum Error {
    CapacityExceeded,
    ItemSizeExceeded,
    TTLTooLong,
}
