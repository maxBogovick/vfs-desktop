import { reactive, watch } from 'vue';
import type { FileItem } from '../types';
import { type FileColorConfig, DEFAULT_COLOR_CONFIG } from '../types/fileColoring';
import { useBookmarks } from './useBookmarks';

// Global state
const config = reactive<FileColorConfig>(JSON.parse(JSON.stringify(DEFAULT_COLOR_CONFIG)));

export function useFileColoring() {
  const { isBookmarked } = useBookmarks();

  const getFileColor = (item: FileItem): string | undefined => {
    if (!config.enabled) return undefined;

    // 1. Special: Favorites (Bookmarks)
    if (isBookmarked(item.path)) {
        return config.special.favorite;
    }

    // 1.1 Special: Recent (< 24h)
    if (item.modifiedTimestamp) {
        const diff = Date.now() - item.modifiedTimestamp;
        // 24 hours
        if (diff < 24 * 60 * 60 * 1000) {
            return config.special.recent;
        }
    }

    // 1.2 Special: Hidden
    if (item.name.startsWith('.')) {
        return config.special.hidden;
    }

    // 2. Custom Rules
    for (const rule of config.customRules) {
        if (rule.enabled && rule.pattern) {
            try {
                const regex = new RegExp(rule.pattern);
                if (regex.test(item.name) || regex.test(item.path)) {
                    return rule.color;
                }
            } catch (e) {
                // Ignore invalid regex
            }
        }
    }

    // 3. Base Types (Folder/Drive/System) - Priority over extensions usually?
    // Actually, usually folders are specific.
    if (item.type === 'folder') return config.base.folder;
    if (item.type === 'drive') return config.base.drive;
    if (item.type === 'system') return config.base.system;

    // 4. Extensions (Files only)
    if (item.type !== 'folder' && item.type !== 'drive' && item.type !== 'system') {
        const ext = item.name.split('.').pop()?.toLowerCase();
        if (ext && config.extensions[ext]) {
            return config.extensions[ext];
        }
    }

    // 5. File Types (Semantic)
    // Map item.type to config types
    if (config.fileTypes[item.type as keyof typeof config.fileTypes]) {
        return config.fileTypes[item.type as keyof typeof config.fileTypes];
    }
    
    // Default file
    return config.base.file;
  };

  const getFileStyle = (item: FileItem) => {
      const color = getFileColor(item);
      return color ? { color: color } : {};
  };

  // Actions to update config
  const updateConfig = (newConfig: Partial<FileColorConfig>) => {
      Object.assign(config, newConfig);
      saveConfig();
  };
  
  const resetConfig = () => {
      Object.assign(config, JSON.parse(JSON.stringify(DEFAULT_COLOR_CONFIG)));
      saveConfig();
  };

  const saveConfig = () => {
      localStorage.setItem('vfdir_color_config', JSON.stringify(config));
  };

  // Load from local storage on init
  const loadConfig = () => {
      const saved = localStorage.getItem('vfdir_color_config');
      if (saved) {
          try {
              const parsed = JSON.parse(saved);
              // Deep merge logic might be better but simple assign for now
              Object.assign(config, parsed);
          } catch (e) {
              console.error('Failed to parse saved color config', e);
          }
      }
  };

  return {
      config,
      getFileColor,
      getFileStyle,
      updateConfig,
      resetConfig,
      loadConfig
  };
}
