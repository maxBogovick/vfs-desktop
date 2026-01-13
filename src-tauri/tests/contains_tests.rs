#[cfg(test)]
mod test{
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::filters::name::{NameSpecification, TextMatchMode};
    use vfdir_lib::core::search::trait_file_specification::FileSpecification;
    fn create_test_file(name: &str) -> FileSystemEntry {
        FileSystemEntry {
            path: format!("/test/{}", name),
            name: name.to_string(),
            is_dir: false,
            is_file: true,
            size: Some(1024),
            modified: Some(1234567890),
            created: Some(1234567890),
            accessed: Some(1234567890),
        }
    }
    #[test]
    fn test_contains_search() {
        let spec = NameSpecification::new(
            "t".to_string(),
            TextMatchMode::Contains
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test.txt")))
    }
    #[test]
    fn wrong_contains_search() {
        let spec = NameSpecification::new(
            "ts".to_string(),
            TextMatchMode::Contains
        ).unwrap();
        assert!(!spec.is_satisfied_by(&create_test_file("test.txt")))
    }
}