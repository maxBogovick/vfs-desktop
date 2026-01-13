#[cfg(test)]
mod tests {
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::trait_file_specification::FileSpecification;
    use vfdir_lib::core::search::combinators::and::AndSpecification;
    // Тестовая спецификация, которая всегда возвращает true
    struct AlwaysTrueSpec;

    impl FileSpecification for AlwaysTrueSpec {
        fn is_satisfied_by(&self, _item: &FileSystemEntry) -> bool {
            true
        }
    }

    // Вспомогательная функция для создания тестового файла
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
//1
    #[test]
    fn test_specification_trait_works() {
        let spec = AlwaysTrueSpec;
        let file = create_test_file("test.txt");

        assert!(spec.is_satisfied_by(&file));
    }

    #[test]
    fn test_specification_can_be_boxed() {
        let spec: Box<dyn FileSpecification> = Box::new(AlwaysTrueSpec);
        let file = create_test_file("test.txt");

        assert!(spec.is_satisfied_by(&file));
    }
    // Спецификация: имя содержит строку

    #[test]
    fn test_and_empty_returns_true() {
        // Пустая AND-спецификация должна пропускать все
        let spec = AndSpecification::new(vec![]);
        let file = create_test_file("anything.txt");
        assert!(spec.is_satisfied_by(&file));
    }
}