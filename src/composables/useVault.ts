import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { vaultCreateStegoContainer, vaultOpenStegoContainer } from '../utils/api'

export enum VaultStatus {
  UNINITIALIZED = 'UNINITIALIZED',
  LOCKED = 'LOCKED',
  UNLOCKED = 'UNLOCKED',
  CHECKING = 'CHECKING',
  DISABLED = 'DISABLED' // When using Real FS backend
}

// Global vault state (singleton pattern)
const status = ref<VaultStatus>(VaultStatus.CHECKING)
const isVaultOverlayVisible = ref(false)
const lastError = ref<string | null>(null)
const isEnabled = ref(true) // Vault enabled only for Virtual FS
const forceShowOverlay = ref(false) // Allow showing overlay even if disabled globally

export function useVault() {
  const isUnlocked = computed(() => status.value === VaultStatus.UNLOCKED)
  const isLocked = computed(() => status.value === VaultStatus.LOCKED)
  const isUninitialized = computed(() => status.value === VaultStatus.UNINITIALIZED)
  const isChecking = computed(() => status.value === VaultStatus.CHECKING)

  /**
   * Check current vault status
   */
  async function checkStatus(): Promise<void> {
    try {
      status.value = VaultStatus.CHECKING

      // First check if vault is enabled (Virtual FS backend)
      const enabled = await invoke<boolean>('vault_is_enabled')
      isEnabled.value = enabled

      if (!enabled && !forceShowOverlay.value) {
        // Vault disabled for Real FS
        status.value = VaultStatus.DISABLED
        isVaultOverlayVisible.value = false
        return
      }

      // Get vault status
      const vaultStatus = await invoke<string>('vault_get_status')
      status.value = vaultStatus as VaultStatus

      // Show overlay if not unlocked (and not disabled)
      if (status.value !== VaultStatus.UNLOCKED && status.value !== VaultStatus.DISABLED) {
        isVaultOverlayVisible.value = true
      } else {
        isVaultOverlayVisible.value = false
      }
    } catch (error) {
      console.error('Failed to check vault status:', error)
      lastError.value = String(error)
      status.value = VaultStatus.LOCKED
      isVaultOverlayVisible.value = true
    }
  }

  /**
   * Force show vault overlay (used for dual panel mixed mode)
   */
  function openVault(): void {
    forceShowOverlay.value = true
    checkStatus()
  }

  /**
   * Initialize vault with password (first-time setup)
   */
  async function initialize(password: string): Promise<void> {
    try {
      lastError.value = null
      await invoke('vault_initialize', { password })
      await checkStatus()
    } catch (error) {
      console.error('Failed to initialize vault:', error)
      lastError.value = String(error)
      throw error
    }
  }

  /**
   * Unlock vault with password
   */
  async function unlock(password: string): Promise<void> {
    try {
      lastError.value = null
      await invoke('vault_unlock', { password })
      await checkStatus()
    } catch (error) {
      console.error('Failed to unlock vault:', error)
      lastError.value = String(error)
      throw error
    }
  }

  /**
   * Lock vault (clear keys from memory)
   */
  async function lock(): Promise<void> {
    try {
      lastError.value = null
      await invoke('vault_lock')
      await checkStatus()
    } catch (error) {
      console.error('Failed to lock vault:', error)
      lastError.value = String(error)
      throw error
    }
  }

  /**
   * Force lock (used when backend returns LOCKED error)
   */
  function forceLock(): void {
    status.value = VaultStatus.LOCKED
    isVaultOverlayVisible.value = true
  }

  async function createStegoContainer(hostPath: string, outputPath: string, password: string): Promise<void> {
    await vaultCreateStegoContainer(hostPath, outputPath, password)
  }

  async function openStegoContainer(containerPath: string, password: string): Promise<void> {
    await vaultOpenStegoContainer(containerPath, password)
    await checkStatus()
  }

  return {
    // State
    status: computed(() => status.value),
    isVaultOverlayVisible: computed(() => isVaultOverlayVisible.value),
    lastError: computed(() => lastError.value),
    isEnabled: computed(() => isEnabled.value),

    // Computed
    isUnlocked,
    isLocked,
    isUninitialized,
    isChecking,

    // Actions
    checkStatus,
    openVault,
    initialize,
    unlock,
    lock,
    forceLock,
    createStegoContainer,
    openStegoContainer,
  }
}