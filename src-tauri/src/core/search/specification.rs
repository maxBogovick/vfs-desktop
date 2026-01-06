use crate::core::FileSystemEntry;
pub trait FileSpecification: Send + Sync {
    /// Проверяет, удовлетворяет ли файл условию
    /// # Аргументы
    /// * `item` - Информация о файле/директории
    /// # Возвращает
    /// `true` если файл подходит под критерий, иначе `false`
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool;
}

/// Комбинирует несколько спецификаций через логическое И (AND)
///
/// # Пример
/// ```
/// let specs: Vec<Box<dyn FileSpecification>> = vec![
///     Box::new(NameSpec::new("test".into(), TextMatchMode::Contains)),
///     Box::new(SizeSpec::new(Some(1024), None)),
/// ];
/// let and_spec = AndSpecification::new(specs);
/// // Теперь файл должен содержать "test" И быть >= 1024 байт
/// ```
pub struct AndSpecification {
    specs: Vec<Box<dyn FileSpecification>>,
}

impl AndSpecification {
    /// Создает новую AND-спецификацию
    pub fn new(specs: Vec<Box<dyn FileSpecification>>) -> Self {
        Self { specs }
    }
}

impl FileSpecification for AndSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        // 🎯 ВАША ЗАДАЧА:
        // Используйте метод .iter().all() для проверки ВСЕХ спецификаций
        //
        let i = self.specs.iter().all(|s| s.is_satisfied_by(item));
        i
        // Подсказка:
        // self.specs.iter().all(|spec| spec.is_satisfied_by(item))
        //
        // Вопрос: Что вернет all() если specs пустой? (подсказка: true!)
    }
}



#[cfg(test)]
mod tests {
    use crate::core::search::enums::NameSpecification;
use super::*;
    use crate::core::search::enums::TextMatchMode;

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
    struct NameContainsSpec(String);
    impl FileSpecification for NameContainsSpec {
        fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
            item.name.to_lowercase().contains(&self.0.to_lowercase())
        }
    }
    // Спецификация: расширение файла
    struct ExtensionSpec(String);
    impl FileSpecification for ExtensionSpec {
        fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
            item.name.to_lowercase().ends_with(&self.0.to_lowercase())
        }
    }
    #[test]
    fn test_and_empty_returns_true() {
        // Пустая AND-спецификация должна пропускать все
        let spec = AndSpecification::new(vec![]);
        let file = create_test_file("anything.txt");
        assert!(spec.is_satisfied_by(&file));
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
            Box::new(ExtensionSpec(".txt".into())),
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_multiple_one_false() {
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("test".into())),
            Box::new(ExtensionSpec(".pdf".into())), // ← файл .txt!
        ];
        let spec = AndSpecification::new(specs);
        let file = create_test_file("test.txt");
        assert!(!spec.is_satisfied_by(&file));
    }
    #[test]
    fn test_and_three_conditions() {
        struct SizeSpec(u64);
        impl FileSpecification for SizeSpec {
            fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
                item.size.map_or(false, |s| s >= self.0)
            }
        }
        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("report".into())),
            Box::new(ExtensionSpec(".pdf".into())),
            Box::new(SizeSpec(1024)),
        ];
        let spec = AndSpecification::new(specs);
        let mut file = create_test_file("monthly_report.pdf");
        file.size = Some(2048);
        assert!(spec.is_satisfied_by(&file));
    }

    #[test]
    fn test_regex_digits() {
        let spec = NameSpecification::new(
            r"test_\d+\.txt".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test_123.txt")));
        assert!(spec.is_satisfied_by(&create_test_file("test_1.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("test_abc.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("test_.txt")));
    }
    #[test]
    fn test_regex_start_anchor() {
        let spec = NameSpecification::new(
            r"^report".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("report.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("my_report.pdf")));
    }
    #[test]
    fn test_regex_end_anchor() {
        let spec = NameSpecification::new(
            r"\.pdf$".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("document.pdf")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.pdf.bak")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.txt")));
    }
    #[test]
    fn test_regex_case_sensitive_default() {
        let spec = NameSpecification::new(
            r"Report".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("Report_2024.pdf")));
        assert!(!spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
    }
    #[test]
    fn test_regex_case_insensitive_flag() {
        let spec = NameSpecification::new(
            r"(?i)report".into(),  // (?i) = case insensitive
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("Report_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("REPORT_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
    }
    #[test]
    fn test_regex_date_pattern() {
        let spec = NameSpecification::new(
            r"\d{4}-\d{2}-\d{2}".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("backup_2024-01-15.zip")));
        assert!(spec.is_satisfied_by(&create_test_file("2024-12-31_log.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("backup_24-1-15.zip")));
    }
    #[test]
    fn test_regex_version() {
        let spec = NameSpecification::new(
            r"v\d+\.\d+\.\d+".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("app_v1.2.3.exe")));
        assert!(spec.is_satisfied_by(&create_test_file("v10.0.1_release.zip")));
        assert!(!spec.is_satisfied_by(&create_test_file("version_1.2.exe")));
    }
    #[test]
    fn test_regex_extension_alternatives() {
        let spec = NameSpecification::new(
            r"\.(jpg|png|gif)$".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("photo.jpg")));
        assert!(spec.is_satisfied_by(&create_test_file("icon.png")));
        assert!(spec.is_satisfied_by(&create_test_file("animation.gif")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.pdf")));
    }
}