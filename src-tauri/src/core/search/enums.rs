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
use crate::core::search::specification::{AndSpecification, ExtensionSpecification, FileSpecification, SizeSpec};
use strsim::levenshtein;
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
    /// use crate::vfdir_lib::core::search::enums::*;
    /// let spec = NameSpecification::new(
    ///     "test".into(),
    ///     TextMatchMode::Contains
    /// );
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
            TextMatchMode::Fuzzy(max_distance) => {
                // 🎯 ВАША ЗАДАЧА:
                //
                // 1. Приведите обе строки к lowercase для case-insensitive поиска
                // 2. Вычислите расстояние: levenshtein(&pattern, &filename)
                // 3. Верните true, если distance <= max_distance
                //
                // Подсказка:
                // let pattern_lower = self.pattern.to_lowercase();
                // let name_lower = item.name.to_lowercase();
                // let distance = levenshtein(&pattern_lower, &name_lower);
                // distance <= *max_distance
                let pattern_lc = self.pattern.to_lowercase();
                let name_lc = item.name.to_lowercase();
                let d = levenshtein(&pattern_lc, &name_lc);
                //if d.eq(&max_distance) { true } else { false }
                d <= max_distance
            }
            TextMatchMode::Contains => {
                item.name.to_lowercase().contains(&self.pattern.to_lowercase())
            }
        }
    }
}

/// Поисковый запрос, содержащий все условия фильтрации
pub struct SearchQuery {
    /// Корневая спецификация (обычно AndSpecification)
    pub root_spec: Box<dyn FileSpecification>,
    /// Искать рекурсивно во всех подпапках?
    pub recursive: bool,
}

/// Builder для удобного создания SearchQuery
///
/// # Пример использования
/// ```
/// let query = SearchQueryBuilder::new()
///     .with_name("report", TextMatchMode::Fuzzy(2))
///     .with_size_range(Some(1024), Some(10_485_760))
///     .with_extension("pdf")
///     .recursive(true)
///     .build();
/// ```
pub struct SearchQueryBuilder {
    specs: Vec<Box<dyn FileSpecification>>,
    recursive: bool,
}

impl SearchQueryBuilder {
    /// Создает новый пустой builder
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            recursive: false,
        }
    }

    /// Добавляет фильтр по имени
    ///
    /// # Примеры
    /// ```
    /// builder.with_name("test", TextMatchMode::Contains)
    /// builder.with_name(r"report_\d+", TextMatchMode::Regex)
    /// builder.with_name("document", TextMatchMode::Fuzzy(2))
    /// ```
    pub fn with_name(mut self, pattern: &str, mode: TextMatchMode) -> Self {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Создайте NameSpecification::new(pattern.to_string(), mode)
        // 2. Обработайте Result (может быть ошибка если regex невалидный)
        //    - Если Ok: добавьте в self.specs
        //    - Если Err: можно panic! или просто игнорировать
        // 3. Верните self для цепочки вызовов
        match NameSpecification::new(pattern.to_string(), mode) {
            Ok(s) => self.specs.push(Box::new(s)),
            Err(e) => panic!("Warning: Invalid pattern: {}", e)
        }
        self
    }

    /// Добавляет фильтр по размеру
    ///
    /// # Аргументы
    /// * `min_bytes` - Минимальный размер (включительно)
    /// * `max_bytes` - Максимальный размер (включительно)
    ///
    /// # Примеры
    /// ```
    /// builder.with_size_range(Some(1024), Some(1_048_576))  // 1KB - 1MB
    /// builder.with_size_range(Some(10_000), None)           // >= 10KB
    /// builder.with_size_range(None, Some(100_000))          // <= 100KB
    /// ```
    pub fn with_size_range(mut self, min_bytes: Option<u64>, max_bytes: Option<u64>) -> Self {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Создайте SizeSpecification::new(min_bytes, max_bytes)
        // 2. Добавьте в self.specs
        // 3. Верните self
        self.specs.push(Box::new(SizeSpec::new(min_bytes, max_bytes)));
        self
    }

    /// Добавляет фильтр по расширению
    ///
    /// # Примеры
    /// ```
    /// builder.with_extension("pdf")
    /// builder.with_extension(".jpg")
    /// ```
    pub fn with_extension(mut self, extension: &str) -> Self {
        // 🎯 ВАША ЗАДАЧА:
        // Аналогично предыдущим методам
        self.specs.push(Box::new(ExtensionSpecification::new(extension.to_string())));
        self
    }

    /// Устанавливает режим рекурсивного поиска
    ///
    /// # Аргументы
    /// * `enable` - true для рекурсивного поиска в подпапках
    pub fn recursive(mut self, enable: bool) -> Self {
        // 🎯 ВАША ЗАДАЧА:
        //
        // Просто установите self.recursive = enable
        // и верните self
        self.recursive = enable;
        self
    }

    /// Создает финальный SearchQuery
    ///
    /// Объединяет все спецификации через AndSpecification
    pub fn build(self) -> SearchQuery {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Создайте AndSpecification из self.specs
        // 2. Оберните в Box<dyn FileSpecification>
        // 3. Создайте и верните SearchQuery
        let r = Box::new(AndSpecification::new(self.specs));
        SearchQuery {
            root_spec: r,
            recursive: self.recursive
        }
    }
}

impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(contains, TextMatchMode::Contains);

    }



}

