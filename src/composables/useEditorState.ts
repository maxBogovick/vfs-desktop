import { ref } from 'vue';
import type { FileItem } from '../types';
import { useFileSystem } from './useFileSystem';
import { useNotifications } from './useNotifications';
import { useFileContentCache } from './useFileContentCache';

/**
 * Composable для управления состоянием текстового редактора
 */
export function useEditorState() {
  const showTextEditor = ref(false);
  const editorFile = ref<FileItem | null>(null);
  const editorFileFs = ref<string | undefined>(undefined);

  const { writeFileContent } = useFileSystem();
  const { success, error } = useNotifications();
  const { invalidate } = useFileContentCache();

  const openEditor = (file: FileItem, panelFs?: string) => {
    editorFile.value = file;
    editorFileFs.value = panelFs;
    showTextEditor.value = true;
  };

  const closeEditor = () => {
    showTextEditor.value = false;
    editorFile.value = null;
    editorFileFs.value = undefined;
  };

  const saveFile = async (content: string, onRefresh?: () => Promise<void>) => {
    if (!editorFile.value) return;

    try {
      await writeFileContent(editorFile.value.path, content, editorFileFs.value);

      // Invalidate cache for this file so next time it loads fresh content
      invalidate(editorFile.value.path, editorFileFs.value);

      success('File saved', `Saved: ${editorFile.value.name}`);

      closeEditor();

      // Refresh directory to show updated file
      if (onRefresh) {
        await onRefresh();
      }
    } catch (err) {
      error('Failed to save file', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  return {
    showTextEditor,
    editorFile,
    editorFileFs,
    openEditor,
    closeEditor,
    saveFile,
  };
}
