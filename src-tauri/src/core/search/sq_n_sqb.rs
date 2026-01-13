use crate::core::search::combinators::and::AndSpecification;
use crate::core::search::filters::name::{NameSpecification, TextMatchMode};
use crate::core::search::filters::extension::ExtensionSpecification;
use crate::core::search::filters::size::SizeSpec;
use crate::core::search::trait_file_specification::FileSpecification;

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
/// use vfdir_lib::core::search::enums::{SearchQueryBuilder, TextMatchMode};
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