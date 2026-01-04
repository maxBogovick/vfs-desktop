import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { useVault } from '../composables/useVault'

/**
 * Secure wrapper around Tauri invoke that intercepts vault-related errors
 *
 * If the backend returns a "LOCKED" error, it automatically triggers
 * the vault overlay to prompt for password.
 */
export async function secureInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args)
  } catch (error: any) {
    // Check if the error is vault-related
    if (typeof error === 'string') {
      // Parse error message for vault status
      const errorLower = error.toLowerCase()

      if (errorLower.includes('locked') || errorLower.includes('vault is locked')) {
        const vault = useVault()
        vault.forceLock()
        throw new Error('Vault is locked. Please unlock to continue.')
      }

      if (errorLower.includes('not initialized') || errorLower.includes('vault is not initialized')) {
        const vault = useVault()
        vault.forceLock()
        throw new Error('Vault not initialized. Please set up your vault.')
      }
    }

    // Re-throw other errors
    throw error
  }
}

/**
 * Regular invoke without vault interception
 * Use this for vault commands themselves to avoid circular logic
 */
export const invoke = tauriInvoke
