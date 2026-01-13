/// Комбинирует несколько спецификаций через логическое И (AND)
///
/// # Пример
/// ```
/// use vfdir_lib::core::search::enums::{NameSpecification, TextMatchMode};
/// use vfdir_lib::core::search::specification::{AndSpecification, FileSpecification, SizeSpec};
/// let specs: Vec<Box<dyn FileSpecification>> = vec![
///     Box::new(NameSpecification::new("test".into(), TextMatchMode::Contains).unwrap()),
///     Box::new(SizeSpec::new(Some(1024), None)),
/// ];
/// let and_spec = AndSpecification::new(specs);
/// // Теперь файл должен содержать "test" И быть >= 1024 байт
/// ```
use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;

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
    }
}