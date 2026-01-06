import { ref, computed } from 'vue';
import type { FileItem } from '../types';

type ClipboardOperation = 'copy' | 'cut' | null;

// Module-level shared clipboard state (singleton for cross-panel copy/paste)
const clipboardItems = ref<FileItem[]>([]);
const operation = ref<ClipboardOperation>(null);
const sourceFilesystem = ref<string | undefined>(undefined);

export function useClipboard() {
  const hasClipboardItems = computed(() => clipboardItems.value.length > 0);
  const isCopyOperation = computed(() => operation.value === 'copy');
  const isCutOperation = computed(() => operation.value === 'cut');

  // Copy items to clipboard
  const copy = (items: FileItem[], filesystem?: string) => {
    if (items.length === 0) return;

    clipboardItems.value = [...items];
    operation.value = 'copy';
    sourceFilesystem.value = filesystem;

    console.log(`[Clipboard] Copied ${items.length} item(s):`, items.map(i => i.name));
  };

  // Cut items to clipboard
  const cut = (items: FileItem[], filesystem?: string) => {
    if (items.length === 0) return;

    clipboardItems.value = [...items];
    operation.value = 'cut';
    sourceFilesystem.value = filesystem;

    console.log(`[Clipboard] Cut ${items.length} item(s):`, items.map(i => i.name));
  };

  // Paste items from clipboard
  const paste = async (
    destinationPath: string,
    onCopy: (sources: string[], destination: string, sourceFs?: string) => Promise<void>,
    onMove: (sources: string[], destination: string, sourceFs?: string) => Promise<void>
  ): Promise<void> => {
    if (!hasClipboardItems.value) {
      console.warn('[Clipboard] Paste failed: Clipboard is empty');
      throw new Error('Clipboard is empty');
    }

    const sources = clipboardItems.value.map(item => item.path);
    console.log(`[Clipboard] Pasting ${sources.length} item(s) to:`, destinationPath);
    console.log(`[Clipboard] Operation:`, operation.value);

    try {
      if (operation.value === 'copy') {
        await onCopy(sources, destinationPath, sourceFilesystem.value);
        console.log(`[Clipboard] ✅ Pasted ${sources.length} item(s) (copied)`);
      } else if (operation.value === 'cut') {
        await onMove(sources, destinationPath, sourceFilesystem.value);
        console.log(`[Clipboard] ✅ Pasted ${sources.length} item(s) (moved)`);
        // Clear clipboard after cut operation
        clear();
      }
    } catch (error) {
      console.error('[Clipboard] ❌ Paste error:', error);
      throw new Error(`Failed to paste: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  // Clear clipboard
  const clear = () => {
    clipboardItems.value = [];
    operation.value = null;
    sourceFilesystem.value = undefined;
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
    sourceFilesystem,
  };
}
