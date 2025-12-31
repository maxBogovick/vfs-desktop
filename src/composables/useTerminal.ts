import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { CommandHistoryEntry, CommandResult } from '../types';

const MAX_HISTORY = 100;

// Module-level state (shared across all components)
const isVisible = ref(false);
const terminalHeight = ref(200);
const history = ref<CommandHistoryEntry[]>([]);
const currentCommand = ref('');
const isExecuting = ref(false);

export function useTerminal() {
  const executeCommand = async (
    command: string,
    workingDir: string
  ): Promise<void> => {
    if (!command.trim() || isExecuting.value) return;

    isExecuting.value = true;

    try {
      const result = await invoke<CommandResult>('execute_command', {
        command,
        workingDir,
      });

      const entry: CommandHistoryEntry = {
        id: `${Date.now()}-${Math.random()}`,
        command,
        workingDir,
        timestamp: Date.now(),
        stdout: result.stdout,
        stderr: result.stderr,
        exitCode: result.exit_code,
        success: result.success,
      };

      // Add to history (prepend, limit to MAX_HISTORY)
      history.value.unshift(entry);
      if (history.value.length > MAX_HISTORY) {
        history.value = history.value.slice(0, MAX_HISTORY);
      }
    } catch (error) {
      const entry: CommandHistoryEntry = {
        id: `${Date.now()}-${Math.random()}`,
        command,
        workingDir,
        timestamp: Date.now(),
        stdout: '',
        stderr: String(error),
        exitCode: -1,
        success: false,
      };
      history.value.unshift(entry);
    } finally {
      isExecuting.value = false;
      currentCommand.value = '';
    }
  };

  const toggleTerminal = () => {
    isVisible.value = !isVisible.value;
  };

  const clearHistory = () => {
    history.value = [];
  };

  return {
    isVisible,
    terminalHeight,
    history,
    currentCommand,
    isExecuting,
    executeCommand,
    toggleTerminal,
    clearHistory,
  };
}
