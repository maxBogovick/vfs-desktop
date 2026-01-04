<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{
  complete: [recoveryKey: string]
  skip: []
}>()

const email = ref('')
const emailVerified = ref(false)
const recoveryKey = ref<string | null>(null)
const error = ref<string | null>(null)
const isSubmitting = ref(false)
const step = ref<'setup' | 'display_key'>('setup')

const isValidEmail = computed(() => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(email.value)
})

const canSubmit = computed(() => isValidEmail.value)

async function setupRecovery() {
  if (!canSubmit.value) return

  try {
    isSubmitting.value = true
    error.value = null

    // Setup recovery channel
    const channels = [
      {
        type: 'email',
        address: email.value,
        verified: true // In production, should send verification email first
      }
    ]

    const key = await invoke<string>('vault_setup_recovery', { channels })
    recoveryKey.value = key
    step.value = 'display_key'
  } catch (err: any) {
    error.value = err.toString().replace('Error: ', '')
  } finally {
    isSubmitting.value = false
  }
}

function complete() {
  if (recoveryKey.value) {
    emit('complete', recoveryKey.value)
  }
}

function skip() {
  emit('skip')
}
</script>

<template>
  <div class="space-y-4">
    <!-- Setup Email -->
    <div v-if="step === 'setup'">
      <h3 class="text-lg font-semibold mb-3 text-[var(--vf-text-primary)]">Setup Password Recovery (Optional)</h3>

      <p class="text-sm text-[var(--vf-text-secondary)] mb-4">
        Configure a recovery method in case you forget your password. You can skip this step and add it later.
      </p>

      <div class="space-y-3">
        <div>
          <label class="block text-xs font-medium text-[var(--vf-text-secondary)] mb-1">
            Recovery Email
          </label>
          <input
            v-model="email"
            type="email"
            placeholder="your.email@example.com"
            class="w-full px-3 py-2 bg-[var(--vf-bg-secondary)] border border-[var(--vf-border)] rounded text-sm text-[var(--vf-text-primary)] focus:outline-none focus:border-[var(--vf-accent-primary)] focus:ring-1 focus:ring-[var(--vf-accent-primary)]"
            :class="{ 'border-red-500': email && !isValidEmail }"
          />
          <div v-if="email && !isValidEmail" class="text-xs text-red-500 mt-1">
            Please enter a valid email address
          </div>
        </div>

        <div class="text-xs text-[var(--vf-text-secondary)] bg-[var(--vf-bg-secondary)] p-3 rounded border border-[var(--vf-border)]">
          <strong>ℹ️ How it works:</strong><br>
          • A recovery key will be generated and sent to your email<br>
          • In the future: Push notifications, SMS, Telegram bot<br>
          • You can change recovery methods anytime in settings
        </div>

        <!-- Error Message -->
        <div v-if="error" class="text-xs text-red-500 bg-red-50 dark:bg-red-900/20 p-2 rounded border border-red-200 dark:border-red-800">
          {{ error }}
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-6">
        <button
          @click="skip"
          class="px-4 py-2 text-sm font-medium text-[var(--vf-text-secondary)] hover:text-[var(--vf-text-primary)] transition-colors"
        >
          Skip
        </button>
        <button
          @click="setupRecovery"
          :disabled="!canSubmit || isSubmitting"
          class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {{ isSubmitting ? 'Setting up...' : 'Setup Recovery' }}
        </button>
      </div>
    </div>

    <!-- Display Recovery Key -->
    <div v-else-if="step === 'display_key' && recoveryKey" class="space-y-4">
      <h3 class="text-lg font-semibold text-[var(--vf-text-primary)]">⚠️ Save Your Recovery Key</h3>

      <p class="text-sm text-[var(--vf-text-secondary)]">
        This is your master recovery key. Save it in a safe place - you'll need it if you lose your password.
      </p>

      <div class="bg-[var(--vf-bg-secondary)] p-4 rounded border-2 border-[var(--vf-accent-primary)]">
        <div class="font-mono text-xs break-all text-[var(--vf-text-primary)] select-all">
          {{ recoveryKey }}
        </div>
      </div>

      <div class="text-xs text-[var(--vf-text-secondary)] space-y-2">
        <p><strong>✅ Save this key by:</strong></p>
        <ul class="list-disc list-inside ml-2 space-y-1">
          <li>Writing it down on paper</li>
          <li>Storing in a password manager</li>
          <li>Printing and storing in a safe place</li>
        </ul>
        <p class="text-red-500 mt-3">
          <strong>⚠️ Never:</strong> Share this key or store it in plain text files
        </p>
      </div>

      <div class="flex justify-end mt-6">
        <button
          @click="complete"
          class="px-4 py-2 bg-[var(--vf-accent-primary)] hover:bg-[var(--vf-accent-hover)] text-white text-sm font-medium rounded transition-colors"
        >
          I've Saved the Key
        </button>
      </div>
    </div>
  </div>
</template>
