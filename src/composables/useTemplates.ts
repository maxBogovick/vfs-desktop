import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FileTemplate } from '../types'

// Глобальный кеш шаблонов
const templatesCache = ref<FileTemplate[]>([])
const isLoading = ref(false)
const selectedTemplate = ref<FileTemplate | null>(null)

export function useTemplates() {
  /**
   * Загрузить все шаблоны
   */
  const loadTemplates = async (): Promise<FileTemplate[]> => {
    if (templatesCache.value.length > 0) {
      return templatesCache.value
    }

    isLoading.value = true
    try {
      const templates = await invoke<FileTemplate[]>('get_file_templates')
      templatesCache.value = templates
      return templates
    } catch (error) {
      console.error('Failed to load templates:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Получить все шаблоны
   */
  const templates = computed(() => templatesCache.value)

  /**
   * Получить контекстные шаблоны для текущего пути
   */
  const getContextualTemplates = (currentPath: string): FileTemplate[] => {
    return templatesCache.value.filter(template => {
      return template.contextPatterns.some(pattern => {
        if (pattern === '**') return true

        // Простая проверка паттерна
        const parts = pattern.split('**')
        if (parts.length === 2) {
          const prefix = parts[0].replace(/^\/|\/$/g, '')
          const suffix = parts[1].replace(/^\/|\/$/g, '')

          if (prefix && !currentPath.includes(prefix)) return false
          if (suffix && !currentPath.includes(suffix)) return false

          return true
        }

        return currentPath.includes(pattern)
      })
    })
  }

  /**
   * Получить содержимое шаблона по ID
   */
  const getTemplateContent = async (templateId: string): Promise<string> => {
    try {
      return await invoke<string>('get_template_content', { templateId })
    } catch (error) {
      console.error('Failed to get template content:', error)
      throw error
    }
  }

  /**
   * Предложить расширение файла на основе файлов в директории
   */
  const suggestExtension = async (path: string): Promise<string | null> => {
    try {
      return await invoke<string | null>('suggest_file_extension', { path })
    } catch (error) {
      console.error('Failed to suggest extension:', error)
      return null
    }
  }

  /**
   * Найти шаблон по расширению файла
   */
  const findTemplateByExtension = (extension: string): FileTemplate | null => {
    return templatesCache.value.find(template =>
      template.fileExtensions.some(ext => ext === extension)
    ) || null
  }

  /**
   * Найти шаблон по имени файла
   */
  const findTemplateByFilename = (filename: string): FileTemplate | null => {
    // Попробуем найти точное совпадение по полному имени
    const exactMatch = templatesCache.value.find(template =>
      template.fileExtensions.some(ext => filename === ext)
    )

    if (exactMatch) return exactMatch

    // Иначе ищем по расширению
    const lastDot = filename.lastIndexOf('.')
    if (lastDot === -1) return null

    const extension = filename.substring(lastDot)
    return findTemplateByExtension(extension)
  }

  /**
   * Получить шаблоны по категории
   */
  const getTemplatesByCategory = (category: string): FileTemplate[] => {
    return templatesCache.value.filter(t => t.category === category)
  }

  /**
   * Получить список всех категорий
   */
  const categories = computed(() => {
    const cats = new Set(templatesCache.value.map(t => t.category))
    return Array.from(cats).sort()
  })

  /**
   * Выбрать шаблон
   */
  const selectTemplate = (template: FileTemplate | null) => {
    selectedTemplate.value = template
  }

  /**
   * Очистить кеш (для отладки)
   */
  const clearCache = () => {
    templatesCache.value = []
  }

  return {
    templates,
    isLoading,
    selectedTemplate,
    categories,
    loadTemplates,
    getContextualTemplates,
    getTemplateContent,
    suggestExtension,
    findTemplateByExtension,
    findTemplateByFilename,
    getTemplatesByCategory,
    selectTemplate,
    clearCache,
  }
}
