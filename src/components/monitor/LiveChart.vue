<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  path: string;
  color: string; // 'green', 'yellow', 'red', 'blue'
  height?: number;
  label?: string;
  value?: string;
  subValue?: string;
}

const props = withDefaults(defineProps<Props>(), {
  height: 100,
  color: 'green'
});

const colorClasses = computed(() => {
  switch (props.color) {
    case 'yellow': return {
      stroke: 'stroke-yellow-400',
      fill: 'fill-yellow-400/20',
      text: 'text-yellow-400'
    };
    case 'red': return {
      stroke: 'stroke-red-500',
      fill: 'fill-red-500/20',
      text: 'text-red-500'
    };
    case 'blue': return {
      stroke: 'stroke-blue-400',
      fill: 'fill-blue-400/20',
      text: 'text-blue-400'
    };
    case 'green':
    default: return {
      stroke: 'stroke-[var(--vf-accent-primary)]', // Use variable if available or neon green
      fill: 'fill-[var(--vf-accent-primary)]/20',
      text: 'text-[var(--vf-accent-primary)]'
    };
  }
});
</script>

<template>
  <div class="relative flex flex-col h-full bg-[#111] border border-[#333] rounded-sm overflow-hidden group">
    <!-- Header / Values -->
    <div class="absolute top-2 left-2 z-10 pointer-events-none">
      <div v-if="label" class="text-[10px] uppercase tracking-wider text-gray-500 font-mono">{{ label }}</div>
      <div v-if="value" :class="['text-xl font-mono font-bold leading-none mt-0.5', colorClasses.text]">
        {{ value }}
      </div>
      <div v-if="subValue" class="text-[10px] text-gray-400 font-mono mt-0.5">{{ subValue }}</div>
    </div>

    <!-- Chart -->
    <div class="flex-1 w-full relative">
        <svg class="w-full h-full preserve-3d" preserveAspectRatio="none" viewBox="0 0 300 100">
            <defs>
                <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
                    <feGaussianBlur stdDeviation="2" result="blur" />
                    <feComposite in="SourceGraphic" in2="blur" operator="over" />
                </filter>
            </defs>
            
            <!-- Grid Lines (Subtle) -->
            <line x1="0" y1="25" x2="300" y2="25" stroke="#222" stroke-width="1" />
            <line x1="0" y1="50" x2="300" y2="50" stroke="#222" stroke-width="1" />
            <line x1="0" y1="75" x2="300" y2="75" stroke="#222" stroke-width="1" />

            <!-- Path -->
            <path
                :d="path"
                :class="[colorClasses.fill]"
                class="transition-[d] duration-300 ease-linear"
            />
            <path
                :d="path"
                fill="none"
                :class="[colorClasses.stroke]"
                stroke-width="1.5"
                vector-effect="non-scaling-stroke"
                filter="url(#glow)"
                class="transition-[d] duration-300 ease-linear"
            />
        </svg>
    </div>
  </div>
</template>

<style scoped>
.preserve-3d {
    transform: translateZ(0);
}
</style>
