import { ref } from 'vue';
import type { FileConflict, ConflictResolution, ConflictAction } from '../types';

const currentConflict = ref<FileConflict | null>(null);
const isConflictDialogOpen = ref(false);
const resolveCallback = ref<((resolution: ConflictResolution) => void) | null>(null);
const rejectCallback = ref<(() => void) | null>(null);

// Saved resolution for "apply to all" feature
const savedResolution = ref<ConflictResolution | null>(null);

// Counter for generating unique names in current operation
let renameCounter = 0;

export function useConflictResolution() {
  // Generate unique name for file using counter
  const generateUniqueName = (fileName: string): string => {
    const dotIndex = fileName.lastIndexOf('.');
    const baseName = dotIndex > 0 ? fileName.substring(0, dotIndex) : fileName;
    const extension = dotIndex > 0 ? fileName.substring(dotIndex) : '';

    renameCounter++;
    if (renameCounter === 1) {
      return `${baseName} (copy)${extension}`;
    } else {
      return `${baseName} (copy ${renameCounter})${extension}`;
    }
  };

  // Show conflict dialog and wait for user resolution
  const requestConflictResolution = (
    conflict: FileConflict
  ): Promise<ConflictResolution> => {
    return new Promise((resolve, reject) => {
      // Check if we have a saved "apply to all" resolution
      if (savedResolution.value) {
        // For rename action, always generate a new unique name
        if (savedResolution.value.action === 'rename') {
          const newName = generateUniqueName(conflict.sourceFile.name);
          resolve({
            action: 'rename',
            applyToAll: true,
            newName: newName,
          });
        } else {
          resolve(savedResolution.value);
        }
        return;
      }

      currentConflict.value = conflict;
      isConflictDialogOpen.value = true;

      resolveCallback.value = (resolution: ConflictResolution) => {
        // Save resolution if "apply to all" is checked
        // For rename action, don't save the specific newName
        if (resolution.applyToAll) {
          if (resolution.action === 'rename') {
            savedResolution.value = {
              action: 'rename',
              applyToAll: true,
              // Don't save newName - it will be generated for each file
            };
          } else {
            savedResolution.value = resolution;
          }
        }

        resolve(resolution);
        closeDialog();
      };

      rejectCallback.value = () => {
        reject(new Error('User cancelled conflict resolution'));
        closeDialog();
      };
    });
  };

  // Handle resolution from dialog
  const handleResolution = (resolution: ConflictResolution) => {
    if (resolveCallback.value) {
      resolveCallback.value(resolution);
    }
  };

  // Handle cancel from dialog
  const handleCancel = () => {
    if (rejectCallback.value) {
      rejectCallback.value();
    }
  };

  // Close dialog and reset state
  const closeDialog = () => {
    isConflictDialogOpen.value = false;
    currentConflict.value = null;
    resolveCallback.value = null;
    rejectCallback.value = null;
  };

  // Reset saved resolution (call this when operation is complete)
  const resetSavedResolution = () => {
    savedResolution.value = null;
    renameCounter = 0; // Reset counter for next operation
  };

  // Check if file exists and get metadata
  const checkConflict = async (
    sourcePath: string,
    destinationDir: string
  ): Promise<FileConflict | null> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{
        source_path: string;
        destination_path: string;
        source_file: {
          name: string;
          size: number;
          modified: number;
        };
        destination_file: {
          name: string;
          size: number;
          modified: number;
        };
      } | null>('check_file_conflict', {
        sourcePath,
        destinationDir,
      });

      if (!result) {
        return null; // No conflict
      }

      // Convert backend format to frontend format
      return {
        sourcePath: result.source_path,
        destinationPath: result.destination_path,
        sourceFile: {
          name: result.source_file.name,
          size: result.source_file.size,
          modified: result.source_file.modified,
        },
        destinationFile: {
          name: result.destination_file.name,
          size: result.destination_file.size,
          modified: result.destination_file.modified,
        },
      };
    } catch (error) {
      console.error('Failed to check file conflict:', error);
      return null;
    }
  };

  return {
    // State
    currentConflict,
    isConflictDialogOpen,

    // Methods
    requestConflictResolution,
    handleResolution,
    handleCancel,
    resetSavedResolution,
    checkConflict,
  };
}
