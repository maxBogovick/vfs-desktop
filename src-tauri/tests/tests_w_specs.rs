mod tests {
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::combinators::and::AndSpecification;
    use vfdir_lib::core::search::filters::extension::ExtensionSpecification;
    use vfdir_lib::core::search::filters::name::NameContainsSpec;
    use vfdir_lib::core::search::filters::size::SizeSpec;
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
    fn test_and_single_condition_true() {
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("test".into())),
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_single_condition_false() {
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("xyz".into())),
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(!spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_multiple_all_true() {
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("test".into())),
            Box::new(ExtensionSpecification::new(".txt".to_string())),
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_multiple_one_false() {
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("test".into())),
            Box::new(ExtensionSpecification::new(".pdf".into())), // ← файл .txt!
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(!spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_three_conditions() {

        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("report".into())),
            Box::new(ExtensionSpecification::new(".pdf".into())),
            Box::new(SizeSpec { min_bytes: Some(0), max_bytes: Some(2048) }),
        ];
        let spec = AndSpecification::new(specs);
        let mut file = create_test_file("monthly_report.pdf");
        file.size = Some(2048);
        assert!(spec.is_satisfied_by(&file));
    }
}