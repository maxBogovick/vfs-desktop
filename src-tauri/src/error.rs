#[derive(Debug, PartialEq)]
pub enum FsError {
    NotFound(String),
    AlreadyExists(String),
    InvalidOperation(String)
}
pub type FsResult<T> = Result<T, FsError>;



pub fn bug(errors: Vec<String>) -> FsError{

    unimplemented!()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_already_exixst() {
        let error = FsError::AlreadyExists("it's already exists".to_string());
        assert_eq!(
            error,
            FsError::AlreadyExists("it's already exists".to_string())
        );
    }
    #[test]
    pub fn test_defound() {
        let error = FsError::NotFound("no any files found yet".to_string());
        assert_eq!(
            error,
            FsError::NotFound("no any files found yet".to_string())
        );
    }
}