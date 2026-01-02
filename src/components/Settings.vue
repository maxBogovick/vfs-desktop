<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, FileSystemBackend, ViewMode } from '../types'
import { useTheme } from '../composables/useTheme'

const emit = defineEmits<{
  close: []
}>()

const config = ref<AppConfig>({
  filesystem_backend: 'real',
  show_hidden_files: false,
  default_view_mode: 'grid',
  theme: 'luna'
})

const saving = ref(false)
const message = ref<{ text: string; type: 'success' | 'error' } | null>(null)

// Загрузить конфигурацию при монтировании
onMounted(async () => {
  try {
    const loadedConfig = await invoke<AppConfig>('get_config')
    config.value = loadedConfig
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
      <div class="settings-content">
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
