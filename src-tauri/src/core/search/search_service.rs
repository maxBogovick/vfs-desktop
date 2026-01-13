use crate::core::{FileSystem, FileSystemEntry};
use crate::core::search::sq_n_sqb::SearchQuery;

/// Сервис для выполнения поиска файлов
///
/// # Пример
/// ```
/// use vfdir_lib::core::search::enums::{SearchQueryBuilder, TextMatchMode};
/// use vfdir_lib::core::search::specification::SearchService;
/// let fs = get_filesystem();
/// let service = SearchService::new(fs);
///
/// let query = SearchQueryBuilder::new()
///     .with_name("report", TextMatchMode::Contains)
///     .recursive(true)
///     .build();
///
/// let results = service.search("/documents", query)?;
/// ```
pub struct SearchService<FS: FileSystem> {
    file_system: FS,
}

impl<FS: FileSystem> SearchService<FS> {
    pub fn new(file_system: FS) -> Self {
        Self { file_system }
    }

    /// Выполняет поиск файлов согласно запросу
    ///
    /// # Аргументы
    /// * `root_path` - Путь к папке, где начинать поиск
    /// * `query` - Поисковый запрос с фильтрами
    ///
    /// # Возвращает
    /// Вектор файлов, которые соответствуют всем критериям
    pub fn search(&self, root_path: &str, query: SearchQuery) -> Result<Vec<FileSystemEntry>, String> {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Создайте пустой вектор для результатов
        // 2. Вызовите вспомогательный метод для обхода
        // 3. Верните результаты
        //
        let mut new_v = Vec::new();
        self.search_recursive(root_path, &query, &mut new_v)?;
        Ok(new_v)
        // Подсказка:
        // let mut results = Vec::new();
        // self.search_recursive(root_path, &query, &mut results)?;
        // Ok(results)
    }

    /// Рекурсивный обход директорий
    fn search_recursive(
        &self,
        path: &str,
        query: &SearchQuery,
        results: &mut Vec<FileSystemEntry>,
    ) -> Result<(), String> {
        // 🎯 ВАША ЗАДАЧА:
        //
        // 1. Получите содержимое директории: self.file_system.read_directory(path)?
        // 2. Для каждого entry:
        //    а) Если это файл И проходит фильтр → добавить в results
        //    б) Если это директория И query.recursive → рекурсивный вызов
        let entries = self.file_system.read_directory(path).unwrap();
        for entry in entries {
            if entry.is_file {
                if query.root_spec.is_satisfied_by(&entry) {
                    results.push(entry);
                } else if entry.is_dir && query.recursive{
                    self.search_recursive(&entry.path, query, results)?;
                }
            }
        }
        Ok(())
        //
        // Псевдокод:
        // let entries = self.file_system.read_directory(path)?;
        //
        // for entry in entries {
        //     if entry.is_file {
        //         if query.root_spec.is_satisfied_by(&entry) {
        //             results.push(entry);
        //         }
        //     } else if entry.is_dir && query.recursive {
        //         self.search_recursive(&entry.path, query, results)?;
        //     }
        // }
        //
        // Ok(())
    }
}