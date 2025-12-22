<script setup lang="ts">
interface Props {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'warning' | 'danger' | 'info';
}

interface Emits {
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: 'OK',
  cancelText: 'Cancel',
  type: 'warning',
});

const emit = defineEmits<Emits>();

const getIcon = () => {
  const icons = {
    warning: '⚠️',
    danger: '🗑️',
    info: 'ℹ️',
  };
  return icons[props.type];
};
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      @click="emit('cancel')"
      class="fixed inset-0 bg-black/30 z-[60] flex items-center justify-center"
    >
      <div
        @click.stop
        class="bg-[#ECE9D8] rounded border-2 border-[#0054E3] shadow-2xl w-[400px] overflow-hidden animate-pop-in"
      >
        <!-- Title Bar -->
        <div class="bg-gradient-to-r from-[#0054E3] to-[#0A246A] h-7 flex items-center px-2 gap-2">
          <div class="w-4 h-4 flex items-center justify-center text-xs">
            {{ getIcon() }}
          </div>
          <div class="flex-1 text-white font-bold text-xs">{{ title }}</div>
          <button
            @click="emit('cancel')"
            class="w-5 h-5 bg-[#C1D2EE] hover:bg-[#FF4444] flex items-center justify-center text-[10px] font-bold border border-white/30"
          >
            ✕
          </button>
        </div>

        <!-- Content -->
        <div class="p-4">
          <div class="flex gap-4 items-start mb-6">
            <div class="text-4xl flex-shrink-0">{{ getIcon() }}</div>
            <div class="flex-1 pt-1">
              <p class="text-sm">{{ message }}</p>
            </div>
          </div>

          <!-- Buttons -->
          <div class="flex justify-end gap-2">
            <button
              @click="emit('cancel')"
              class="px-4 py-1.5 bg-gradient-to-b from-white to-[#E3DED4] border border-[#8B8B8B] hover:border-[#0054E3] active:bg-[#C1D2EE] rounded text-xs min-w-[75px]"
            >
              {{ cancelText }}
            </button>
            <button
              @click="emit('confirm')"
              :class="[
                'px-4 py-1.5 border-2 rounded text-xs font-bold min-w-[75px]',
                type === 'danger'
                  ? 'bg-gradient-to-b from-[#FF6B6B] to-[#EE5A6F] border-[#C92A2A] text-white hover:from-[#FF5252] hover:to-[#E53935]'
                  : 'bg-gradient-to-b from-white to-[#E3DED4] border-[#0054E3] hover:bg-[#C1D2EE]'
              ]"
            >
              {{ confirmText }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </transition>
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
  animation: popIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes popIn {
  0% {
    transform: scale(0.9);
    opacity: 0;
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
