import { ref } from 'vue';
import { useFileSystem } from './useFileSystem';

interface CacheEntry {
  path: string;
  content: string;
  timestamp: number;
  size: number;
}

// Singleton cache
const cache = ref<Map<string, CacheEntry>>(new Map());
const MAX_CACHE_SIZE = 10; // Максимум 10 файлов в кеше
const MAX_CACHE_MEMORY = 50 * 1024 * 1024; // 50MB максимум

export function useFileContentCache() {
  const { readFileContent: readFromFS } = useFileSystem();

  // Получить размер кеша в байтах
  const getCacheSize = (): number => {
    let totalSize = 0;
    cache.value.forEach(entry => {
      totalSize += entry.size;
    });
    return totalSize;
  };

  // Очистить самые старые записи для освобождения места
  const evictOldest = () => {
    if (cache.value.size === 0) return;

    // Сортируем по timestamp и удаляем самую старую
    const entries = Array.from(cache.value.entries());
    entries.sort((a, b) => a[1].timestamp - b[1].timestamp);

    const [oldestKey] = entries[0];
    cache.value.delete(oldestKey);
  };

  // Очистить кеш если превышен лимит памяти
  const enforceMemoryLimit = () => {
    while (getCacheSize() > MAX_CACHE_MEMORY && cache.value.size > 0) {
      evictOldest();
    }
  };

  // Очистить кеш если превышен лимит количества
  const enforceSizeLimit = () => {
    while (cache.value.size > MAX_CACHE_SIZE) {
      evictOldest();
    }
  };

  // Получить файл из кеша или загрузить
  const getFileContent = async (path: string, maxSize?: number, panelFs?: string): Promise<string> => {
    const fsKey = panelFs || 'real';
    const cacheKey = `${fsKey}:${path}`;

    // Проверяем кеш
    const cached = cache.value.get(cacheKey);
    if (cached) {
      // Обновляем timestamp для LRU
      cached.timestamp = Date.now();
      return cached.content;
    }

    // Загружаем файл
    const content = await readFromFS(path, maxSize, panelFs);

    // Добавляем в кеш
    const entry: CacheEntry = {
      path: cacheKey,
      content,
      timestamp: Date.now(),
      size: content.length * 2, // Примерная оценка размера в памяти (UTF-16)
    };

    cache.value.set(cacheKey, entry);

    // Применяем лимиты
    enforceSizeLimit();
    enforceMemoryLimit();

    return content;
  };

  // Очистить весь кеш
  const clearCache = () => {
    cache.value.clear();
  };

  // Удалить конкретный файл из кеша
  const invalidate = (path: string, panelFs?: string) => {
    const fsKey = panelFs || 'real';
    const cacheKey = `${fsKey}:${path}`;
    cache.value.delete(cacheKey);
  };

  // Получить статистику кеша
  const getCacheStats = () => {
    return {
      entries: cache.value.size,
      totalSize: getCacheSize(),
      maxSize: MAX_CACHE_MEMORY,
      maxEntries: MAX_CACHE_SIZE,
    };
  };

  return {
    getFileContent,
    clearCache,
    invalidate,
    getCacheStats,
  };
}
