#[cfg(test)]
mod test{
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::filters::size::SizeSpec;
    use vfdir_lib::core::search::trait_file_specification::FileSpecification;
    fn create_file_with_size(name: &str, size: Option<u64>) -> FileSystemEntry {
        FileSystemEntry {
            path: format!("/test/{}", name),
            name: name.to_string(),
            is_dir: false,
            is_file: true,
            size,
            modified: Some(1234567890),
            created: Some(1234567890),
            accessed: Some(1234567890),
        }
    }
    #[test]
    fn test_size_no_limits() {
        let spec = SizeSpec::new(None, None);
        assert!(spec.is_satisfied_by(&create_file_with_size("small.txt", Some(100))));
        assert!(spec.is_satisfied_by(&create_file_with_size("large.txt", Some(1_000_000))));
    }
    #[test]
    fn test_size_min_only() {
        let spec = SizeSpec::new(Some(1000), None);
        assert!(spec.is_satisfied_by(&create_file_with_size("big.txt", Some(5000))));
        assert!(spec.is_satisfied_by(&create_file_with_size("exact.txt", Some(1000))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("small.txt", Some(500))));
    }
    #[test]
    fn test_size_max_only() {
        let spec = SizeSpec::new(None, Some(1000));
        assert!(spec.is_satisfied_by(&create_file_with_size("small.txt", Some(500))));
        assert!(spec.is_satisfied_by(&create_file_with_size("exact.txt", Some(1000))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("big.txt", Some(5000))));
    }
    #[test]
    fn test_size_range() {
        let spec = SizeSpec::new(Some(1000), Some(5000));
        assert!(spec.is_satisfied_by(&create_file_with_size("good1.txt", Some(1000))));
        assert!(spec.is_satisfied_by(&create_file_with_size("good2.txt", Some(3000))));
        assert!(spec.is_satisfied_by(&create_file_with_size("good3.txt", Some(5000))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("too_small.txt", Some(999))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("too_big.txt", Some(5001))));
    }
    #[test]
    fn test_size_no_size_info() {
        let spec = SizeSpec::new(Some(1000), None);
        // Файл без информации о размере не должен проходить фильтр
        assert!(!spec.is_satisfied_by(&create_file_with_size("unknown.txt", None)));
    }
    #[test]
    fn test_size_realistic_small_files() {
        // Маленькие файлы: до 100KB
        let spec = SizeSpec::new(None, Some(100 * 1024));
        assert!(spec.is_satisfied_by(&create_file_with_size("config.json", Some(1024))));
        assert!(spec.is_satisfied_by(&create_file_with_size("readme.txt", Some(50 * 1024))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("video.mp4", Some(10 * 1024 * 1024))));
    }
    #[test]
    fn test_size_realistic_documents() {
        // Документы: от 10KB до 10MB
        let spec = SizeSpec::new(
            Some(10 * 1024),
            Some(10 * 1024 * 1024)
        );
        assert!(!spec.is_satisfied_by(&create_file_with_size("tiny.txt", Some(1024))));
        assert!(spec.is_satisfied_by(&create_file_with_size("document.pdf", Some(500 * 1024))));
        assert!(spec.is_satisfied_by(&create_file_with_size("presentation.pptx", Some(5 * 1024 * 1024))));
        assert!(!spec.is_satisfied_by(&create_file_with_size("movie.mkv", Some(1024 * 1024 * 1024))));
    }
}