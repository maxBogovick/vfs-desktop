import { ref, computed } from 'vue';
import type { FileItem } from '../types';

type ClipboardOperation = 'copy' | 'cut' | null;

export function useClipboard() {
  const clipboardItems = ref<FileItem[]>([]);
  const operation = ref<ClipboardOperation>(null);

  const hasClipboardItems = computed(() => clipboardItems.value.length > 0);
  const isCopyOperation = computed(() => operation.value === 'copy');
  const isCutOperation = computed(() => operation.value === 'cut');

  // Copy items to clipboard
  const copy = (items: FileItem[]) => {
    if (items.length === 0) return;

    clipboardItems.value = [...items];
    operation.value = 'copy';

    console.log(`Copied ${items.length} item(s) to clipboard`);
  };

  // Cut items to clipboard
  const cut = (items: FileItem[]) => {
    if (items.length === 0) return;

    clipboardItems.value = [...items];
    operation.value = 'cut';

    console.log(`Cut ${items.length} item(s) to clipboard`);
  };

  // Paste items from clipboard
  const paste = async (
    destinationPath: string,
    onCopy: (sources: string[], destination: string) => Promise<void>,
    onMove: (sources: string[], destination: string) => Promise<void>
  ): Promise<void> => {
    if (!hasClipboardItems.value) {
      throw new Error('Clipboard is empty');
    }

    const sources = clipboardItems.value.map(item => item.path);

    try {
      if (operation.value === 'copy') {
        await onCopy(sources, destinationPath);
        console.log(`Pasted ${sources.length} item(s) (copied)`);
      } else if (operation.value === 'cut') {
        await onMove(sources, destinationPath);
        console.log(`Pasted ${sources.length} item(s) (moved)`);
        // Clear clipboard after cut operation
        clear();
      }
    } catch (error) {
      console.error(error);
      throw new Error(`Failed to paste: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  // Clear clipboard
  const clear = () => {
    clipboardItems.value = [];
    operation.value = null;
  };

  // Check if a specific item is in clipboard
  const isInClipboard = (itemPath: string): boolean => {
    return clipboardItems.value.some(item => item.path === itemPath);
  };

  return {
    clipboardItems,
    operation,
    hasClipboardItems,
    isCopyOperation,
    isCutOperation,
    copy,
    cut,
    paste,
    clear,
    isInClipboard,
  };
}
