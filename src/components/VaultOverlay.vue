<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVault, VaultStatus } from '../composables/useVault'
import PasswordRecovery from './PasswordRecovery.vue'
import RecoverySetup from './RecoverySetup.vue'

const vault = useVault()
const showRecovery = ref(false)
const showRecoverySetup = ref(false)
const isRecoveryConfigured = ref(false)

const password = ref('')
const confirmPassword = ref('')
const isSubmitting = ref(false)
const error = ref<string | null>(null)

const isSetupMode = computed(() => vault.status.value === VaultStatus.UNINITIALIZED)
const isLoginMode = computed(() => vault.status.value === VaultStatus.LOCKED)

const title = computed(() => {
  if (isSetupMode.value) return '🔒 Secure Vault Setup'
  if (isLoginMode.value) return '🔐 Unlock Vault'
  return '⏳ Loading...'
})

const passwordsMatch = computed(() => {
  if (!isSetupMode.value) return true
  return password.value === confirmPassword.value
})

const canSubmit = computed(() => {
  if (isSubmitting.value) return false
  if (!password.value) return false
  if (isSetupMode.value && !passwordsMatch.value) return false
  return true
})

async function handleSubmit() {
  if (!canSubmit.value) return

  isSubmitting.value = true
  error.value = null

  try {
    if (isSetupMode.value) {
      // Setup new vault
      if (!passwordsMatch.value) {
        error.value = 'Passwords do not match'
        return
      }
      await vault.initialize(password.value)

      // Clear form
      password.value = ''
      confirmPassword.value = ''

      // Show recovery setup after vault creation
      showRecoverySetup.value = true
    } else {
      // Unlock existing vault
      await vault.unlock(password.value)

      // Clear form on success
      password.value = ''
      confirmPassword.value = ''
    }
  } catch (err: any) {
    error.value = err.toString().replace('Error: ', '')
  } finally {
    isSubmitting.value = false
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && canSubmit.value) {
    handleSubmit()
  }
}

function handleRecoverySuccess() {
  showRecovery.value = false
  // Vault will be unlocked automatically
  vault.checkStatus()
}

function handleRecoveryCancel() {
  showRecovery.value = false
}

async function handleRecoverySetupComplete(recoveryKey: string) {
  console.log('Recovery key saved:', recoveryKey.substring(0, 10) + '...')
  showRecoverySetup.value = false
  // Refresh recovery status
  await checkRecoveryConfigured()
  // Vault is already unlocked, just close the overlay
}

function handleRecoverySetupSkip() {
  showRecoverySetup.value = false
  // User skipped recovery setup, vault is still unlocked
}

// Check if recovery is configured when in login mode
async function checkRecoveryConfigured() {
  if (isLoginMode.value) {
    try {
      const configured = await invoke<boolean>('vault_is_recovery_configured')
      console.log('[VaultOverlay] Recovery configured:', configured)
      isRecoveryConfigured.value = configured
    } catch (err) {
      console.error('[VaultOverlay] Failed to check recovery configuration:', err)
      isRecoveryConfigured.value = false
    }
  }
}

// Watch vault status and check recovery when entering login mode
watch(() => vault.status.value, () => {
  checkRecoveryConfigured()
}, { immediate: true })
</script>

<template>
  <div>
    <transition name="fade">
    <div
      v-if="vault.isVaultOverlayVisible.value || showRecoverySetup"
      class="fixed inset-0 bg-black/50 backdrop-blur-sm z-[9999] flex items-center justify-center"
    >
      <div
        @click.stop
        class="bg-[var(--vf-bg-primary)] rounded border-2 border-[var(--vf-accent-primary)] shadow-2xl w-[450px] overflow-hidden animate-pop-in"
      >
        <!-- Recovery Setup (after vault creation) -->
        <div v-if="showRecoverySetup">
          <!-- Title Bar -->
          <div class="bg-gradient-to-r from-[var(--vf-accent-primary)] to-[var(--vf-accent-hover)] h-8 flex items-center px-3 gap-2">
            <div class="flex-1 text-white font-bold text-sm">🔐 Setup Password Recovery</div>
          </div>

          <!-- RecoverySetup Component -->
          <div class="p-6">
            <RecoverySetup
              @complete="handleRecoverySetupComplete"
              @skip="handleRecoverySetupSkip"
            />
          </div>
        </div>

        <!-- Normal Vault Setup/Login -->
        <div v-else>
          <!-- Title Bar -->
          <div class="bg-gradient-to-r from-[var(--vf-accent-primary)] to-[var(--vf-accent-hover)] h-8 flex items-center px-3 gap-2">
            <div class="flex-1 text-white font-bold text-sm">{{ title }}</div>
          </div>

          <!-- Content -->
          <div class="p-6 space-y-4">
          <!-- Description -->
          <div class="text-sm text-[var(--vf-text-secondary)]">
            <p v-if="isSetupMode">
              Create a master password to secure your virtual file system. This password will be required every time you start the application.
            </p>
            <p v-else-if="isLoginMode">
              Enter your master password to unlock the vault and access your files.
            </p>
            <p v-else>
              Checking vault status...
            </p>
          </div>

          <!-- Password Input -->
          <div v-if="!vault.isChecking.value" class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-[var(--vf-text-secondary)] mb-1">
                Master Password
              </label>
              <input
                v-model="password"
                type="password"
                @keydown="handleKeydown"
                :disabled="isSubmitting"
                class="w-full px-3 py-2 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-sm text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
                placeholder="Enter your password"
                autofocus
              />
            </div>

            <!-- Confirm Password (only in setup mode) -->
            <div v-if="isSetupMode">
              <label class="block text-xs font-medium text-[var(--vf-text-secondary)] mb-1">
                Confirm Password
              </label>
              <input
                v-model="confirmPassword"
                type="password"
                @keydown="handleKeydown"
                :disabled="isSubmitting"
                class="w-full px-3 py-2 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-sm text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
                :class="{ 'border-red-500': confirmPassword && !passwordsMatch }"
                placeholder="Confirm your password"
              />
              <div v-if="confirmPassword && !passwordsMatch" class="text-xs text-red-500 mt-1">
                Passwords do not match
              </div>
            </div>

            <!-- Error Message -->
            <div v-if="error" class="text-xs text-red-500 bg-red-50 dark:bg-red-900/20 p-2 rounded border border-red-200 dark:border-red-800">
              {{ error }}
            </div>

            <!-- Forgot Password Link (Login mode only, if recovery configured) -->
            <div v-if="isLoginMode" class="text-center">
              <button
                @click="showRecovery = true"
                class="text-xs text-[var(--vf-accent-primary)] hover:text-[var(--vf-accent-hover)] hover:underline transition-colors"
              >
                Forgot password?
              </button>
            </div>

            <!-- Security Note -->
            <div v-if="isSetupMode" class="text-xs text-[var(--vf-text-secondary)] bg-[var(--vf-bg-secondary)] p-3 rounded border border-[var(--vf-border)]">
              <strong>⚠️ Important:</strong> Store this password securely. It cannot be recovered if lost. All vault data will be encrypted with Argon2id + AES-256-GCM.
            </div>
          </div>

          <!-- Loading state -->
          <div v-else class="text-center py-4">
            <div class="animate-spin inline-block w-8 h-8 border-4 border-[var(--vf-accent-primary)] border-t-transparent rounded-full"></div>
          </div>
        </div>

          <!-- Actions -->
          <div v-if="!vault.isChecking.value" class="flex justify-end gap-2 px-6 pb-6">
            <button
              @click="handleSubmit"
              :disabled="!canSubmit"
              class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ isSubmitting ? 'Processing...' : isSetupMode ? 'Create Vault' : 'Unlock' }}
            </button>
          </div>
        </div>
        <!-- End Normal Vault Setup/Login -->
      </div>
    </div>
    </transition>

    <!-- Password Recovery Modal -->
    <PasswordRecovery
      v-if="showRecovery"
      @success="handleRecoverySuccess"
      @cancel="handleRecoveryCancel"
    />
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.animate-pop-in {
  animation: pop-in 0.2s ease-out;
}

@keyframes pop-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
