<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{
  success: []
  cancel: []
}>()

enum RecoveryStep {
  SELECT_CHANNEL = 'select_channel',
  ENTER_CODE = 'enter_code',
  SET_PASSWORD = 'set_password',
  SUCCESS = 'success'
}

const currentStep = ref<RecoveryStep>(RecoveryStep.SELECT_CHANNEL)
const availableChannels = ref<string[]>([])
const selectedChannel = ref<string>('')
const verificationCode = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const error = ref<string | null>(null)
const isLoading = ref(false)

const passwordsMatch = computed(() => newPassword.value === confirmPassword.value)
const canSubmitCode = computed(() => verificationCode.value.length === 6)
const canSubmitPassword = computed(() =>
  newPassword.value.length >= 8 && passwordsMatch.value
)

// Load available recovery channels
async function loadChannels() {
  try {
    isLoading.value = true
    const channels = await invoke<string[]>('vault_get_recovery_channels')
    availableChannels.value = channels

    if (channels.length === 0) {
      error.value = 'No recovery channels configured'
    } else if (channels.length === 1) {
      selectedChannel.value = channels[0]
    }
  } catch (err: any) {
    error.value = err.toString()
  } finally {
    isLoading.value = false
  }
}

// Request password reset
async function requestReset() {
  if (!selectedChannel.value) {
    error.value = 'Please select a recovery channel'
    return
  }

  try {
    isLoading.value = true
    error.value = null

    await invoke('vault_request_password_reset', {
      channelType: selectedChannel.value.toLowerCase()
    })

    currentStep.value = RecoveryStep.ENTER_CODE
  } catch (err: any) {
    error.value = err.toString().replace('Error: ', '')
  } finally {
    isLoading.value = false
  }
}

// Verify code
function verifyCode() {
  if (canSubmitCode.value) {
    currentStep.value = RecoveryStep.SET_PASSWORD
  }
}

// Set new password
async function setNewPassword() {
  if (!canSubmitPassword.value) return

  try {
    isLoading.value = true
    error.value = null

    await invoke('vault_verify_reset_code', {
      code: verificationCode.value,
      newPassword: newPassword.value
    })

    currentStep.value = RecoveryStep.SUCCESS

    // Auto-close after 2 seconds
    setTimeout(() => {
      emit('success')
    }, 2000)
  } catch (err: any) {
    error.value = err.toString().replace('Error: ', '')
  } finally {
    isLoading.value = false
  }
}

// Initialize
loadChannels()
</script>

<template>
  <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-[10000] flex items-center justify-center p-4">
    <div class="bg-[var(--vf-bg-primary)] rounded border-2 border-[var(--vf-accent-primary)] shadow-2xl w-full max-w-md overflow-hidden">
      <!-- Title Bar -->
      <div class="bg-gradient-to-r from-[var(--vf-accent-primary)] to-[var(--vf-accent-hover)] h-8 flex items-center px-3 gap-2">
        <div class="flex-1 text-white font-bold text-sm">🔓 Password Recovery</div>
        <button
          @click="emit('cancel')"
          class="text-white hover:bg-white/20 rounded px-2 text-xs"
        >
          ✕
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-4">
        <!-- Step 1: Select Channel -->
        <div v-if="currentStep === RecoveryStep.SELECT_CHANNEL">
          <h3 class="text-lg font-semibold mb-4 text-[var(--vf-text-primary)]">Select Recovery Method</h3>

          <div v-if="isLoading" class="text-center py-8">
            <div class="animate-spin inline-block w-8 h-8 border-4 border-[var(--vf-accent-primary)] border-t-transparent rounded-full"></div>
            <p class="mt-2 text-sm text-[var(--vf-text-secondary)]">Loading...</p>
          </div>

          <div v-else-if="availableChannels.length > 0" class="space-y-3">
            <label
              v-for="channel in availableChannels"
              :key="channel"
              class="block p-3 border border-[var(--vf-border)] rounded hover:border-[var(--vf-accent-primary)] cursor-pointer"
              :class="{ 'border-[var(--vf-accent-primary)] bg-[var(--vf-accent-primary)]/5': selectedChannel === channel }"
            >
              <input
                v-model="selectedChannel"
                type="radio"
                :value="channel"
                class="mr-2"
              />
              <span class="text-sm font-medium">{{ channel }}</span>
            </label>

            <p class="text-xs text-[var(--vf-text-secondary)] mt-4">
              A verification code will be sent to your registered {{ selectedChannel.toLowerCase() }}.
            </p>
          </div>

          <div v-else class="text-center py-8">
            <p class="text-sm text-[var(--vf-text-secondary)]">No recovery methods configured</p>
          </div>
        </div>

        <!-- Step 2: Enter Verification Code -->
        <div v-else-if="currentStep === RecoveryStep.ENTER_CODE">
          <h3 class="text-lg font-semibold mb-4 text-[var(--vf-text-primary)]">Enter Verification Code</h3>

          <p class="text-sm text-[var(--vf-text-secondary)] mb-4">
            A 6-digit code has been sent to your {{ selectedChannel.toLowerCase() }}.
          </p>

          <input
            v-model="verificationCode"
            type="text"
            maxlength="6"
            placeholder="000000"
            @keyup.enter="verifyCode"
            class="w-full px-4 py-3 text-center text-2xl tracking-widest bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
            autofocus
          />

          <p class="text-xs text-[var(--vf-text-secondary)] mt-2">
            Code expires in 15 minutes
          </p>
        </div>

        <!-- Step 3: Set New Password -->
        <div v-else-if="currentStep === RecoveryStep.SET_PASSWORD" class="space-y-3">
          <h3 class="text-lg font-semibold mb-4 text-[var(--vf-text-primary)]">Set New Password</h3>

          <div>
            <label class="block text-xs font-medium text-[var(--vf-text-secondary)] mb-1">
              New Password
            </label>
            <input
              v-model="newPassword"
              type="password"
              placeholder="Enter new password"
              class="w-full px-3 py-2 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-sm text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-[var(--vf-text-secondary)] mb-1">
              Confirm Password
            </label>
            <input
              v-model="confirmPassword"
              type="password"
              placeholder="Confirm new password"
              @keyup.enter="setNewPassword"
              class="w-full px-3 py-2 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-sm text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
              :class="{ 'border-red-500': confirmPassword && !passwordsMatch }"
            />
            <div v-if="confirmPassword && !passwordsMatch" class="text-xs text-red-500 mt-1">
              Passwords do not match
            </div>
          </div>

          <p class="text-xs text-[var(--vf-text-secondary)]">
            Password must be at least 8 characters
          </p>
        </div>

        <!-- Step 4: Success -->
        <div v-else-if="currentStep === RecoveryStep.SUCCESS" class="text-center py-8">
          <div class="text-6xl mb-4">✅</div>
          <h3 class="text-lg font-semibold text-[var(--vf-text-primary)] mb-2">Password Reset Successful</h3>
          <p class="text-sm text-[var(--vf-text-secondary)]">You can now use your new password to unlock the vault.</p>
        </div>

        <!-- Error Message -->
        <div v-if="error" class="text-xs text-red-500 bg-red-50 dark:bg-red-900/20 p-3 rounded border border-red-200 dark:border-red-800">
          {{ error }}
        </div>
      </div>

      <!-- Actions -->
      <div v-if="currentStep !== RecoveryStep.SUCCESS" class="flex justify-end gap-2 px-6 pb-6">
        <button
          @click="emit('cancel')"
          class="px-4 py-2 text-sm font-medium text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] transition-colors"
        >
          Cancel
        </button>

        <button
          v-if="currentStep === RecoveryStep.SELECT_CHANNEL"
          @click="requestReset"
          :disabled="!selectedChannel || isLoading"
          class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ isLoading ? 'Sending...' : 'Send Code' }}
        </button>

        <button
          v-else-if="currentStep === RecoveryStep.ENTER_CODE"
          @click="verifyCode"
          :disabled="!canSubmitCode"
          class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Next
        </button>

        <button
          v-else-if="currentStep === RecoveryStep.SET_PASSWORD"
          @click="setNewPassword"
          :disabled="!canSubmitPassword || isLoading"
          class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ isLoading ? 'Resetting...' : 'Reset Password' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Remove number input spinners */
input[type="text"]::-webkit-inner-spin-button,
input[type="text"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>
