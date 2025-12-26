/// Определяет режим сравнения строк при поиске
#[derive(Debug, Clone, PartialEq)]
pub enum TextMatchMode {
    /// Точное совпадение (case-sensitive)
    /// Пример: "test.txt" найдет только "test.txt"
    Exact,

    /// Содержит подстроку (case-insensitive)
    /// Пример: "test" найдет "my_test_file.txt", "test.doc", "TEST.pdf"
    Contains,

    /// Регулярное выражение
    /// Пример: r"test_\d+\.txt" найдет "test_123.txt", "test_99.txt"
    Regex,

    /// Нечеткий поиск с максимальной дистанцией
    /// Пример: Fuzzy(2) найдет "documnet" если искали "document" (2 ошибки)
    Fuzzy(usize),
}

impl TextMatchMode {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_match_mode_creation() {
        let exact = TextMatchMode::Exact;
        let contains = TextMatchMode::Contains;
        let regex = TextMatchMode::Regex;
        let fuzzy = TextMatchMode::Fuzzy(2);

        assert_eq!(exact, TextMatchMode::Exact);
        assert_eq!(fuzzy, TextMatchMode::Fuzzy(2));
    }
}

