#[cfg(test)]
mod tests {
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
    fn test_fuzzy_exact_match() {
        let spec = NameSpecification::new(
            "test.txt".into(),
            TextMatchMode::Fuzzy(0)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("tast.txt")));
    }
    #[test]
    fn test_fuzzy_one_error_substitution() {
        let spec = NameSpecification::new(
            "test.txt".into(),
            TextMatchMode::Fuzzy(1)
        ).unwrap();
        // Замена одной буквы
        assert!(spec.is_satisfied_by(&create_test_file("test.txt")));  // 0 ошибок
        assert!(spec.is_satisfied_by(&create_test_file("tast.txt")));  // 1 ошибка: e→a
        assert!(spec.is_satisfied_by(&create_test_file("tost.txt")));  // 1 ошибка: e→o
    }
    #[test]
    fn test_fuzzy_one_error_deletion() {
        let spec = NameSpecification::new(
            "test".into(),
            TextMatchMode::Fuzzy(1)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test")));   // 0 ошибок
        assert!(spec.is_satisfied_by(&create_test_file("tes")));    // 1 удаление
        assert!(spec.is_satisfied_by(&create_test_file("tst")));    // 1 удаление
    }
    #[test]
    fn test_fuzzy_one_error_insertion() {
        let spec = NameSpecification::new(
            "test".into(),
            TextMatchMode::Fuzzy(1)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test")));    // 0 ошибок
        assert!(spec.is_satisfied_by(&create_test_file("test1")));   // 1 вставка
        assert!(spec.is_satisfied_by(&create_test_file("tests")));   // 1 вставка
    }
    #[test]
    fn test_fuzzy_two_errors() {
        let spec = NameSpecification::new(
            "test".into(),
            TextMatchMode::Fuzzy(2)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test")));    // 0
        assert!(spec.is_satisfied_by(&create_test_file("tast")));    // 1: e→a
        assert!(spec.is_satisfied_by(&create_test_file("toast")));   // 2: e→o, s→a
        assert!(spec.is_satisfied_by(&create_test_file("best")));    // 1: t→b
    }
    #[test]
    fn test_fuzzy_exceed_threshold() {
        let spec = NameSpecification::new(
            "test".into(),
            TextMatchMode::Fuzzy(1)
        ).unwrap();
        // Больше 1 ошибки
        assert!(!spec.is_satisfied_by(&create_test_file("toast")));  // 2 ошибки
        assert!(!spec.is_satisfied_by(&create_test_file("xyz")));    // много ошибок
    }
    #[test]
    fn test_fuzzy_case_insensitive() {
        let spec = NameSpecification::new(
            "test".into(),
            TextMatchMode::Fuzzy(0)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test")));
        assert!(spec.is_satisfied_by(&create_test_file("TEST")));
        assert!(spec.is_satisfied_by(&create_test_file("TeSt")));
    }
    #[test]
    fn test_fuzzy_typo_document() {
        let spec = NameSpecification::new(
            "document.txt".into(),
            TextMatchMode::Fuzzy(2)
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("document.txt")));   // 0
        assert!(spec.is_satisfied_by(&create_test_file("documnet.txt")));   // 2: перестановка
        assert!(spec.is_satisfied_by(&create_test_file("dcument.txt")));    // 1: удаление o
        assert!(!spec.is_satisfied_by(&create_test_file("doc.txt")));       // слишком много
    }
    #[test]
    fn test_fuzzy_realistic_search() {
        // Реалистичный сценарий: ищем "report_2024.pdf"
        let spec = NameSpecification::new(
            "report_2024.pdf".into(),
            TextMatchMode::Fuzzy(3)
        ).unwrap();
        // Найдет с опечатками
        assert!(spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("reprot_2024.pdf")));  // перестановка
        assert!(spec.is_satisfied_by(&create_test_file("report_2023.pdf")));  // 1 цифра
        // Не найдет если слишком разные
        assert!(!spec.is_satisfied_by(&create_test_file("document_2024.pdf")));
    }
}