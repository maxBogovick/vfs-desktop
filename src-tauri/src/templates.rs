use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Шаблон файла с предопределенным содержимым
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTemplate {
    pub id: String,
    pub name: String,
    pub content: String,
    pub category: String,
    pub icon: String,
    pub file_extensions: Vec<String>,
    pub context_patterns: Vec<String>,
}

/// Реестр шаблонов файлов
pub struct TemplateRegistry {
    templates: Vec<FileTemplate>,
}

impl TemplateRegistry {
    /// Создать новый реестр с встроенными шаблонами
    pub fn new() -> Self {
        let templates = vec![
            // Git
            FileTemplate {
                id: "gitignore".to_string(),
                name: ".gitignore".to_string(),
                content: r#"# Dependencies
node_modules/
vendor/

# Build outputs
dist/
build/
out/
target/

# Environment
.env
.env.local
.env.*.local

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*
"#.to_string(),
                category: "Git".to_string(),
                icon: "🔧".to_string(),
                file_extensions: vec![".gitignore".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // Node.js
            FileTemplate {
                id: "package_json".to_string(),
                name: "package.json".to_string(),
                content: r#"{
  "name": "",
  "version": "1.0.0",
  "description": "",
  "main": "index.js",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1",
    "start": "node index.js",
    "dev": "nodemon index.js"
  },
  "keywords": [],
  "author": "",
  "license": "ISC",
  "dependencies": {},
  "devDependencies": {}
}
"#.to_string(),
                category: "Node.js".to_string(),
                icon: "📦".to_string(),
                file_extensions: vec!["package.json".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // Documentation
            FileTemplate {
                id: "readme".to_string(),
                name: "README.md".to_string(),
                content: r#"# Project Name

## Description

A brief description of your project.

## Installation

```bash
# Add installation instructions here
```

## Usage

```bash
# Add usage examples here
```

## Features

- Feature 1
- Feature 2
- Feature 3

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details.
"#.to_string(),
                category: "Documentation".to_string(),
                icon: "📖".to_string(),
                file_extensions: vec![".md".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // Vue 3
            FileTemplate {
                id: "vue_component".to_string(),
                name: "Component.vue".to_string(),
                content: r#"<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

// Props
interface Props {
  // Add props here
}

const props = defineProps<Props>()

// Emits
interface Emits {
  // Add emits here
}

const emit = defineEmits<Emits>()

// State
const state = ref('')

// Computed
const computed

Value = computed(() => {
  return state.value.toUpperCase()
})

// Methods
const handleClick = () => {
  // Handle click
}

// Lifecycle
onMounted(() => {
  // Component mounted
})
</script>

<template>
  <div class="component">
    <!-- Template content -->
  </div>
</template>

<style scoped>
.component {
  /* Component styles */
}
</style>
"#.to_string(),
                category: "Vue".to_string(),
                icon: "💚".to_string(),
                file_extensions: vec![".vue".to_string()],
                context_patterns: vec!["**/components/**".to_string(), "**/src/**".to_string()],
            },

            // Rust
            FileTemplate {
                id: "rust_module".to_string(),
                name: "module.rs".to_string(),
                content: r#"//! Module documentation
//!
//! Describe what this module does

use std::fmt;

/// Main struct for this module
#[derive(Debug, Clone)]
pub struct Example {
    // Add fields here
}

impl Example {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            // Initialize fields
        }
    }

    /// Example method
    pub fn do_something(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let example = Example::new();
        assert!(example.do_something().is_ok());
    }
}
"#.to_string(),
                category: "Rust".to_string(),
                icon: "🦀".to_string(),
                file_extensions: vec![".rs".to_string()],
                context_patterns: vec!["**/src/**".to_string()],
            },

            // TypeScript
            FileTemplate {
                id: "ts_interface".to_string(),
                name: "types.ts".to_string(),
                content: r#"/**
 * Main interface documentation
 */
export interface Example {
  id: string
  name: string
  createdAt: Date
  updatedAt?: Date
}

/**
 * Type aliases
 */
export type ExampleId = string
export type ExampleStatus = 'active' | 'inactive' | 'pending'

/**
 * Utility types
 */
export type PartialExample = Partial<Example>
export type RequiredExample = Required<Example>

/**
 * Constants
 */
export const EXAMPLE_CONSTANTS = {
  MAX_LENGTH: 100,
  MIN_LENGTH: 1,
} as const
"#.to_string(),
                category: "TypeScript".to_string(),
                icon: "📘".to_string(),
                file_extensions: vec![".ts".to_string(), ".d.ts".to_string()],
                context_patterns: vec!["**/types/**".to_string(), "**/src/**".to_string()],
            },

            // JavaScript
            FileTemplate {
                id: "js_module".to_string(),
                name: "module.js".to_string(),
                content: r#"/**
 * Module description
 * @module Example
 */

/**
 * Main function
 * @param {string} input - Input parameter
 * @returns {string} Output result
 */
export function example(input) {
  return input.toUpperCase()
}

/**
 * Class example
 */
export class Example {
  constructor(name) {
    this.name = name
  }

  greet() {
    return `Hello, ${this.name}!`
  }
}

export default {
  example,
  Example,
}
"#.to_string(),
                category: "JavaScript".to_string(),
                icon: "💛".to_string(),
                file_extensions: vec![".js".to_string(), ".mjs".to_string()],
                context_patterns: vec!["**/src/**".to_string()],
            },

            // JSON
            FileTemplate {
                id: "json_config".to_string(),
                name: "config.json".to_string(),
                content: r#"{
  "version": "1.0.0",
  "settings": {
    "enabled": true,
    "port": 3000,
    "host": "localhost"
  },
  "features": [
    "feature1",
    "feature2"
  ]
}
"#.to_string(),
                category: "Config".to_string(),
                icon: "⚙️".to_string(),
                file_extensions: vec![".json".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // HTML
            FileTemplate {
                id: "html_page".to_string(),
                name: "index.html".to_string(),
                content: r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <title>Page Title</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <header>
        <h1>Welcome</h1>
    </header>

    <main>
        <section>
            <p>Content goes here</p>
        </section>
    </main>

    <footer>
        <p>&copy; 2024</p>
    </footer>

    <script src="script.js"></script>
</body>
</html>
"#.to_string(),
                category: "Web".to_string(),
                icon: "🌐".to_string(),
                file_extensions: vec![".html".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // CSS
            FileTemplate {
                id: "css_stylesheet".to_string(),
                name: "styles.css".to_string(),
                content: r#"/* Reset and base styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:root {
    --primary-color: #3498db;
    --secondary-color: #2ecc71;
    --text-color: #333;
    --bg-color: #fff;
    --font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

body {
    font-family: var(--font-family);
    color: var(--text-color);
    background-color: var(--bg-color);
    line-height: 1.6;
}

/* Components */
.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

.button {
    display: inline-block;
    padding: 10px 20px;
    background-color: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s ease;
}

.button:hover {
    background-color: #2980b9;
}
"#.to_string(),
                category: "Web".to_string(),
                icon: "🎨".to_string(),
                file_extensions: vec![".css".to_string()],
                context_patterns: vec!["**".to_string()],
            },

            // Python
            FileTemplate {
                id: "python_module".to_string(),
                name: "module.py".to_string(),
                content: r#"""
Module documentation

This module provides...
"""

from typing import List, Optional, Dict, Any


class Example:
    """Example class documentation"""

    def __init__(self, name: str):
        """
        Initialize the Example

        Args:
            name: The name parameter
        """
        self.name = name

    def greet(self) -> str:
        """Return a greeting message"""
        return f"Hello, {self.name}!"

    def __repr__(self) -> str:
        return f"Example(name='{self.name}')"


def example_function(param: str) -> str:
    """
    Example function documentation

    Args:
        param: Input parameter

    Returns:
        Processed result
    """
    return param.upper()


if __name__ == "__main__":
    # Example usage
    example = Example("World")
    print(example.greet())
"#.to_string(),
                category: "Python".to_string(),
                icon: "🐍".to_string(),
                file_extensions: vec![".py".to_string()],
                context_patterns: vec!["**".to_string()],
            },
        ];

        Self { templates }
    }

    /// Получить все шаблоны
    pub fn get_all_templates(&self) -> Vec<FileTemplate> {
        self.templates.clone()
    }

    /// Получить шаблон по ID
    pub fn get_template_by_id(&self, id: &str) -> Option<FileTemplate> {
        self.templates.iter().find(|t| t.id == id).cloned()
    }

    /// Получить контекстные шаблоны для заданного пути
    /// Фильтрует шаблоны на основе context_patterns
    pub fn get_contextual_templates(&self, path: &str) -> Vec<FileTemplate> {
        self.templates
            .iter()
            .filter(|template| {
                template.context_patterns.iter().any(|pattern| {
                    if pattern == "**" {
                        return true;
                    }
                    
                    // Improved wildcard matching
                    let pattern_parts: Vec<&str> = pattern.split("**").collect();
                    let mut current_idx = 0;
                    
                    for part in pattern_parts {
                        if part.is_empty() { continue; }
                        
                        if let Some(idx) = path[current_idx..].find(part) {
                            current_idx += idx + part.len();
                        } else {
                            return false;
                        }
                    }
                    return true;
                })
            })
            .cloned()
            .collect()
    }

    /// Предложить расширение файла на основе файлов в директории
    /// Анализирует расширения и возвращает наиболее частое
    pub fn suggest_extension(&self, files: &[String]) -> Option<String> {
        if files.is_empty() {
            return None;
        }

        let mut extension_counts: HashMap<String, usize> = HashMap::new();

        for file in files {
            if let Some(ext_pos) = file.rfind('.') {
                let ext = &file[ext_pos..];
                // Игнорировать скрытые файлы без расширения (.gitignore, .env)
                if ext.len() > 1 {
                    *extension_counts.entry(ext.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Найти расширение с максимальным количеством
        extension_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ext, _)| ext)
    }

    /// Найти шаблон по расширению файла
    pub fn find_template_by_extension(&self, extension: &str) -> Option<FileTemplate> {
        self.templates
            .iter()
            .find(|template| {
                template.file_extensions.iter().any(|ext| ext == extension)
            })
            .cloned()
    }

    /// Получить шаблоны по категории
    pub fn get_templates_by_category(&self, category: &str) -> Vec<FileTemplate> {
        self.templates
            .iter()
            .filter(|t| t.category == category)
            .cloned()
            .collect()
    }

    /// Получить список всех категорий
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .templates
            .iter()
            .map(|t| t.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_registry_creation() {
        let registry = TemplateRegistry::new();
        assert!(!registry.get_all_templates().is_empty());
    }

    #[test]
    fn test_get_template_by_id() {
        let registry = TemplateRegistry::new();
        let template = registry.get_template_by_id("gitignore");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, ".gitignore");
    }

    #[test]
    fn test_suggest_extension() {
        let registry = TemplateRegistry::new();
        let files = vec![
            "file1.vue".to_string(),
            "file2.vue".to_string(),
            "file3.vue".to_string(),
            "README.md".to_string(),
        ];
        let suggestion = registry.suggest_extension(&files);
        assert_eq!(suggestion, Some(".vue".to_string()));
    }

    #[test]
    fn test_find_template_by_extension() {
        let registry = TemplateRegistry::new();
        let template = registry.find_template_by_extension(".vue");
        assert!(template.is_some());
        assert_eq!(template.unwrap().id, "vue_component");
    }

    #[test]
    fn test_contextual_templates() {
        let registry = TemplateRegistry::new();
        let templates = registry.get_contextual_templates("/project/src/components/");
        // Vue component должен быть в списке
        assert!(templates.iter().any(|t| t.id == "vue_component"));
    }
}
