use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;

// Спецификация: расширение файла
/// Спецификация для фильтрации по расширению файла
///
/// # Примеры
/// ```
/// // Найти все PDF
/// use vfdir_lib::core::search::specification::ExtensionSpecification;
/// let spec = ExtensionSpecification::new("pdf");
/// let spec = ExtensionSpecification::new(".pdf"); // то же самое
///
/// // Найти все изображения (нужно несколько спецификаций через OR)
/// let pdf_spec = ExtensionSpecification::new("jpg");
/// let png_spec = ExtensionSpecification::new("png");
/// ```
pub struct ExtensionSpecification {
    pub extension: String,
}

impl ExtensionSpecification {
    /// Создает новую спецификацию по расширению
    ///
    /// Автоматически добавляет точку если её нет
    pub fn new(extension: String) -> Self {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Если extension начинается с точки, оставить как есть
        // 2. Если нет точки, добавить её в начало
        // 3. Привести к lowercase для case-insensitive поиска
        //
        // Подсказка:
        // let ext = if extension.starts_with('.') {
        //     extension.to_lowercase()
        // } else {
        //     format!(".{}", extension.to_lowercase())
        // };
        let ext = if extension.starts_with('.') { extension.to_lowercase() } else { format!(".{}", extension.to_lowercase()) };
        Self {extension: ext}
    }
}

impl FileSpecification for ExtensionSpecification {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Приведите имя файла к lowercase
        // 2. Проверьте, заканчивается ли имя на self.extension
        item.name.to_lowercase().ends_with(&self.extension)
    }
}