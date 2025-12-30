# VFDir API Service Layer

Универсальный слой бизнес-логики, который может использоваться любыми клиентами:
- Tauri desktop приложение
- HTTP REST API сервер
- Консольные приложения
- Тесты
- Любые другие Rust приложения

## Архитектура

```
┌─────────────────────────────────────────────────────────────┐
│                      Клиенты / Транспорты                    │
├──────────────────┬──────────────────┬──────────────────────┤
│  Tauri Commands  │  HTTP REST API   │   CLI / Tests        │
└────────┬─────────┴────────┬─────────┴─────────┬─────────────┘
         │                  │                   │
         └──────────────────┼───────────────────┘
                            │
         ┌──────────────────▼──────────────────┐
         │      API Service Layer              │
         │  (Бизнес-логика, framework-agnostic)│
         ├─────────────────────────────────────┤
         │  • FileService                      │
         │  • BatchService                     │
         │  • BookmarkService                  │
         │  • SystemService                    │
         │  • ConfigService                    │
         └─────────────────┬───────────────────┘
                           │
         ┌─────────────────▼───────────────────┐
         │      Core Layer                     │
         │  • FileSystem trait                 │
         │  • RealFileSystem                   │
         │  • VirtualFileSystem                │
         └─────────────────────────────────────┘
```

## Структура

```
src-tauri/src/
├── api_service/              # Универсальный API слой
│   ├── mod.rs               # Главный модуль с Api facade
│   ├── models.rs            # Общие типы и модели
│   ├── files.rs             # FileService - операции с файлами
│   ├── batch.rs             # BatchService - пакетные операции
│   ├── bookmarks.rs         # BookmarkService - закладки
│   ├── system.rs            # SystemService - системные операции
│   └── config.rs            # ConfigService - конфигурация
│
├── commands.rs              # Tauri команды (тонкая обертка над API)
└── api_server/              # HTTP сервер (тонкая обертка над API)
    └── handlers/            # HTTP handlers используют api_service
```

## Использование

### 1. Прямое использование (в Rust коде)

```rust
use vfdir_lib::api_service::{Api, API};

fn main() {
    // Использовать глобальный singleton
    let files = API.files.list_directory("/Users").unwrap();
    println!("Files: {:?}", files);

    // Или создать новый экземпляр
    let api = Api::new();
    let home = api.system.get_home_directory().unwrap();
    println!("Home: {}", home);
}
```

### 2. В Tauri командах

```rust
use crate::api_service::API;

#[tauri::command]
pub fn read_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    API.files
        .list_directory(&path)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn create_folder(path: String, name: String) -> Result<(), String> {
    API.files
        .create_folder(&path, &name)
        .map_err(|e| e.message)
}

#[tauri::command]
pub fn get_bookmarks() -> Result<Vec<Bookmark>, String> {
    API.bookmarks
        .get_all()
        .map_err(|e| e.message)
}
```

### 3. В HTTP REST API handlers

```rust
use axum::{Json, extract::Query};
use crate::api_service::API;

pub async fn list_directory(
    Query(params): Query<ListDirQuery>,
) -> Result<Json<Vec<FileSystemEntry>>, ApiError> {
    let files = API.files
        .list_directory(&params.path)
        .map_err(into_http_error)?;

    Ok(Json(files))
}

pub async fn create_folder(
    Json(req): Json<CreateFolderRequest>,
) -> Result<StatusCode, ApiError> {
    API.files
        .create_folder(&req.path, &req.name)
        .map_err(into_http_error)?;

    Ok(StatusCode::OK)
}
```

### 4. В тестах

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations() {
        let api = Api::new();

        // Создать папку
        api.files.create_folder("/tmp", "test_folder").unwrap();

        // Проверить что создалась
        assert!(api.files.file_exists("/tmp/test_folder"));

        // Удалить
        api.files.delete_item("/tmp/test_folder").unwrap();
    }
}
```

## API Services

### FileService

Операции с файлами и директориями:

```rust
let file_service = &API.files;

// Список файлов
let files = file_service.list_directory("/Users")?;

// Информация о файле
let info = file_service.get_file_info("/path/to/file")?;

// Создать папку
file_service.create_folder("/parent/path", "new_folder")?;

// Копировать файлы
file_service.copy_items(&["/file1", "/file2"], "/destination")?;

// Переместить файлы
file_service.move_items(&["/file1"], "/destination")?;

// Переименовать
file_service.rename_item("/old/path", "new_name")?;

// Удалить
file_service.delete_item("/path/to/file")?;
file_service.delete_items(&["/file1", "/file2"])?;

// Прочитать содержимое
let content = file_service.read_file_content("/file.txt", None)?;

// Открыть файл
file_service.open_file("/document.pdf")?;

// Показать в файловом менеджере
file_service.reveal_in_finder("/path/to/file")?;

// Нормализовать путь
let normalized = file_service.normalize_path("/path/../to/./file")?;

// Автодополнение путей
let suggestions = file_service.get_path_suggestions("/Us")?;

// Проверить существование
if file_service.file_exists("/path/to/file") {
    println!("File exists!");
}

// Получить расширение
if let Some(ext) = file_service.get_extension("/file.txt") {
    println!("Extension: {}", ext);
}
```

### BatchService

Пакетные операции:

```rust
let batch_service = &API.batch;

// Предпросмотр переименования
let preview = batch_service.preview_rename(&BatchRenameRequest {
    files: vec!["/file1.txt".to_string(), "/file2.txt".to_string()],
    patterns: vec![/* ... */],
    apply_to_folders: false,
    apply_to_files: true,
    preserve_extension: true,
})?;

// Запустить пакетное переименование
let result = batch_service.queue_rename(&request)?;
println!("Operation ID: {}", result.operation_id);

// Изменить атрибуты
batch_service.queue_attribute_change(&BatchAttributeRequest {
    files: vec!["/file1".to_string()],
    permissions: Some(PermissionsChange { /* ... */ }),
    dates: None,
    tags: None,
})?;

// Получить все операции
let operations = batch_service.get_operations()?;

// Отменить операцию
batch_service.cancel_operation("operation-id")?;

// Повторить операцию
batch_service.retry_operation("operation-id")?;
```

### BookmarkService

Управление закладками:

```rust
let bookmark_service = &API.bookmarks;

// Получить все закладки
let bookmarks = bookmark_service.get_all()?;

// Добавить закладку
let bookmark = bookmark_service.add(
    "/path/to/folder".to_string(),
    Some("My Bookmark".to_string())
)?;

// Удалить закладку
bookmark_service.remove(&bookmark.id)?;

// Переименовать закладку
bookmark_service.rename(&bookmark.id, "New Name".to_string())?;
```

### SystemService

Системные операции:

```rust
let system_service = &API.system;

// Домашняя директория
let home = system_service.get_home_directory()?;

// Системные папки (Desktop, Documents, etc.)
let folders = system_service.get_system_folders()?;

// Статистика системы
let stats = system_service.get_stats()?;
println!("Memory: {} MB, CPU: {}%", stats.memory_mb, stats.cpu_percent);

// Открыть терминал
system_service.open_terminal("/path/to/directory")?;

// Рассчитать размер директории
let size = system_service.calculate_directory_size("/path")?;
```

### ConfigService

Конфигурация приложения:

```rust
let config_service = &API.config;

// Получить конфигурацию
let config = config_service.get()?;

// Обновить конфигурацию
config_service.update(new_config)?;

// UI state
let ui_state = config_service.get_ui_state()?;
config_service.save_ui_state(new_ui_state)?;

// Сменить backend файловой системы
config_service.set_filesystem_backend("real")?; // или "virtual"
```

## Обработка ошибок

Все методы возвращают `ApiResult<T>` = `Result<T, ApiError>`:

```rust
use vfdir_lib::api_service::models::{ApiResult, ApiError};

match API.files.list_directory("/nonexistent") {
    Ok(files) => println!("Files: {:?}", files),
    Err(err) => {
        println!("Error code: {}", err.code);
        println!("Error message: {}", err.message);
    }
}
```

Типы ошибок:
- `FILE_NOT_FOUND` - файл или директория не найдены
- `PERMISSION_DENIED` - доступ запрещен
- `INVALID_PATH` - неверный путь
- `OPERATION_FAILED` - операция не выполнена
- `VALIDATION_ERROR` - ошибка валидации

Создание ошибок:

```rust
return Err(ApiError::file_not_found("/path"));
return Err(ApiError::permission_denied("/path"));
return Err(ApiError::invalid_path("/path"));
return Err(ApiError::operation_failed("Something went wrong"));
return Err(ApiError::validation_error("Invalid input"));
return Err(ApiError::new("CUSTOM_CODE", "Custom message"));
```

## Расширение

### Добавить новый метод в существующий сервис

```rust
// В api_service/files.rs
impl FileService {
    pub fn my_new_operation(&self, path: &str) -> ApiResult<String> {
        // Ваша логика
        Ok("result".to_string())
    }
}
```

### Создать новый сервис

1. Создать файл `api_service/my_service.rs`:

```rust
use super::models::*;

pub struct MyService {}

impl MyService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn do_something(&self) -> ApiResult<()> {
        Ok(())
    }
}
```

2. Добавить в `api_service/mod.rs`:

```rust
pub mod my_service;
pub use my_service::MyService;

pub struct Api {
    // ...
    pub my_service: MyService,
}

impl Api {
    pub fn new() -> Self {
        Self {
            // ...
            my_service: MyService::new(),
        }
    }
}
```

## Преимущества

✅ **Единый источник истины** - вся бизнес-логика в одном месте
✅ **Framework-agnostic** - не привязан к Tauri или HTTP
✅ **Легко тестировать** - можно тестировать без UI
✅ **Переиспользование** - один код для всех клиентов
✅ **Типобезопасность** - все операции типизированы
✅ **Понятная структура** - логика отделена от транспорта

## Следующие шаги

1. Дополнить `BatchService` реализацией из `useBatchOperations.ts`
2. Обновить все Tauri команды для использования API service
3. Обновить все HTTP handlers для использования API service
4. Добавить интеграционные тесты
5. Добавить примеры для CLI приложения

## Примеры проектов

### CLI инструмент

```rust
use clap::Parser;
use vfdir_lib::api_service::API;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Ls { path: String },
    Mkdir { path: String, name: String },
    Cp { sources: Vec<String>, dest: String },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Ls { path } => {
            let files = API.files.list_directory(&path).unwrap();
            for file in files {
                println!("{}", file.name);
            }
        }
        Command::Mkdir { path, name } => {
            API.files.create_folder(&path, &name).unwrap();
            println!("Created folder: {}", name);
        }
        Command::Cp { sources, dest } => {
            API.files.copy_items(&sources, &dest).unwrap();
            println!("Copied {} files", sources.len());
        }
    }
}
```

## Лицензия

Same as main project
