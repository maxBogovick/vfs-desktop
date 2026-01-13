#[cfg(test)]
mod tests {
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::filters::name::TextMatchMode;
    use vfdir_lib::core::search::sq_n_sqb::SearchQueryBuilder;

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
    fn test_builder_empty() {
        let query = SearchQueryBuilder::new().build();
        assert!(!query.recursive);
        // Пустой AndSpec должен пропускать все файлы
        let file = create_test_file("any.txt");
        assert!(query.root_spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_builder_with_name_only() {
        let query = SearchQueryBuilder::new()
            .with_name("test", TextMatchMode::Contains)
            .build();
        let file1 = create_test_file("test.txt");
        let file2 = create_test_file("other.txt");
        assert!(query.root_spec.is_satisfied_by(&file1));
        assert!(!query.root_spec.is_satisfied_by(&file2));
    }
    #[test]
    fn test_builder_with_size_only() {
        let query = SearchQueryBuilder::new()
            .with_size_range(Some(1000), Some(5000))
            .build();
        let file1 = create_file_with_size("small.txt", Some(500));
        let file2 = create_file_with_size("good.txt", Some(3000));
        let file3 = create_file_with_size("big.txt", Some(10000));
        assert!(!query.root_spec.is_satisfied_by(&file1));
        assert!(query.root_spec.is_satisfied_by(&file2));
        assert!(!query.root_spec.is_satisfied_by(&file3));
    }
    #[test]
    fn test_builder_with_extension_only() {
        let query = SearchQueryBuilder::new()
            .with_extension("pdf")
            .build();
        let file1 = create_test_file("document.pdf");
        let file2 = create_test_file("image.jpg");
        assert!(query.root_spec.is_satisfied_by(&file1));
        assert!(!query.root_spec.is_satisfied_by(&file2));
    }
    #[test]
    fn test_builder_combined_filters() {
        let query = SearchQueryBuilder::new()
            .with_name("report", TextMatchMode::Contains)
            .with_extension("pdf")
            .with_size_range(Some(10_000), None)
            .build();
        let mut file1 = create_test_file("monthly_report.pdf");
        file1.size = Some(50_000);
        let mut file2 = create_test_file("report.txt");  // неправильное расширение
        file2.size = Some(50_000);
        let mut file3 = create_test_file("monthly_report.pdf");  // слишком маленький
        file3.size = Some(5_000);
        let mut file4 = create_test_file("document.pdf");  // нет "report" в имени
        file4.size = Some(50_000);
        assert!(query.root_spec.is_satisfied_by(&file1));  // ✅ Все условия
        assert!(!query.root_spec.is_satisfied_by(&file2)); // ❌ Не pdf
        assert!(!query.root_spec.is_satisfied_by(&file3)); // ❌ Слишком маленький
        assert!(!query.root_spec.is_satisfied_by(&file4)); // ❌ Нет "report"
    }
    #[test]
    fn test_builder_recursive_flag() {
        let query1 = SearchQueryBuilder::new()
            .recursive(true)
            .build();
        assert!(query1.recursive);
        let query2 = SearchQueryBuilder::new()
            .recursive(false)
            .build();
        assert!(!query2.recursive);
        let query3 = SearchQueryBuilder::new().build();
        assert!(!query3.recursive);  // По умолчанию false
    }
    #[test]
    fn test_builder_realistic_scenario_1() {
        // Сценарий: найти все PDF отчеты за 2024 год, больше 1MB
        let query = SearchQueryBuilder::new()
            .with_name(r"report.*2024", TextMatchMode::Regex)
            .with_extension("pdf")
            .with_size_range(Some(1_048_576), None)
            .recursive(true)
            .build();
        let mut file1 = create_test_file("report_january_2024.pdf");
        file1.size = Some(2_000_000);
        let mut file2 = create_test_file("report_2023.pdf");
        file2.size = Some(2_000_000);
        assert!(query.root_spec.is_satisfied_by(&file1));
        assert!(!query.root_spec.is_satisfied_by(&file2)); // 2023, не 2024
    }
    #[test]
    fn test_builder_realistic_scenario_2() {
        // Сценарий: найти маленькие изображения (опечатки допустимы)
        let query = SearchQueryBuilder::new()
            .with_name("image", TextMatchMode::Fuzzy(2))
            .with_extension("jpg")
            .with_size_range(None, Some(500_000))
            .build();
        let mut file1 = create_test_file("imoge.jpg");  // опечатка!
        file1.size = Some(100_000);
        let mut file2 = create_test_file("image.jpg");
        file2.size = Some(2_000_000);  // слишком большой
        assert!(!query.root_spec.is_satisfied_by(&file1));
        assert!(!query.root_spec.is_satisfied_by(&file2));
    }
    #[test]
    fn test_builder_chaining() {
        // Проверяем, что методы можно вызывать в любом порядке
        let query1 = SearchQueryBuilder::new()
            .with_name("test", TextMatchMode::Contains)
            .with_extension("txt")
            .recursive(true)
            .build();
        let query2 = SearchQueryBuilder::new()
            .recursive(true)
            .with_extension("txt")
            .with_name("test", TextMatchMode::Contains)
            .build();
        let file = create_test_file("test.txt");
        assert!(query1.root_spec.is_satisfied_by(&file));
        assert!(query2.root_spec.is_satisfied_by(&file));
    }
}