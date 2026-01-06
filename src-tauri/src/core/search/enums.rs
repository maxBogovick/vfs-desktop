/// Определяет режим сравнения строк при поиске
#[derive(Debug, Clone, PartialEq)]
pub enum TextMatchMode {
    Exact,
    Contains,
    Regex,
    Fuzzy(usize),
}
use regex::Regex;
use crate::core::FileSystemEntry;
use crate::core::search::specification::FileSpecification;

/// Спецификация для фильтрации по имени файла
///
/// Поддерживает 4 режима поиска:
/// - Exact: точное совпадение
/// - Contains: содержит подстроку
/// - Regex: регулярное выражение
/// - Fuzzy: нечеткий поиск
pub struct NameSpecification {
    pattern: String,
    mode: TextMatchMode,
    // Храним скомпилированный regex для оптимизации!
    // Компилируем 1 раз при создании, используем N раз при поиске
    compiled_regex: Option<Regex>,
}

impl NameSpecification {
    /// Создает новую спецификацию поиска по имени
    ///
    /// # Ошибки
    /// Возвращает ошибку если regex невалидный
    ///
    /// # Примеры
    /// ```
    /// let spec = NameSpecification::new(
    ///     "test".into(),
    ///     TextMatchMode::Contains
    /// )?;
    /// ```
    pub fn new(pattern: String, mode: TextMatchMode) -> Result<Self, String> {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Если mode == TextMatchMode::Regex:
        //    - Используйте Regex::new(&pattern)
        //    - Обработайте ошибку компиляции regex
        //    - Сохраните результат в Some(regex)
        //

        // 2. Для остальных режимов: compiled_regex = None
        //
        // Подсказка по обработке ошибок:
        // match Regex::new(&pattern) {
        //     Ok(regex) => Some(regex),
        //     Err(e) => return Err(format!("Invalid regex: {}", e)),
        // }

        let compiled_regex = match mode {
            TextMatchMode::Regex => {
                match Regex::new(&pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => return Err(format!("invalid regex {}", e)),
                }
            }
            _ => None,
        };


        Ok(Self {
            pattern,
            mode,
            compiled_regex,
        })
    }
}

impl FileSpecification for NameSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool  {
        match self.mode {
            TextMatchMode::Regex => {
                // 🎯 ВАША ЗАДАЧА:
                // 1. Получите compiled_regex из self.compiled_regex
                //    (используйте if let Some(regex) = ...)
                // 2. Вызовите regex.is_match(&item.name)
                // 3. Если regex = None, верните false (или panic! для безопасности)
                if let Some(regex) = &self.compiled_regex { regex.is_match(&item.name) } else { panic!("Regex not compiled!"); }
            }
            TextMatchMode::Exact => {
                item.name.to_lowercase().eq(&self.pattern)
            }
            _ => false
        }
    }
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

