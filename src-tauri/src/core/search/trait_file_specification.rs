use crate::core::FileSystemEntry;
pub trait FileSpecification: Send + Sync {
    /// Проверяет, удовлетворяет ли файл условию
    /// # Аргументы
    /// * `item` - Информация о файле/директории
    /// # Возвращает
    /// `true` если файл подходит под критерий, иначе `false`
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool;
}