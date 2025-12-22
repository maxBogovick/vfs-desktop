use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Clone)]
pub struct Metadata {
    created_at: u64,
    modified_at: u64,
    size: usize
}
impl Metadata{
    pub fn new(created_at: u64, modified_at: u64, size: usize) -> Metadata{
        Self{created_at, modified_at, size}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    pub fn creating_metadata() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let new = Metadata::new(now, now, 10);

        assert_eq!(new.created_at, now);
        assert_eq!(new.modified_at, now);
        assert_eq!(new.size, 10);
    }

    #[test]
    pub fn having_the_size() {
        let new = Metadata::new(1, 2, 3);

        assert_eq!(new.size, 3);
    }

    #[test]
    pub fn cloned_metadata() {
        let m = Metadata::new(5, 6, 7);
        let c = m.clone();

        assert_eq!(c.created_at, m.created_at);
        assert_eq!(c.modified_at, m.modified_at);
        assert_eq!(c.size, m.size);
    }
}
