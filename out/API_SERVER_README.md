# VFDir REST API Server

Universal REST API backend для файлового менеджера, который можно использовать с любыми клиентами: веб, мобильными и консольными приложениями.

## 🚀 Быстрый старт

### Запуск сервера

```bash
# Собрать и запустить сервер
cd src-tauri
cargo build --features api-server --bin vfdir-server
cargo run --features api-server --bin vfdir-server

# Или с опциями
cargo run --features api-server --bin vfdir-server -- --host 0.0.0.0 --port 8080
```

### Опции командной строки

```bash
vfdir-server [OPTIONS]

Options:
  -H, --host <HOST>      Server host address [default: 127.0.0.1]
  -p, --port <PORT>      Server port [default: 3000]
  -v, --verbose          Enable verbose logging
  -h, --help             Print help
```

## 📚 Документация API

После запуска сервера, документация Swagger UI доступна по адресу:

```
http://localhost:3000/swagger-ui/
```

OpenAPI спецификация:

```
http://localhost:3000/api-docs/openapi.json
```

## 🔌 Endpoints

### File Operations

- `GET /api/v1/files?path=/path` - Список файлов в директории
- `GET /api/v1/files/info?path=/path/to/file` - Информация о файле
- `POST /api/v1/files/create-folder` - Создать папку
- `POST /api/v1/files/copy` - Копировать файлы
- `POST /api/v1/files/move` - Переместить файлы
- `POST /api/v1/files/rename` - Переименовать файл
- `DELETE /api/v1/files` - Удалить файлы
- `GET /api/v1/files/content?path=/path/to/file` - Прочитать содержимое файла
- `POST /api/v1/files/open` - Открыть файл системным приложением
- `POST /api/v1/files/reveal` - Показать файл в файловом менеджере

### Batch Operations

- `POST /api/v1/batch/rename` - Пакетное переименование
- `POST /api/v1/batch/rename/preview` - Предпросмотр переименования
- `POST /api/v1/batch/attributes` - Пакетное изменение атрибутов
- `GET /api/v1/batch/operations` - Список операций
- `GET /api/v1/batch/operations/:id` - Детали операции
- `DELETE /api/v1/batch/operations/:id` - Отменить операцию
- `POST /api/v1/batch/operations/:id/retry` - Повторить операцию

### Bookmarks

- `GET /api/v1/bookmarks` - Список закладок
- `POST /api/v1/bookmarks` - Добавить закладку
- `DELETE /api/v1/bookmarks/:id` - Удалить закладку
- `PUT /api/v1/bookmarks/:id` - Переименовать закладку

### System

- `GET /api/v1/system/home` - Домашняя директория
- `GET /api/v1/system/folders` - Системные папки
- `GET /api/v1/system/stats` - Статистика системы
- `POST /api/v1/system/terminal` - Открыть терминал

### Configuration

- `GET /api/v1/config` - Получить конфигурацию
- `PUT /api/v1/config` - Обновить конфигурацию
- `GET /api/v1/config/ui-state` - Получить UI state
- `PUT /api/v1/config/ui-state` - Сохранить UI state

### WebSocket

- `WS ws://localhost:3000/api/v1/ws/operations` - Real-time обновления операций
- `WS ws://localhost:3000/api/v1/ws/filesystem` - Real-time изменения файловой системы

## 💡 Примеры использования

### cURL

```bash
# Список файлов
curl "http://localhost:3000/api/v1/files?path=/Users"

# Создать папку
curl -X POST http://localhost:3000/api/v1/files/create-folder \
  -H "Content-Type: application/json" \
  -d '{"path": "/Users/test", "name": "new_folder"}'

# Копировать файлы
curl -X POST http://localhost:3000/api/v1/files/copy \
  -H "Content-Type: application/json" \
  -d '{
    "sources": ["/path/to/file1.txt", "/path/to/file2.txt"],
    "destination": "/path/to/destination"
  }'

# Пакетное переименование
curl -X POST http://localhost:3000/api/v1/batch/rename \
  -H "Content-Type: application/json" \
  -d '{
    "files": ["/path/to/file1.txt", "/path/to/file2.txt"],
    "config": {
      "patterns": [{
        "type": "prefix",
        "enabled": true,
        "text": "new_"
      }],
      "applyToFolders": false,
      "applyToFiles": true,
      "preserveExtension": true
    }
  }'
```

### JavaScript/TypeScript

```typescript
// Список файлов
const response = await fetch('http://localhost:3000/api/v1/files?path=/Users');
const data = await response.json();
console.log(data.files);

// Создать папку
await fetch('http://localhost:3000/api/v1/files/create-folder', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    path: '/Users/test',
    name: 'new_folder'
  })
});

// WebSocket для real-time обновлений
const ws = new WebSocket('ws://localhost:3000/api/v1/ws/operations');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.type === 'progress') {
    console.log('Operation progress:', message.data);
  }
};
```

### Python

```python
import requests

# Список файлов
response = requests.get('http://localhost:3000/api/v1/files', params={'path': '/Users'})
files = response.json()['files']

# Создать папку
requests.post('http://localhost:3000/api/v1/files/create-folder', json={
    'path': '/Users/test',
    'name': 'new_folder'
})

# Пакетное переименование
requests.post('http://localhost:3000/api/v1/batch/rename', json={
    'files': ['/path/to/file1.txt', '/path/to/file2.txt'],
    'config': {
        'patterns': [{
            'type': 'prefix',
            'enabled': True,
            'text': 'new_'
        }],
        'applyToFolders': False,
        'applyToFiles': True,
        'preserveExtension': True
    }
})
```

### React Example (Web Client)

```tsx
import { useState, useEffect } from 'react';

function FileManager() {
  const [files, setFiles] = useState([]);

  useEffect(() => {
    // Загрузить файлы
    fetch('http://localhost:3000/api/v1/files?path=/Users')
      .then(res => res.json())
      .then(data => setFiles(data.files));

    // WebSocket для real-time обновлений
    const ws = new WebSocket('ws://localhost:3000/api/v1/ws/filesystem');

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.type === 'change') {
        // Обновить файлы при изменениях
        refetchFiles();
      }
    };

    return () => ws.close();
  }, []);

  return (
    <div>
      {files.map(file => (
        <div key={file.path}>{file.name}</div>
      ))}
    </div>
  );
}
```

## 🔒 CORS

API поддерживает CORS для веб-клиентов:

- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

## 📱 Использование в мобильных приложениях

### React Native

```typescript
import axios from 'axios';

const API_BASE = 'http://192.168.1.100:3000/api/v1';

export const fileService = {
  async listFiles(path: string) {
    const response = await axios.get(`${API_BASE}/files`, {
      params: { path }
    });
    return response.data.files;
  },

  async createFolder(path: string, name: string) {
    await axios.post(`${API_BASE}/files/create-folder`, {
      path,
      name
    });
  }
};
```

### Flutter/Dart

```dart
import 'package:http/http.dart' as http;
import 'dart:convert';

class FileService {
  final String baseUrl = 'http://192.168.1.100:3000/api/v1';

  Future<List<dynamic>> listFiles(String path) async {
    final response = await http.get(
      Uri.parse('$baseUrl/files?path=$path'),
    );
    final data = json.decode(response.body);
    return data['files'];
  }

  Future<void> createFolder(String path, String name) async {
    await http.post(
      Uri.parse('$baseUrl/files/create-folder'),
      headers: {'Content-Type': 'application/json'},
      body: json.encode({'path': path, 'name': name}),
    );
  }
}
```

## 🖥️ Консольное приложение (CLI)

Пример простого CLI на Python:

```python
#!/usr/bin/env python3
import requests
import sys

API_BASE = 'http://localhost:3000/api/v1'

def list_files(path):
    response = requests.get(f'{API_BASE}/files', params={'path': path})
    files = response.json()['files']
    for file in files:
        print(f"{'[DIR]' if file['isDir'] else '[FILE]'} {file['name']}")

def create_folder(path, name):
    requests.post(f'{API_BASE}/files/create-folder', json={
        'path': path,
        'name': name
    })
    print(f'Created folder: {name}')

if __name__ == '__main__':
    command = sys.argv[1] if len(sys.argv) > 1 else 'help'

    if command == 'ls':
        path = sys.argv[2] if len(sys.argv) > 2 else '/Users'
        list_files(path)
    elif command == 'mkdir':
        path = sys.argv[2]
        name = sys.argv[3]
        create_folder(path, name)
    else:
        print('Usage: vfdir-cli <command> [args]')
        print('Commands: ls <path>, mkdir <path> <name>')
```

## 🔐 Безопасность

**⚠️ ВАЖНО:** Текущая версия НЕ включает authentication. Для production использования необходимо:

1. Добавить JWT authentication
2. Настроить HTTPS/TLS
3. Ограничить CORS origins
4. Добавить rate limiting
5. Валидацию и санитизацию всех входных данных

## 📊 Производительность

- Асинхронная обработка запросов (Tokio runtime)
- WebSocket для real-time updates
- Поддержка batch операций
- Эффективная работа с большими файлами

## 🛠️ Разработка

### Добавление новых endpoints

1. Добавить модель в `api_server/models.rs`
2. Создать handler в `api_server/handlers/`
3. Зарегистрировать route в `api_server/mod.rs`
4. Обновить OpenAPI документацию

### Тестирование API

```bash
# Запустить сервер
cargo run --features api-server --bin vfdir-server

# В другом терминале
curl http://localhost:3000/api/v1/files?path=/Users
```

## 📝 License

Same as main project

## 🤝 Contributing

Contributions are welcome! Please see main project README for guidelines.
