<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import type { GroupBy } from '../composables/useGrouping';

interface Props {
  modelValue: GroupBy;
  options: ReadonlyArray<{ value: string; label: string; icon: string }>;
}

interface Emits {
  (e: 'update:modelValue', value: GroupBy): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const isOpen = ref(false);
const dropdownRef = ref<HTMLDivElement | null>(null);

const toggleDropdown = () => {
  isOpen.value = !isOpen.value;
};

const selectOption = (value: GroupBy) => {
  emit('update:modelValue', value);
  isOpen.value = false;
};

const handleClickOutside = (event: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});

const getCurrentLabel = () => {
  const current = props.options.find(opt => opt.value === props.modelValue);
  return current?.icon || '▪️';
};
</script>

<template>
  <div ref="dropdownRef" class="relative">
    <!-- Trigger Button -->
    <button
      @click="toggleDropdown"
      :class="[
        'w-[30px] h-[28px] bg-gradient-to-b from-white to-[#E3DED4]',
        'border border-[#8B8B8B] hover:border-[#0054E3]',
        'flex items-center justify-center transition-all',
        isOpen && 'bg-[#C1D2EE] from-[#C1D2EE] to-[#A8C0E8] border-[#0054E3]'
      ]"
      title="Group By"
    >
      {{ getCurrentLabel() }}
    </button>

    <!-- Dropdown Menu -->
    <transition name="dropdown">
      <div
        v-if="isOpen"
        class="absolute top-full right-0 mt-1 bg-[#ECE9D8] border-2 border-[#0054E3] rounded shadow-2xl min-w-[200px] z-50 overflow-hidden"
      >
        <!-- Header -->
        <div class="bg-gradient-to-r from-[#0054E3] to-[#0A246A] px-3 py-1.5 border-b border-[#0054E3]">
          <div class="text-white font-bold text-[10px] uppercase tracking-wide">Group By</div>
        </div>

        <!-- Options -->
        <div class="py-1">
          <label
            v-for="option in options"
            :key="option.value"
            class="flex items-center gap-2 px-3 py-2 hover:bg-[#C1D2EE] cursor-pointer transition-colors group"
            @click="selectOption(option.value as GroupBy)"
          >
            <!-- Radio Button -->
            <div class="relative flex items-center justify-center w-4 h-4">
              <div
                :class="[
                  'w-3 h-3 rounded-full border-2 transition-all',
                  modelValue === option.value
                    ? 'border-[#0054E3] bg-[#0054E3]'
                    : 'border-[#7F9DB9] bg-white group-hover:border-[#0054E3]'
                ]"
              >
                <div
                  v-if="modelValue === option.value"
                  class="absolute inset-0 flex items-center justify-center"
                >
                  <div class="w-1.5 h-1.5 rounded-full bg-white"></div>
                </div>
              </div>
            </div>

            <!-- Icon -->
            <span class="text-sm">{{ option.icon }}</span>

            <!-- Label -->
            <span
              :class="[
                'flex-1 text-[11px] transition-colors',
                modelValue === option.value
                  ? 'font-bold text-[#0054E3]'
                  : 'text-[#0b0b0b] group-hover:text-[#0054E3]'
              ]"
            >
              {{ option.label }}
            </span>
          </label>
        </div>

        <!-- Footer -->
        <div class="border-t border-[#919B9C] px-3 py-1.5 bg-[#F1EFE2]">
          <div class="text-[9px] text-gray-500 italic">
            {{ modelValue === 'none' ? 'Items are not grouped' : `Grouped by ${options.find(o => o.value === modelValue)?.label}` }}
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}
</style>
