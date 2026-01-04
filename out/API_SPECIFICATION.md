# VFDir REST API Specification v1.0

## Обзор

VFDir предоставляет полнофункциональный REST API для управления файловой системой. API может использоваться веб-приложениями, мобильными клиентами и консольными утилитами.

**Base URL**: `http://localhost:3000/api/v1`

**Swagger UI**: `http://localhost:3000/swagger-ui/`

**WebSocket Endpoints**:
- Operations Progress: `ws://localhost:3000/api/v1/ws/operations`
- FileSystem Changes: `ws://localhost:3000/api/v1/ws/filesystem`

## Содержание

1. [Аутентификация](#аутентификация)
2. [Общие типы данных](#общие-типы-данных)
3. [Обработка ошибок](#обработка-ошибок)
4. [File Operations API](#file-operations-api)
5. [Batch Operations API](#batch-operations-api)
6. [Bookmarks API](#bookmarks-api)
7. [System API](#system-api)
8. [Configuration API](#configuration-api)
9. [WebSocket API](#websocket-api)

---

## Аутентификация

В текущей версии API не требует аутентификации. Все endpoints доступны без токенов.

**CORS**: Разрешены запросы с любых origins (`Access-Control-Allow-Origin: *`)

---

## Общие типы данных

### FileSystemEntry

Представление файла или директории в файловой системе.

```json
{
  "path": "/Users/john/Documents/file.txt",
  "name": "file.txt",
  "isDir": false,
  "isFile": true,
  "size": 1024,
  "modified": 1672531200000,
  "created": 1672444800000,
  "accessed": 1672617600000
}
```

**Поля:**
- `path` (string) - Полный путь к файлу/директории
- `name` (string) - Имя файла/директории
- `isDir` (boolean) - Является ли директорией
- `isFile` (boolean) - Является ли файлом
- `size` (number | null) - Размер в байтах (null для директорий)
- `modified` (number | null) - Unix timestamp последней модификации (миллисекунды)
- `created` (number | null) - Unix timestamp создания (миллисекунды)
- `accessed` (number | null) - Unix timestamp последнего доступа (миллисекунды)

### Bookmark

Закладка на директорию.

```json
{
  "id": "uuid-string",
  "name": "Projects",
  "path": "/Users/john/Projects",
  "createdAt": 1672531200000
}
```

**Поля:**
- `id` (string) - Уникальный идентификатор
- `name` (string) - Отображаемое имя
- `path` (string) - Путь к директории
- `createdAt` (number) - Unix timestamp создания (миллисекунды)

---

## Обработка ошибок

Все ошибки возвращаются в едином формате:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

### Коды ошибок

| Код | HTTP Status | Описание |
|-----|-------------|----------|
| `FILE_NOT_FOUND` | 404 | Файл или директория не найдены |
| `PERMISSION_DENIED` | 403 | Нет прав доступа |
| `INVALID_PATH` | 400 | Некорректный путь |
| `VALIDATION_ERROR` | 400 | Ошибка валидации входных данных |
| `OPERATION_FAILED` | 500 | Операция не выполнена |
| `NOT_IMPLEMENTED` | 501 | Функциональность не реализована |

### Примеры ошибок

```json
{
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "File not found: /path/to/missing/file.txt"
  }
}
```

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Empty file name"
  }
}
```

---

## File Operations API

### 1. Список файлов в директории

Получить список файлов и поддиректорий.

**Endpoint**: `GET /files`

**Query Parameters**:
- `path` (string, required) - Путь к директории

**Request Example**:
```bash
GET /api/v1/files?path=/Users/john/Documents
```

**Response 200**:
```json
{
  "files": [
    {
      "path": "/Users/john/Documents/file1.txt",
      "name": "file1.txt",
      "isDir": false,
      "isFile": true,
      "size": 2048,
      "modified": 1672531200000,
      "created": 1672444800000,
      "accessed": 1672617600000
    },
    {
      "path": "/Users/john/Documents/Projects",
      "name": "Projects",
      "isDir": true,
      "isFile": false,
      "size": null,
      "modified": 1672531200000,
      "created": 1672444800000,
      "accessed": 1672617600000
    }
  ]
}
```

**Response 404**: Directory not found

---

### 2. Информация о файле/директории

Получить метаданные конкретного файла или директории.

**Endpoint**: `GET /files/info`

**Query Parameters**:
- `path` (string, required) - Путь к файлу/директории

**Request Example**:
```bash
GET /api/v1/files/info?path=/Users/john/Documents/file.txt
```

**Response 200**: См. [FileSystemEntry](#filesystementry)

**Response 404**: File not found

---

### 3. Создать папку

Создать новую директорию.

**Endpoint**: `POST /files/create-folder`

**Request Body**:
```json
{
  "path": "/Users/john/Documents",
  "name": "NewFolder"
}
```

**Fields**:
- `path` (string) - Путь к родительской директории
- `name` (string) - Имя новой папки

**Response 200**: Empty (успешно создано)

**Response 400**: Invalid path or name
**Response 409**: Folder already exists

---

### 4. Копировать файлы/папки

Копировать один или несколько файлов/директорий.

**Endpoint**: `POST /files/copy`

**Request Body**:
```json
{
  "sources": [
    "/Users/john/Documents/file1.txt",
    "/Users/john/Documents/folder1"
  ],
  "destination": "/Users/john/Backup"
}
```

**Fields**:
- `sources` (string[]) - Массив путей к исходным файлам/папкам
- `destination` (string) - Путь к целевой директории

**Response 200**: Empty (успешно скопировано)

**Response 404**: Source file not found
**Response 400**: Invalid paths

---

### 5. Переместить файлы/папки

Переместить один или несколько файлов/директорий.

**Endpoint**: `POST /files/move`

**Request Body**:
```json
{
  "sources": [
    "/Users/john/Documents/file1.txt"
  ],
  "destination": "/Users/john/Archive"
}
```

**Fields**: Аналогично `/files/copy`

**Response 200**: Empty (успешно перемещено)

**Response 404**: Source file not found
**Response 400**: Invalid paths

---

### 6. Переименовать файл/папку

Переименовать файл или директорию.

**Endpoint**: `POST /files/rename`

**Request Body**:
```json
{
  "oldPath": "/Users/john/Documents/old_name.txt",
  "newName": "new_name.txt"
}
```

**Fields**:
- `oldPath` (string) - Текущий полный путь
- `newName` (string) - Новое имя (без пути)

**Response 200**: Empty (успешно переименовано)

**Response 404**: File not found
**Response 409**: File with new name already exists

---

### 7. Удалить файлы/папки

Удалить один или несколько файлов/директорий.

**Endpoint**: `DELETE /files`

**Request Body**:
```json
{
  "paths": [
    "/Users/john/Documents/file1.txt",
    "/Users/john/Documents/folder1"
  ]
}
```

**Fields**:
- `paths` (string[]) - Массив путей к удаляемым файлам/папкам

**Response 200**: Empty (успешно удалено)

**Response 404**: File not found
**Response 403**: Permission denied

---

### 8. Прочитать содержимое файла

Получить текстовое содержимое файла.

**Endpoint**: `GET /files/content`

**Query Parameters**:
- `path` (string, required) - Путь к файлу
- `maxSize` (number, optional) - Максимальный размер для чтения (байты)

**Request Example**:
```bash
GET /api/v1/files/content?path=/Users/john/file.txt&maxSize=10240
```

**Response 200**:
```json
{
  "content": "File content as string..."
}
```

**Response 404**: File not found
**Response 400**: File too large (if maxSize exceeded)

---

### 9. Открыть файл в системном приложении

Открыть файл в ассоциированном приложении ОС.

**Endpoint**: `POST /files/open`

**Request Body**:
```json
{
  "path": "/Users/john/Documents/document.pdf"
}
```

**Response 200**: Empty (файл открыт)

**Response 404**: File not found

---

### 10. Показать в Finder/Explorer

Показать файл в файловом менеджере ОС.

**Endpoint**: `POST /files/reveal`

**Request Body**:
```json
{
  "path": "/Users/john/Documents/file.txt"
}
```

**Response 200**: Empty (файл показан)

**Response 404**: File not found

---

### 11-13. Операции с прогрессом

Копирование, перемещение и удаление с отслеживанием прогресса через WebSocket.

**Endpoints**:
- `POST /files/copy-with-progress`
- `POST /files/move-with-progress`
- `POST /files/delete-with-progress`

**Request Body (copy/move)**:
```json
{
  "sources": ["/path/to/source"],
  "destination": "/path/to/dest",
  "operationId": "optional-custom-id"
}
```

**Request Body (delete)**:
```json
{
  "paths": ["/path/to/file"],
  "operationId": "optional-custom-id"
}
```

**Response 200**:
```json
{
  "operationId": "uuid-generated-or-custom-id"
}
```

После успешного старта операции:
1. Клиент получает `operationId`
2. Подключается к WebSocket `/ws/operations`
3. Получает обновления прогресса в реальном времени

См. [WebSocket API](#websocket-api) для деталей.

---

## Batch Operations API

Массовые операции над файлами.

### 1. Превью массового переименования

Получить предварительный просмотр результатов массового переименования без выполнения операции.

**Endpoint**: `POST /batch/rename/preview`

**Request Body**:
```json
{
  "files": [
    "/Users/john/Documents/file1.txt",
    "/Users/john/Documents/file2.txt"
  ],
  "config": {
    "patterns": [
      {
        "type": "prefix",
        "enabled": true,
        "text": "renamed_"
      },
      {
        "type": "numbering",
        "enabled": true,
        "startNumber": 1,
        "increment": 1,
        "padding": 3,
        "position": "suffix",
        "separator": "_"
      }
    ],
    "applyToFolders": false,
    "applyToFiles": true,
    "preserveExtension": true
  }
}
```

**Pattern Types**:

1. **Prefix** - добавить префикс
```json
{
  "type": "prefix",
  "enabled": true,
  "text": "prefix_"
}
```

2. **Suffix** - добавить суффикс
```json
{
  "type": "suffix",
  "enabled": true,
  "text": "_suffix",
  "beforeExtension": true
}
```

3. **Replace** - найти и заменить
```json
{
  "type": "replace",
  "enabled": true,
  "searchText": "old",
  "replaceText": "new",
  "caseSensitive": false,
  "wholeWord": false
}
```

4. **Regex** - регулярные выражения
```json
{
  "type": "regex",
  "enabled": true,
  "pattern": "\\d+",
  "replacement": "X",
  "flags": "g"
}
```

5. **Numbering** - нумерация
```json
{
  "type": "numbering",
  "enabled": true,
  "startNumber": 1,
  "increment": 1,
  "padding": 3,
  "position": "prefix|suffix",
  "separator": "_"
}
```

6. **Case** - изменить регистр
```json
{
  "type": "case",
  "enabled": true,
  "caseType": "lowercase|uppercase|titlecase|camelcase"
}
```

**Response 200**:
```json
[
  {
    "originalName": "file1.txt",
    "newName": "renamed_001_file1.txt",
    "hasError": false,
    "errorMessage": null
  },
  {
    "originalName": "file2.txt",
    "newName": "renamed_002_file2.txt",
    "hasError": false,
    "errorMessage": null
  }
]
```

**Response 501**: Not implemented yet

---

### 2. Выполнить массовое переименование

Запустить операцию массового переименования.

**Endpoint**: `POST /batch/rename`

**Request Body**: Аналогично `/batch/rename/preview`

**Response 200**:
```json
{
  "operationId": "uuid",
  "preview": [
    {
      "originalName": "file1.txt",
      "newName": "renamed_file1.txt",
      "hasError": false,
      "errorMessage": null
    }
  ]
}
```

**Response 501**: Not implemented yet

---

### 3. Массовое изменение атрибутов

Изменить права доступа, даты и теги для группы файлов.

**Endpoint**: `POST /batch/attributes`

**Request Body**:
```json
{
  "files": [
    "/Users/john/Documents/file1.txt",
    "/Users/john/Documents/file2.txt"
  ],
  "changes": {
    "permissions": {
      "readable": true,
      "writable": false,
      "executable": false,
      "recursive": true
    },
    "dates": {
      "modified": 1672531200000,
      "created": 1672444800000,
      "accessed": null
    },
    "tags": {
      "operation": "add|remove|replace",
      "tags": ["work", "important"]
    }
  }
}
```

**Response 200**:
```json
{
  "operationId": "uuid"
}
```

**Response 501**: Not implemented yet

---

### 4. Список операций в очереди

Получить список всех batch операций.

**Endpoint**: `GET /batch/operations`

**Response 200**:
```json
[
  {
    "operationId": "uuid",
    "operationType": "copy|move|delete",
    "status": "running|paused|completed|cancelled|failed",
    "currentBytes": 1024000,
    "totalBytes": 2048000,
    "currentItems": 5,
    "totalItems": 10,
    "currentFile": "/path/to/current/file.txt",
    "speedBytesPerSec": 102400.5,
    "etaSeconds": 10.5,
    "errorMessage": null
  }
]
```

---

### 5. Информация об операции

Получить детали конкретной операции.

**Endpoint**: `GET /batch/operations/:id`

**Response 200**: См. формат в `/batch/operations`

**Response 404**: Operation not found

---

### 6. Отменить операцию

Отменить выполняющуюся операцию.

**Endpoint**: `DELETE /batch/operations/:id`

**Response 200**: Empty (операция отменена)

**Response 404**: Operation not found

---

### 7. Повторить операцию

Повторить failed операцию.

**Endpoint**: `POST /batch/operations/:id/retry`

**Response 200**: Empty (операция перезапущена)

**Response 404**: Operation not found
**Response 501**: Not implemented yet

---

## Bookmarks API

### 1. Получить все закладки

**Endpoint**: `GET /bookmarks`

**Response 200**:
```json
{
  "bookmarks": [
    {
      "id": "uuid-1",
      "name": "Documents",
      "path": "/Users/john/Documents",
      "createdAt": 1672531200000
    },
    {
      "id": "uuid-2",
      "name": "Projects",
      "path": "/Users/john/Projects",
      "createdAt": 1672444800000
    }
  ]
}
```

---

### 2. Добавить закладку

**Endpoint**: `POST /bookmarks`

**Request Body**:
```json
{
  "path": "/Users/john/Documents",
  "name": "My Documents"
}
```

**Fields**:
- `path` (string) - Путь к директории
- `name` (string, optional) - Отображаемое имя (если null, используется имя директории)

**Response 201**:
```json
{
  "id": "uuid",
  "name": "My Documents",
  "path": "/Users/john/Documents",
  "createdAt": 1672531200000
}
```

**Response 400**: Invalid path or validation error

---

### 3. Удалить закладку

**Endpoint**: `DELETE /bookmarks/:id`

**Response 204**: Empty (закладка удалена)

**Response 404**: Bookmark not found

---

### 4. Переименовать закладку

**Endpoint**: `PUT /bookmarks/:id`

**Request Body**:
```json
{
  "newName": "Updated Name"
}
```

**Response 200**: Empty (закладка переименована)

**Response 404**: Bookmark not found
**Response 400**: Invalid name

---

## System API

### 1. Получить домашнюю директорию

**Endpoint**: `GET /system/home`

**Response 200**:
```json
{
  "path": "/Users/john"
}
```

---

### 2. Получить системные папки

Получить список важных системных директорий (Desktop, Documents, Downloads и т.д.)

**Endpoint**: `GET /system/folders`

**Response 200**:
```json
{
  "folders": [
    {
      "path": "/Users/john/Desktop",
      "name": "Desktop",
      "isDir": true,
      "isFile": false,
      "size": null,
      "modified": 1672531200000,
      "created": 1672444800000,
      "accessed": 1672617600000
    },
    {
      "path": "/Users/john/Documents",
      "name": "Documents",
      "isDir": true,
      "isFile": false,
      "size": null,
      "modified": 1672531200000,
      "created": 1672444800000,
      "accessed": 1672617600000
    }
  ]
}
```

---

### 3. Системная статистика

Получить информацию об использовании системных ресурсов.

**Endpoint**: `GET /system/stats`

**Response 200**:
```json
{
  "memoryMb": 8192.5,
  "cpuPercent": 45.2
}
```

**Fields**:
- `memoryMb` (number) - Использование памяти в МБ
- `cpuPercent` (number) - Загрузка CPU в процентах

---

### 4. Открыть терминал

Открыть терминал в указанной директории.

**Endpoint**: `POST /system/terminal`

**Request Body**:
```json
{
  "path": "/Users/john/Projects"
}
```

**Response 200**: Empty (терминал открыт)

**Response 404**: Directory not found

---

## Configuration API

### 1. Получить конфигурацию

**Endpoint**: `GET /config`

**Response 200**:
```json
{
  "filesystemBackend": "real|virtual",
  "showHiddenFiles": false,
  "defaultViewMode": "grid|list|details",
  "theme": "luna",
  "bookmarks": [...],
  "uiState": {
    "sidebarWidth": 240,
    "previewWidth": 300,
    "tabs": [...],
    "activeTabId": 123,
    "lastPath": ["Users", "john", "Documents"],
    "panelMode": "single|dual",
    "dualPanelConfig": {
      "leftPanelWidthPercent": 50,
      "leftPanel": {...},
      "rightPanel": {...},
      "activePanel": "left|right"
    },
    "window": {
      "width": 1200,
      "height": 800,
      "x": 100,
      "y": 100,
      "maximized": false
    },
    "sidebar": {
      "expandedFolders": ["/Users/john"],
      "quickAccessExpanded": true,
      "folderTreeExpanded": true,
      "favoritesExpanded": false
    }
  }
}
```

---

### 2. Обновить конфигурацию

**Endpoint**: `PUT /config`

**Request Body**: Полный объект конфигурации (см. GET /config)

**Response 200**: Empty (конфигурация обновлена)

**Response 400**: Invalid configuration

---

### 3. Получить UI состояние

**Endpoint**: `GET /config/ui-state`

**Response 200**: См. поле `uiState` в GET /config

---

### 4. Сохранить UI состояние

**Endpoint**: `PUT /config/ui-state`

**Request Body**: Объект UIState (см. GET /config/ui-state)

**Response 200**: Empty (состояние сохранено)

**Response 400**: Invalid UI state

---

## WebSocket API

### 1. Operations Progress WebSocket

Получение обновлений прогресса для file operations.

**URL**: `ws://localhost:3000/api/v1/ws/operations`

**Protocol**: WebSocket

#### Подключение

```javascript
const ws = new WebSocket('ws://localhost:3000/api/v1/ws/operations');

ws.onopen = () => {
  console.log('Connected to operations WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  handleProgressUpdate(message);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket connection closed');
};
```

#### Формат сообщений

Все сообщения в формате JSON с полем `type`:

**Progress Update**:
```json
{
  "type": "progress",
  "data": {
    "operationId": "uuid",
    "operationType": "copy|move|delete",
    "status": "running|paused|completed|cancelled|failed",
    "currentBytes": 1024000,
    "totalBytes": 2048000,
    "currentItems": 5,
    "totalItems": 10,
    "currentFile": "/path/to/current/file.txt",
    "speedBytesPerSec": 102400.5,
    "etaSeconds": 10.5,
    "errorMessage": null
  }
}
```

**Progress Fields**:
- `operationId` (string) - ID операции
- `operationType` (string) - Тип: "copy", "move", "delete"
- `status` (string) - Статус: "running", "paused", "completed", "cancelled", "failed"
- `currentBytes` (number) - Обработано байт
- `totalBytes` (number) - Всего байт
- `currentItems` (number) - Обработано файлов
- `totalItems` (number) - Всего файлов
- `currentFile` (string | null) - Текущий обрабатываемый файл
- `speedBytesPerSec` (number) - Скорость в байтах/сек
- `etaSeconds` (number | null) - Оценка времени до завершения (сек)
- `errorMessage` (string | null) - Сообщение об ошибке (если status = "failed")

#### Пример использования

```javascript
// 1. Запустить операцию копирования
const response = await fetch('/api/v1/files/copy-with-progress', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    sources: ['/path/to/large/folder'],
    destination: '/path/to/destination'
  })
});

const { operationId } = await response.json();

// 2. Подключиться к WebSocket
const ws = new WebSocket('ws://localhost:3000/api/v1/ws/operations');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  if (message.type === 'progress' && message.data.operationId === operationId) {
    const progress = message.data;

    // Обновить UI
    updateProgressBar(progress.currentBytes / progress.totalBytes * 100);
    updateSpeed(progress.speedBytesPerSec);
    updateETA(progress.etaSeconds);

    if (progress.status === 'completed') {
      console.log('Operation completed!');
      ws.close();
    } else if (progress.status === 'failed') {
      console.error('Operation failed:', progress.errorMessage);
      ws.close();
    }
  }
};
```

---

### 2. FileSystem Changes WebSocket

Получение уведомлений об изменениях в файловой системе.

**URL**: `ws://localhost:3000/api/v1/ws/filesystem`

**Protocol**: WebSocket

#### Формат сообщений

**FileSystem Change Event**:
```json
{
  "type": "change",
  "data": {
    "path": "/Users/john/Documents/file.txt",
    "changeType": "created|modified|deleted"
  }
}
```

**Fields**:
- `path` (string) - Путь к изменённому файлу/директории
- `changeType` (string) - Тип изменения: "created", "modified", "deleted"

#### Пример использования

```javascript
const ws = new WebSocket('ws://localhost:3000/api/v1/ws/filesystem');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  if (message.type === 'change') {
    const { path, changeType } = message.data;

    console.log(`File ${changeType}: ${path}`);

    // Обновить список файлов в UI
    refreshFileList();
  }
};
```

---

## Примеры использования

### Пример 1: Копирование файла с отслеживанием прогресса

```javascript
async function copyFileWithProgress(source, destination) {
  // 1. Запустить операцию
  const response = await fetch('/api/v1/files/copy-with-progress', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      sources: [source],
      destination: destination
    })
  });

  const { operationId } = await response.json();

  // 2. Подключиться к WebSocket
  const ws = new WebSocket('ws://localhost:3000/api/v1/ws/operations');

  return new Promise((resolve, reject) => {
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);

      if (message.type === 'progress' && message.data.operationId === operationId) {
        const { status, currentBytes, totalBytes, speedBytesPerSec } = message.data;

        console.log(`Progress: ${(currentBytes / totalBytes * 100).toFixed(1)}%`);
        console.log(`Speed: ${(speedBytesPerSec / 1024 / 1024).toFixed(2)} MB/s`);

        if (status === 'completed') {
          ws.close();
          resolve();
        } else if (status === 'failed') {
          ws.close();
          reject(new Error(message.data.errorMessage));
        }
      }
    };
  });
}

// Использование
await copyFileWithProgress('/source/large-file.zip', '/destination');
```

---

### Пример 2: Массовое переименование с превью

```javascript
async function batchRename(files, patterns) {
  // 1. Получить превью
  const previewResponse = await fetch('/api/v1/batch/rename/preview', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      files: files,
      config: {
        patterns: patterns,
        applyToFolders: false,
        applyToFiles: true,
        preserveExtension: true
      }
    })
  });

  const preview = await previewResponse.json();

  // 2. Показать превью пользователю
  console.log('Preview:');
  preview.forEach(item => {
    console.log(`${item.originalName} → ${item.newName}`);
  });

  // 3. Подтвердить и выполнить
  const confirmed = confirm('Apply these changes?');
  if (confirmed) {
    const response = await fetch('/api/v1/batch/rename', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        files: files,
        config: {
          patterns: patterns,
          applyToFolders: false,
          applyToFiles: true,
          preserveExtension: true
        }
      })
    });

    const result = await response.json();
    console.log('Operation ID:', result.operationId);
  }
}

// Использование
await batchRename(
  ['/path/file1.txt', '/path/file2.txt'],
  [
    { type: 'prefix', enabled: true, text: 'renamed_' },
    { type: 'numbering', enabled: true, startNumber: 1, increment: 1, padding: 3, position: 'suffix', separator: '_' }
  ]
);
```

---

### Пример 3: Работа с закладками

```javascript
async function manageBookmarks() {
  // 1. Получить все закладки
  const response = await fetch('/api/v1/bookmarks');
  const { bookmarks } = await response.json();

  console.log('Current bookmarks:', bookmarks);

  // 2. Добавить новую закладку
  const addResponse = await fetch('/api/v1/bookmarks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      path: '/Users/john/Projects',
      name: 'My Projects'
    })
  });

  const newBookmark = await addResponse.json();
  console.log('Created bookmark:', newBookmark);

  // 3. Переименовать закладку
  await fetch(`/api/v1/bookmarks/${newBookmark.id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      newName: 'Work Projects'
    })
  });

  // 4. Удалить закладку
  await fetch(`/api/v1/bookmarks/${newBookmark.id}`, {
    method: 'DELETE'
  });
}
```

---

## Лимиты и ограничения

1. **Размер запроса**: Максимум 10 MB для request body
2. **WebSocket**: Максимум 100 одновременных подключений
3. **Batch операции**: Максимум 1000 файлов за один запрос
4. **Файловый контент**: По умолчанию максимум 1 MB при чтении через `/files/content`
5. **Rate limiting**: Нет (в текущей версии)

---

## Версионирование API

Текущая версия: **v1**

API использует версионирование через URL path (`/api/v1/...`).

При изменении breaking changes будет создана новая версия (`/api/v2/...`).

---

## Поддержка

- **GitHub**: https://github.com/your-repo/vfdir
- **Документация**: http://localhost:3000/swagger-ui/
- **Issues**: https://github.com/your-repo/vfdir/issues

---

## Changelog

### v1.0.0 (2025-01-01)
- Первый релиз REST API
- Поддержка базовых file operations
- WebSocket для progress tracking
- Batch operations (частично)
- Bookmarks API
- System API
- Configuration API
