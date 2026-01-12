import type { FileItem } from '../types';
import { useFileSystem } from './useFileSystem';
import { useNotifications } from './useNotifications';
import { useDialogs } from './useDialogs';

/**
 * Composable для операций с архивами (создание, извлечение)
 */
export function useArchiveOperations() {
  const { extractArchive, createArchive } = useFileSystem();
  const { success, error } = useNotifications();
  const { showInput, closeInput } = useDialogs();

  /**
   * Извлечь архив в текущую директорию
   */
  const extractHere = async (
    item: FileItem,
    currentDir: string,
    onRefresh?: () => Promise<void>
  ) => {
    try {
      const destination = currentDir.endsWith('/') ? currentDir : currentDir + '/';
      await extractArchive(item.path, destination);
      success('Extracted', `Extracted ${item.name}`);

      if (onRefresh) {
        await onRefresh();
      }
    } catch (err) {
      error('Extraction failed', err instanceof Error ? err.message : String(err));
    }
  };

  /**
   * Извлечь архив в отдельную папку с именем архива
   */
  const extractToFolder = async (
    item: FileItem,
    currentDir: string,
    onRefresh?: () => Promise<void>
  ) => {
    try {
      // Extract to a folder with the same name as the archive
      const folderName = item.name.replace(/\.(zip|tar|gz|tgz)$/i, '');
      const destination = currentDir.endsWith('/')
        ? `${currentDir}${folderName}`
        : `${currentDir}/${folderName}`;

      await extractArchive(item.path, destination);
      success('Extracted', `Extracted ${item.name} to ${folderName}`);

      if (onRefresh) {
        await onRefresh();
      }
    } catch (err) {
      error('Extraction failed', err instanceof Error ? err.message : String(err));
    }
  };

  /**
   * Создать архив из выбранных файлов
   */
  const compress = async (
    selectedItems: FileItem[],
    format: 'zip' | 'tar' | 'tar.gz',
    currentDir: string,
    onRefresh?: () => Promise<void>
  ) => {
    if (selectedItems.length === 0) return;

    // Determine archive name
    let archiveName = 'archive';
    if (selectedItems.length === 1) {
      archiveName = selectedItems[0].name;
    } else {
      // Use parent folder name if multiple items
      const parentName = currentDir.split('/').pop() || 'archive';
      archiveName = parentName;
    }

    // Prompt for name
    showInput(
      'Create Archive',
      'Enter archive name:',
      async (name: string) => {
        if (!name) return closeInput();

        let filename = name;
        if (!filename.endsWith(`.${format}`)) {
          filename += `.${format}`;
        }

        const destinationPath = currentDir.endsWith('/')
          ? `${currentDir}${filename}`
          : `${currentDir}/${filename}`;

        const sourcePaths = selectedItems.map((i) => i.path);

        try {
          await createArchive(sourcePaths, destinationPath);
          success('Archive Created', `Created ${filename}`);
          closeInput();

          if (onRefresh) {
            await onRefresh();
          }
        } catch (err) {
          error('Compression failed', err instanceof Error ? err.message : String(err));
        }
      },
      archiveName
    );
  };

  return {
    extractHere,
    extractToFolder,
    compress,
  };
}
