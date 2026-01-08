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
/// use vfdir_lib::core::search::enums::{NameSpecification, TextMatchMode};
/// use vfdir_lib::core::search::specification::{AndSpecification, FileSpecification};
/// let specs: Vec<Box<dyn FileSpecification>> = vec![
///     Box::new(NameSpecification::new("test".into(), TextMatchMode::Contains).unwrap()),
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
pub struct SizeSpec {
    min_bytes: Option<u64>,
    max_bytes: Option<u64>,
}
impl SizeSpec {
    pub fn new(min_bytes: Option<u64>, max_bytes: Option<u64>) -> Self {
        Self { min_bytes, max_bytes }
    }
}
impl FileSpecification for SizeSpec {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        // 🎯 ВАША ЗАДАЧА:
        // 1. Получите размер из item.size (это Option<u64>)
        //    Если size = None, что вернуть? (подсказка: false, т.к. размер неизвестен)
        // 2. Проверьте минимальную границу:
        //    if let Some(min) = self.min_bytes {
        //        if size < min { return false; }
        //    }
        // 3. Проверьте максимальную границу:
        //    if let Some(max) = self.max_bytes {
        //        if size > max { return false; }
        //    }
        // 4. Если оба условия прошли, верните true
        // Альтернативный подход (короче):
        // let size = item.size?;  // вернет false если None
        // self.min_bytes.map_or(true, |min| size >= min) &&
        // self.max_bytes.map_or(true, |max| size <= max)
        match item.size {
            None => false,
            Some(m) => {
                if self.min_bytes.is_none() && self.max_bytes.is_none() {
                    true
                } else if let Some(min) = self.min_bytes {
                    if let Some(max) = self.max_bytes {
                        m >= min && m <= max
                    } else {
                        m >= min
                    }
                } else if let Some(max) = self.max_bytes {
                    m <= max
                } else {
                    false
                }
            }
        }
        // Option<u64> может быть:
        /*match item.size {
            Some(size) => println!("Размер: {} байт", size),
            None => println!("Размер неизвестен (директория или ошибка)"),
        }*/
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::search::enums::NameSpecification;
    use crate::core::search::enums::TextMatchMode;
    use crate::core::search::specification::SizeSpec;
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

        let specs: Vec<Box<dyn FileSpecification>> = vec![
            Box::new(NameContainsSpec("report".into())),
            Box::new(ExtensionSpec(".pdf".into())),
            Box::new(SizeSpec { min_bytes: Some(0), max_bytes: Some(2048) }),
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