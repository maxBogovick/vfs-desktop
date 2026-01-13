#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use vfdir_lib::core::{FileSystem, FileSystemEntry, FileSystemResult};
    use vfdir_lib::core::search::filters::name::TextMatchMode;
    use vfdir_lib::core::search::search_service::SearchService;
    use vfdir_lib::core::search::sq_n_sqb::SearchQueryBuilder;

    // Мок файловой системы для тестирования
    struct MockFileSystem {
        structure: HashMap<String, Vec<FileSystemEntry>>,
    }
    impl MockFileSystem {
        fn new() -> Self {
            let mut structure = HashMap::new();
            // Создаем тестовую структуру:
            // /test/
            //   ├── file1.txt (1000 bytes)
            //   ├── file2.pdf (5000 bytes)
            //   ├── image.jpg (2000 bytes)
            //   └── subfolder/
            //       ├── file3.txt (3000 bytes)
            //       └── document.pdf (10000 bytes)
            structure.insert(
                "/test".to_string(),
                vec![
                    FileSystemEntry {
                        path: "/test/file1.txt".into(),
                        name: "file1.txt".into(),
                        is_file: true,
                        is_dir: false,
                        size: Some(1000),
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                    FileSystemEntry {
                        path: "/test/file2.pdf".into(),
                        name: "file2.pdf".into(),
                        is_file: true,
                        is_dir: false,
                        size: Some(5000),
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                    FileSystemEntry {
                        path: "/test/image.jpg".into(),
                        name: "image.jpg".into(),
                        is_file: true,
                        is_dir: false,
                        size: Some(2000),
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                    FileSystemEntry {
                        path: "/test/subfolder".into(),
                        name: "subfolder".into(),
                        is_file: false,
                        is_dir: true,
                        size: None,
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                ],
            );
            structure.insert(
                "/test/subfolder".to_string(),
                vec![
                    FileSystemEntry {
                        path: "/test/subfolder/file3.txt".into(),
                        name: "file3.txt".into(),
                        is_file: true,
                        is_dir: false,
                        size: Some(3000),
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                    FileSystemEntry {
                        path: "/test/subfolder/document.pdf".into(),
                        name: "document.pdf".into(),
                        is_file: true,
                        is_dir: false,
                        size: Some(10000),
                        modified: Some(123),
                        created: Some(123),
                        accessed: Some(123),
                    },
                ],
            );
            Self { structure }
        }
    }
    impl FileSystem for MockFileSystem {
        fn read_directory(&self, path: &str) -> FileSystemResult<Vec<FileSystemEntry>> {
            todo!()
        }

        fn get_file_info(&self, path: &str) -> FileSystemResult<FileSystemEntry> {
            todo!()
        }

        fn delete_item(&self, path: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn rename_item(&self, old_path: &str, new_name: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn create_folder(&self, path: &str, name: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn copy_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn move_items(&self, sources: &[String], destination: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn get_home_directory(&self) -> FileSystemResult<String> {
            todo!()
        }

        fn get_system_folders(&self) -> FileSystemResult<Vec<FileSystemEntry>> {
            todo!()
        }

        fn read_file_content(&self, path: &str, max_size: Option<u64>) -> FileSystemResult<String> {
            todo!()
        }

        fn open_file(&self, path: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn reveal_in_finder(&self, path: &str) -> FileSystemResult<()> {
            todo!()
        }

        fn normalize_path(&self, path: &str) -> FileSystemResult<String> {
            todo!()
        }

        fn get_path_suggestions(&self, partial_path: &str) -> FileSystemResult<Vec<String>> {
            todo!()
        }

        fn open_terminal(&self, path: &str) -> FileSystemResult<()> {
            todo!()
        }
    }
    #[test]
    fn test_search_no_filters_not_recursive() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new().build();
        let results = service.search("/test", query).unwrap();
        // Должны найти 3 файла в корневой папке (без subfolder)
        assert_eq!(results.len(), 3);
    }
    #[test]
    fn test_search_no_filters_recursive() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // Должны найти все 5 файлов
        assert_eq!(results.len(), 5);
    }
    #[test]
    fn test_search_by_extension_not_recursive() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_extension("pdf")
            .build();
        let results = service.search("/test", query).unwrap();
        // Только file2.pdf в корне
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "file2.pdf");
    }
    #[test]
    fn test_search_by_extension_recursive() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_extension("pdf")
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // file2.pdf + document.pdf
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|f| f.name == "file2.pdf"));
        assert!(results.iter().any(|f| f.name == "document.pdf"));
    }
    #[test]
    fn test_search_by_name_contains() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_name("file", TextMatchMode::Contains)
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // file1.txt, file2.pdf, file3.txt
        assert_eq!(results.len(), 3);
    }
    #[test]
    fn test_search_by_size_range() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_size_range(Some(2000), Some(5000))
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // image.jpg (2000), file3.txt (3000), file2.pdf (5000)
        assert_eq!(results.len(), 3);
    }
    #[test]
    fn test_search_combined_filters() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_name("file", TextMatchMode::Contains)
            .with_extension("txt")
            .with_size_range(Some(2000), None)
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // Только file3.txt (содержит "file", расширение .txt, размер >= 2000)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "file3.txt");
    }
    #[test]
    fn test_search_regex_pattern() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_name(r"file\d+\.txt", TextMatchMode::Regex)
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // file1.txt, file3.txt
        assert_eq!(results.len(), 2);
    }
    #[test]
    fn test_search_fuzzy_matching() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_name("documnet", TextMatchMode::Fuzzy(2))  // опечатка!
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        // Должен найти document.pdf (2 операции для исправления)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "document.pdf");
    }
    #[test]
    fn test_search_empty_results() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new()
            .with_name("nonexistent", TextMatchMode::Exact)
            .recursive(true)
            .build();
        let results = service.search("/test", query).unwrap();
        assert_eq!(results.len(), 0);
    }
    #[test]
    fn test_search_invalid_path() {
        let fs = MockFileSystem::new();
        let service = SearchService::new(fs);
        let query = SearchQueryBuilder::new().build();
        let result = service.search("/nonexistent", query);
        assert!(result.is_err());
    }
}