# VFDir REST API Specification

Universal REST API for file manager operations that can be used by web, mobile, and console clients.

## Base URL

```
http://localhost:3000/api/v1
```

## Authentication

Currently no authentication required. Will be added in future versions.

## Endpoints

### File System Operations

#### GET `/files`
List directory contents

**Query Parameters:**
- `path` (string, required) - Directory path

**Response:**
```json
{
  "files": [
    {
      "path": "/path/to/file",
      "name": "file.txt",
      "isDir": false,
      "isFile": true,
      "size": 1024,
      "modified": 1640000000,
      "created": 1640000000,
      "accessed": 1640000000
    }
  ]
}
```

#### GET `/files/info`
Get file/directory information

**Query Parameters:**
- `path` (string, required) - File/directory path

**Response:** Single FileSystemEntry object

#### POST `/files/create-folder`
Create new folder

**Body:**
```json
{
  "path": "/parent/path",
  "name": "new_folder"
}
```

#### POST `/files/copy`
Copy files/folders

**Body:**
```json
{
  "sources": ["/path/to/source1", "/path/to/source2"],
  "destination": "/path/to/destination"
}
```

#### POST `/files/move`
Move files/folders

**Body:**
```json
{
  "sources": ["/path/to/source1"],
  "destination": "/path/to/destination"
}
```

#### POST `/files/rename`
Rename file/folder

**Body:**
```json
{
  "oldPath": "/path/to/old",
  "newName": "new_name"
}
```

#### DELETE `/files`
Delete files/folders

**Body:**
```json
{
  "paths": ["/path/to/file1", "/path/to/file2"]
}
```

#### GET `/files/content`
Read file content

**Query Parameters:**
- `path` (string, required) - File path
- `maxSize` (number, optional) - Max size to read in bytes

**Response:** File content as text

#### POST `/files/open`
Open file with system default application

**Body:**
```json
{
  "path": "/path/to/file"
}
```

#### POST `/files/reveal`
Reveal file in system file manager

**Body:**
```json
{
  "path": "/path/to/file"
}
```

### Batch Operations

#### POST `/batch/rename`
Queue batch rename operation

**Body:**
```json
{
  "files": ["/path/to/file1", "/path/to/file2"],
  "config": {
    "patterns": [
      {
        "type": "prefix",
        "enabled": true,
        "text": "new_"
      }
    ],
    "applyToFolders": true,
    "applyToFiles": true,
    "preserveExtension": true
  }
}
```

**Response:**
```json
{
  "operationId": "batch-1640000000-abc123",
  "preview": [...]
}
```

#### POST `/batch/rename/preview`
Preview batch rename without executing

**Body:** Same as `/batch/rename`

**Response:**
```json
{
  "preview": [
    {
      "originalName": "file1.txt",
      "newName": "new_file1.txt",
      "hasError": false
    }
  ],
  "validation": {
    "isValid": true,
    "errors": [],
    "warnings": []
  }
}
```

#### POST `/batch/attributes`
Queue batch attribute change operation

**Body:**
```json
{
  "files": ["/path/to/file1"],
  "changes": {
    "permissions": {
      "readable": true,
      "writable": true,
      "executable": false,
      "recursive": false
    },
    "dates": {
      "modified": 1640000000
    },
    "tags": {
      "operation": "add",
      "tags": ["work", "important"]
    }
  }
}
```

**Response:**
```json
{
  "operationId": "batch-1640000000-xyz789"
}
```

#### GET `/batch/operations`
Get all batch operations

**Response:**
```json
{
  "operations": [
    {
      "id": "batch-1640000000-abc123",
      "type": "rename",
      "status": "completed",
      "itemsCount": 10,
      "processedCount": 10,
      "failedCount": 0,
      "createdAt": 1640000000,
      "completedAt": 1640000100
    }
  ]
}
```

#### GET `/batch/operations/:id`
Get specific operation details

**Response:** Single operation with results

#### DELETE `/batch/operations/:id`
Cancel/remove operation

#### POST `/batch/operations/:id/retry`
Retry failed operation

### Bookmarks

#### GET `/bookmarks`
Get all bookmarks

**Response:**
```json
{
  "bookmarks": [
    {
      "id": "bookmark-1",
      "name": "Projects",
      "path": "/home/user/projects",
      "created_at": 1640000000
    }
  ]
}
```

#### POST `/bookmarks`
Add bookmark

**Body:**
```json
{
  "path": "/path/to/bookmark",
  "name": "My Bookmark"
}
```

#### DELETE `/bookmarks/:id`
Remove bookmark

#### PUT `/bookmarks/:id`
Rename bookmark

**Body:**
```json
{
  "name": "New Name"
}
```

### System

#### GET `/system/home`
Get home directory path

**Response:**
```json
{
  "path": "/home/user"
}
```

#### GET `/system/folders`
Get system folders (Desktop, Downloads, etc.)

**Response:**
```json
{
  "folders": [...]
}
```

#### GET `/system/stats`
Get system statistics

**Response:**
```json
{
  "memory_mb": 150.5,
  "cpu_percent": 12.3
}
```

#### POST `/system/terminal`
Open terminal at path

**Body:**
```json
{
  "path": "/path/to/directory"
}
```

### Configuration

#### GET `/config`
Get application configuration

**Response:**
```json
{
  "filesystem_backend": "real",
  "show_hidden_files": false,
  "default_view_mode": "list",
  "theme": "classic"
}
```

#### PUT `/config`
Update configuration

**Body:** Full config object

#### GET `/config/ui-state`
Get UI state

#### PUT `/config/ui-state`
Save UI state

### WebSocket (Real-time Updates)

#### WS `/ws/operations`
WebSocket connection for real-time operation progress updates

**Message Format:**
```json
{
  "type": "progress",
  "data": {
    "operationId": "batch-1640000000-abc123",
    "operationType": "rename",
    "status": "running",
    "currentBytes": 1024,
    "totalBytes": 10240,
    "currentItems": 1,
    "totalItems": 10,
    "currentFile": "/path/to/file",
    "speedBytesPerSec": 1024000,
    "etaSeconds": 9
  }
}
```

#### WS `/ws/filesystem`
WebSocket for file system change notifications

**Message Format:**
```json
{
  "type": "change",
  "data": {
    "path": "/path/to/changed/file",
    "changeType": "created|modified|deleted"
  }
}
```

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

Common error codes:
- `FILE_NOT_FOUND` - File or directory not found
- `PERMISSION_DENIED` - Permission denied
- `INVALID_PATH` - Invalid path provided
- `OPERATION_FAILED` - Operation failed
- `VALIDATION_ERROR` - Validation failed

## CORS

The API supports CORS for web clients:
- `Access-Control-Allow-Origin: *` (configurable)
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

## Rate Limiting

Currently no rate limiting. Will be added in future versions.

## Versioning

API version is included in the URL path (`/api/v1`). Future versions will maintain backward compatibility.
