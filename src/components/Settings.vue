<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types'
import { useTheme } from '../composables/useTheme'
import { useVault } from '../composables/useVault'
import SettingsFileColors from './SettingsFileColors.vue'

const props = withDefaults(defineProps<{
  initialTab?: 'general' | 'colors'
}>(), {
  initialTab: 'general'
})

const emit = defineEmits<{
  close: []
}>()

const currentTab = ref<'general' | 'colors'>(props.initialTab)
const config = ref<AppConfig>({
  filesystem_backend: 'real',
  show_hidden_files: false,
  default_view_mode: 'grid',
  theme: 'luna'
})

const saving = ref(false)
const message = ref<{ text: string; type: 'success' | 'error' } | null>(null)
const vault = useVault()
const vaultActionInProgress = ref(false)
const showResetConfirm = ref(false)
const resetConfirmStep = ref(1)

// Vault directory management
const vaultDirectory = ref<string>('')
const defaultVaultDirectory = ref<string>('')
const isCustomVaultPath = ref(false)
const showMigrationConfirm = ref(false)
const pendingVaultPath = ref<string | null>(null)

// Загрузить конфигурацию при монтировании
onMounted(async () => {
  try {
    const loadedConfig = await invoke<AppConfig>('get_config')
    config.value = loadedConfig

    // Load vault directory info if using virtual backend
    if (config.value.filesystem_backend === 'virtual') {
      await loadVaultDirectoryInfo()
    }
  } catch (error) {
    console.error('Failed to load config:', error)
    showMessage(`Failed to load settings: ${error}`, 'error')
  }
})

// Сохранить настройки
const saveSettings = async () => {
  saving.value = true
  message.value = null

  try {
    await invoke('update_config', { newConfig: config.value })

    // Apply theme immediately after saving
    const { setTheme } = useTheme()
    await setTheme(config.value.theme as any)

    // Re-check vault status (important when switching between Real/Virtual FS)
    await vault.checkStatus()

    // Notify toolbar to update FS badge
    window.dispatchEvent(new Event('fs-config-changed'))

    showMessage('Settings saved successfully!', 'success')

    // Закрыть через небольшую задержку
    setTimeout(() => {
      emit('close')
    }, 1000)
  } catch (error) {
    console.error('Failed to save config:', error)
    showMessage(`Failed to save settings: ${error}`, 'error')
  } finally {
    saving.value = false
  }
}

// Показать сообщение
const showMessage = (text: string, type: 'success' | 'error') => {
  message.value = { text, type }
  setTimeout(() => {
    message.value = null
  }, 3000)
}

// Vault management functions
const lockVault = async () => {
  try {
    vaultActionInProgress.value = true
    await vault.lock()
    showMessage('Vault locked successfully. Reloading...', 'success')
    // Reload page to ensure all state is cleared and vault overlay shows
    setTimeout(() => {
      window.location.reload()
    }, 1000)
  } catch (error) {
    console.error('Failed to lock vault:', error)
    showMessage(`Failed to lock vault: ${error}`, 'error')
    vaultActionInProgress.value = false
  }
}

const unlockVault = () => {
  // This will show the unlock overlay
  vault.forceLock()
  emit('close')
}

const resetVault = () => {
  // Show custom confirmation dialog
  resetConfirmStep.value = 1
  showResetConfirm.value = true
}

const confirmReset = () => {
  if (resetConfirmStep.value === 1) {
    // First confirmation passed, show second
    resetConfirmStep.value = 2
  } else if (resetConfirmStep.value === 2) {
    // Both confirmations passed, execute reset
    executeReset()
  }
}

const cancelReset = () => {
  showResetConfirm.value = false
  resetConfirmStep.value = 1
}

const executeReset = async () => {
  showResetConfirm.value = false

  try {
    vaultActionInProgress.value = true
    await invoke('vault_reset')
    console.log('Vault reset reset successfully!')
    showMessage('Vault reset successfully! Reloading application...', 'success')

    // Close settings and reload after a delay
    setTimeout(() => {
      window.location.reload()
    }, 1500)
  } catch (error) {
    console.error('Failed to reset vault:', error)
    showMessage(`Failed to reset vault: ${error}`, 'error')
  } finally {
    vaultActionInProgress.value = false
    resetConfirmStep.value = 1
  }
}

// Vault directory management functions
const loadVaultDirectoryInfo = async () => {
  try {
    const [current, defaultDir] = await Promise.all([
      invoke<string>('vault_get_current_directory'),
      invoke<string>('vault_get_default_directory')
    ])

    vaultDirectory.value = current
    defaultVaultDirectory.value = defaultDir
    isCustomVaultPath.value = current !== defaultDir
  } catch (error) {
    console.error('Failed to load vault directory info:', error)
  }
}

const selectVaultDirectory = async () => {
  try {
    const selected = await invoke<string | null>('vault_select_directory')

    if (selected) {
      // Show migration confirmation dialog
      pendingVaultPath.value = selected
      showMigrationConfirm.value = true
    }
  } catch (error) {
    showMessage(`Failed to select directory: ${error}`, 'error')
  }
}

const confirmVaultMigration = async (migrate: boolean) => {
  if (!pendingVaultPath.value) return

  try {
    vaultActionInProgress.value = true

    await invoke('vault_set_custom_directory', {
      path: pendingVaultPath.value,
      migrateData: migrate
    })

    await loadVaultDirectoryInfo()

    showMessage(
      migrate
        ? 'Vault directory changed and data migrated successfully!'
        : 'Vault directory changed. Old data remains in previous location.',
      'success'
    )

    if (migrate) {
      // Reload to reinitialize vault with new location
      setTimeout(() => window.location.reload(), 1500)
    }
  } catch (error) {
    showMessage(`Failed to change vault directory: ${error}`, 'error')
  } finally {
    vaultActionInProgress.value = false
    showMigrationConfirm.value = false
    pendingVaultPath.value = null
  }
}

const resetToDefaultVaultDirectory = async () => {
  try {
    vaultActionInProgress.value = true

    await invoke('vault_reset_to_default_directory', {
      migrateData: true
    })

    await loadVaultDirectoryInfo()
    showMessage('Vault directory reset to default and data migrated!', 'success')

    setTimeout(() => window.location.reload(), 1500)
  } catch (error) {
    showMessage(`Failed to reset vault directory: ${error}`, 'error')
  } finally {
    vaultActionInProgress.value = false
  }
}

const createStegoVault = async () => {
  try {
    // 1. Select Host File
    const hostPath = await invoke<string | null>('vault_select_file')
    if (!hostPath) return

    // 2. Select Output File
    const outputPath = await invoke<string | null>('vault_save_file_dialog')
    if (!outputPath) return

    // 3. Prompt for Password
    const password = prompt('Enter a password to encrypt the hidden vault container:')
    if (!password) return

    vaultActionInProgress.value = true
    
    // 4. Create Container
    await vault.createStegoContainer(hostPath, outputPath, password)
    
    showMessage(`Vault successfully hidden in: ${outputPath}`, 'success')
  } catch (error) {
    console.error('Steganography error:', error)
    showMessage(`Failed to create hidden vault: ${error}`, 'error')
  } finally {
    vaultActionInProgress.value = false
  }
}

// Отменить изменения
const cancel = () => {
  emit('close')
}

// Get theme description
const getThemeDescription = (theme: string): string => {
  const descriptions: Record<string, string> = {
    luna: 'Classic Windows XP blue theme',
    classic: 'Windows 95/98 gray theme',
    royale: 'Windows XP Media Center Edition theme',
    silver: 'Windows XP Silver theme',
    dark: 'Modern dark theme'
  }
  return descriptions[theme] || ''
}
</script>

<template>
  <div class="settings-overlay" @click.self="cancel">
    <div class="settings-dialog">
      <!-- Title bar -->
      <div class="title-bar">
        <span class="title">Settings</span>
        <button class="close-btn" @click="cancel">×</button>
      </div>

      <!-- Content -->
      <div class="flex border-b border-[#ACA899] bg-[#ECE9D8] px-2 pt-2 gap-1">
        <button 
          @click="currentTab = 'general'"
          :class="[
            'px-4 py-1.5 rounded-t text-sm border-t border-l border-r border-[#ACA899] mb-[-1px] z-10',
            currentTab === 'general' ? 'bg-white font-medium border-b-white pb-2' : 'bg-[#E3E3E3] hover:bg-[#F0F0F0]'
          ]"
        >
          General
        </button>
        <button 
          @click="currentTab = 'colors'"
          :class="[
            'px-4 py-1.5 rounded-t text-sm border-t border-l border-r border-[#ACA899] mb-[-1px] z-10',
            currentTab === 'colors' ? 'bg-white font-medium border-b-white pb-2' : 'bg-[#E3E3E3] hover:bg-[#F0F0F0]'
          ]"
        >
          File Colors
        </button>
      </div>

      <div class="settings-content bg-white" v-if="currentTab === 'general'">
        <!-- File System Backend -->
        <div class="setting-section">
          <h3 class="section-title">File System</h3>

          <div class="setting-item">
            <label class="setting-label">Backend:</label>
            <div class="radio-group">
              <label class="radio-option">
                <input
                  v-model="config.filesystem_backend"
                  type="radio"
                  value="real"
                  name="backend"
                />
                <span>Real File System</span>
                <span class="radio-description">Access your actual files on disk</span>
              </label>

              <label class="radio-option">
                <input
                  v-model="config.filesystem_backend"
                  type="radio"
                  value="virtual"
                  name="backend"
                />
                <span>Virtual File System</span>
                <span class="radio-description">Use in-memory virtual file system for testing</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Vault Security -->
        <div v-if="config.filesystem_backend === 'virtual'" class="setting-section">
          <h3 class="section-title">Vault Security</h3>

          <div class="setting-item">
            <label class="setting-label">Status:</label>
            <div class="flex items-center gap-3">
              <span class="px-3 py-1 rounded text-xs font-medium" :class="{
                'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400': vault.isUnlocked.value,
                'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400': !vault.isUnlocked.value
              }">
                {{ vault.isUnlocked.value ? '🔓 Unlocked' : '🔒 Locked' }}
              </span>
            </div>
          </div>

          <div class="setting-item">
            <label class="setting-label">Actions:</label>
            <div class="flex gap-2">
              <button
                v-if="vault.isUnlocked.value"
                @click="lockVault"
                :disabled="vaultActionInProgress"
                class="px-4 py-2 bg-orange-500 hover:bg-orange-600 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {{ vaultActionInProgress ? 'Locking...' : '🔒 Lock Vault' }}
              </button>

              <button
                v-else
                @click="unlockVault"
                :disabled="vaultActionInProgress"
                class="px-4 py-2 bg-green-500 hover:bg-green-600 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                🔓 Unlock Vault
              </button>
            </div>
          </div>

          <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
            Lock vault to require password authentication. Unlock to access encrypted data.
          </div>

          <!-- Vault Directory Settings -->
          <div class="setting-item mt-4">
            <label class="setting-label font-semibold">Vault Storage Location:</label>

            <div class="space-y-2">
              <div class="text-sm text-gray-700 dark:text-gray-300">
                <span class="font-medium">Current:</span>
                <code class="ml-2 px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs block mt-1">
                  {{ vaultDirectory }}
                </code>
              </div>

              <div class="text-xs text-gray-500 dark:text-gray-400">
                <span class="font-medium">Default:</span>
                <code class="ml-2">{{ defaultVaultDirectory }}</code>
              </div>
            </div>

            <div class="flex gap-2 mt-3">
              <button
                @click="selectVaultDirectory"
                :disabled="vaultActionInProgress"
                class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                📁 Change Location
              </button>

              <button
                v-if="isCustomVaultPath"
                @click="resetToDefaultVaultDirectory"
                :disabled="vaultActionInProgress"
                class="px-4 py-2 bg-gray-500 hover:bg-gray-600 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                🔄 Reset to Default
              </button>
            </div>

            <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
              Change where vault files are stored. Data can be automatically migrated to the new location.
            </div>

            <!-- Steganography Tool -->
            <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
              <label class="setting-label font-semibold">Steganography Tool:</label>
              <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">
                Hide your entire vault inside a video or image file. The file will still play/view normally.
              </div>
              <button
                @click="createStegoVault"
                :disabled="vaultActionInProgress"
                class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
              >
                🕵️ Hide Vault in File...
              </button>
            </div>
          </div>

          <!-- Danger Zone: Reset Vault -->
          <div class="setting-item mt-4 pt-4 border-t border-red-200 dark:border-red-800">
            <label class="setting-label text-red-600 dark:text-red-400 font-bold">⚠️ Danger Zone:</label>
            <button
              @click="resetVault"
              :disabled="vaultActionInProgress"
              class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              🗑️ Reset Vault (Delete All Data)
            </button>
            <p class="text-xs text-red-600 dark:text-red-400 mt-2">
              This will permanently delete all vault data, settings, and recovery configuration. This action cannot be undone!
            </p>
          </div>
        </div>

        <!-- Display Settings -->
        <div class="setting-section">
          <h3 class="section-title">Display</h3>

          <div class="setting-item">
            <label class="setting-label">
              <input
                v-model="config.show_hidden_files"
                type="checkbox"
                class="checkbox"
              />
              <span>Show hidden files</span>
            </label>
          </div>

          <div class="setting-item">
            <label class="setting-label">Default View Mode:</label>
            <select v-model="config.default_view_mode" class="select">
              <option value="grid">Grid</option>
              <option value="list">List</option>
              <option value="details">Details</option>
            </select>
          </div>

          <div class="setting-item">
            <label class="setting-label">Theme:</label>
            <select v-model="config.theme" class="select">
              <option value="luna">Luna (Default)</option>
              <option value="classic">Classic</option>
              <option value="royale">Royale</option>
              <option value="silver">Silver</option>
              <option value="dark">Dark Mode</option>
            </select>
            <p class="setting-description">
              {{ getThemeDescription(config.theme) }}
            </p>
          </div>
        </div>

        <!-- Message -->
        <div v-if="message" :class="['message', message.type]">
          {{ message.text }}
        </div>
      </div>

      <SettingsFileColors v-if="currentTab === 'colors'" />

      <!-- Buttons -->
      <div class="button-bar">
        <button
          class="btn btn-primary"
          :disabled="saving"
          @click="saveSettings"
        >
          {{ saving ? 'Saving...' : 'Save' }}
        </button>
        <button
          class="btn btn-secondary"
          :disabled="saving"
          @click="cancel"
        >
          Cancel
        </button>
      </div>
    </div>

    <!-- Reset Vault Confirmation Dialog -->
    <div v-if="showResetConfirm" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[2000]" @click.self="cancelReset">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-[500px] border-2 border-red-500">
        <!-- Title -->
        <div class="bg-red-600 text-white px-4 py-3 flex items-center gap-2">
          <span class="text-xl">⚠️</span>
          <h3 class="font-bold text-lg">
            {{ resetConfirmStep === 1 ? 'Confirm Vault Reset' : 'Final Warning!' }}
          </h3>
        </div>

        <!-- Content -->
        <div class="p-6">
          <div v-if="resetConfirmStep === 1" class="space-y-4">
            <p class="text-gray-800 dark:text-gray-200 font-semibold">
              This will PERMANENTLY delete ALL vault data!
            </p>
            <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-4">
              <p class="text-sm text-gray-700 dark:text-gray-300 mb-2 font-semibold">This includes:</p>
              <ul class="list-disc list-inside text-sm text-gray-700 dark:text-gray-300 space-y-1">
                <li>All encrypted files and folders</li>
                <li>Password configuration</li>
                <li>Recovery settings and keys</li>
                <li>All vault metadata</li>
              </ul>
            </div>
            <p class="text-red-600 dark:text-red-400 font-bold text-sm">
              ⚠️ This action CANNOT be undone!
            </p>
          </div>

          <div v-else class="space-y-4">
            <p class="text-red-600 dark:text-red-400 font-bold text-lg">
              Are you ABSOLUTELY sure?
            </p>
            <p class="text-gray-700 dark:text-gray-300 text-sm">
              This is your last chance to cancel. After clicking "Yes, Delete Everything", all your vault data will be permanently destroyed.
            </p>
            <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-300 dark:border-yellow-800 rounded p-3">
              <p class="text-sm text-yellow-800 dark:text-yellow-300">
                💡 <strong>Tip:</strong> Make sure you have backups of any important data before proceeding.
              </p>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="px-6 pb-6 flex justify-end gap-3">
          <button
            @click="cancelReset"
            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded font-medium transition-colors"
          >
            Cancel
          </button>
          <button
            @click="confirmReset"
            class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded font-medium transition-colors"
          >
            {{ resetConfirmStep === 1 ? 'Yes, I Understand' : 'Yes, Delete Everything' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Migration Confirmation Dialog -->
    <div v-if="showMigrationConfirm" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[2000]" @click.self="showMigrationConfirm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-[500px] border-2 border-blue-500">
        <div class="bg-blue-600 text-white px-4 py-3 flex items-center gap-2">
          <span class="text-xl">📦</span>
          <h3 class="font-bold text-lg">Migrate Vault Data?</h3>
        </div>

        <div class="p-6 space-y-4">
          <p class="text-gray-800 dark:text-gray-200">
            Do you want to move your existing vault data to the new location?
          </p>

          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p class="text-sm text-gray-700 dark:text-gray-300 mb-2 font-semibold">Options:</p>
            <ul class="list-disc list-inside text-sm text-gray-700 dark:text-gray-300 space-y-1">
              <li><strong>Migrate:</strong> Copy all vault files to new location and delete old files</li>
              <li><strong>Don't Migrate:</strong> Start fresh in new location (old data remains accessible at old location)</li>
            </ul>
          </div>

          <div class="text-sm text-gray-600 dark:text-gray-400">
            <strong>New location:</strong>
            <code class="block mt-1 px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs">
              {{ pendingVaultPath }}
            </code>
          </div>
        </div>

        <div class="px-6 pb-6 flex justify-end gap-3">
          <button
            @click="showMigrationConfirm = false"
            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded font-medium transition-colors"
          >
            Cancel
          </button>
          <button
            @click="confirmVaultMigration(false)"
            class="px-4 py-2 bg-orange-500 hover:bg-orange-600 text-white rounded font-medium transition-colors"
          >
            Don't Migrate
          </button>
          <button
            @click="confirmVaultMigration(true)"
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-colors"
          >
            Migrate Data
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-dialog {
  background: #ECE9D8;
  border: 1px solid #0054E3;
  border-radius: 8px 8px 0 0;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

/* Title Bar */
.title-bar {
  background: linear-gradient(to bottom, #0997FF, #0053EE 8%, #003DD9);
  color: white;
  padding: 4px 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-radius: 8px 8px 0 0;
  font-weight: bold;
  font-size: 13px;
}

.close-btn {
  background: #E64846;
  color: white;
  border: none;
  border-radius: 3px;
  width: 24px;
  height: 20px;
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #FF5D5B;
}

/* Content */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.setting-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: bold;
  color: #003C74;
  margin-bottom: 12px;
  padding-bottom: 4px;
  border-bottom: 1px solid #BCD5EE;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-label {
  font-size: 13px;
  color: #000;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.setting-description {
  margin-top: 6px;
  font-size: 11px;
  color: #666;
  font-style: italic;
}

/* Radio Group */
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.radio-option {
  display: flex;
  flex-direction: column;
  padding: 12px;
  border: 1px solid #ACA899;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.radio-option:hover {
  border-color: #0054E3;
  background: #F0F5FF;
}

.radio-option input[type="radio"] {
  margin-right: 8px;
}

.radio-option span:first-of-type {
  font-weight: 500;
  margin-left: 24px;
}

.radio-description {
  font-size: 12px;
  color: #666;
  margin-left: 32px;
  margin-top: 4px;
}

/* Checkbox */
.checkbox {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

/* Select */
.select {
  padding: 6px 8px;
  border: 1px solid #ACA899;
  border-radius: 4px;
  background: white;
  font-size: 13px;
  min-width: 200px;
  cursor: pointer;
}

.select:focus {
  outline: none;
  border-color: #0054E3;
}

/* Message */
.message {
  padding: 12px;
  border-radius: 4px;
  margin-top: 16px;
  font-size: 13px;
}

.message.success {
  background: #D4EDDA;
  border: 1px solid #C3E6CB;
  color: #155724;
}

.message.error {
  background: #F8D7DA;
  border: 1px solid #F5C6CB;
  color: #721C24;
}

/* Button Bar */
.button-bar {
  padding: 12px 20px;
  background: #ECE9D8;
  border-top: 1px solid #ACA899;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 6px 20px;
  border: 1px solid;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  min-width: 80px;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(to bottom, #FFF 0%, #DCDCDC 100%);
  border-color: #003C74;
  color: #000;
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(to bottom, #FFF 0%, #C0C0C0 100%);
}

.btn-secondary {
  background: linear-gradient(to bottom, #FFF 0%, #DCDCDC 100%);
  border-color: #ACA899;
  color: #000;
}

.btn-secondary:hover:not(:disabled) {
  background: linear-gradient(to bottom, #FFF 0%, #C0C0C0 100%);
}
</style>
