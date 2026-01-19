<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  cores: number[]; // Array of usage 0-100
}

const props = defineProps<Props>();

// Determine grid columns based on core count
// 4 -> 2x2
// 8 -> 4x2
// 12 -> 4x3
// 16 -> 4x4 or 8x2
// 32 -> 8x4
const gridStyle = computed(() => {
  const count = props.cores.length;
  let cols = 4;
  if (count <= 4) cols = 2;
  else if (count <= 16) cols = 4;
  else if (count <= 32) cols = 8;
  else cols = 10;
  
  return {
    display: 'grid',
    gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
    gap: '4px'
  };
});

const getCoreStyle = (usage: number) => {
  // Map 0-100 to 130-0 Hue
  // 0% -> 130 (Neon Green)
  // 50% -> 65 (Yellow)
  // 100% -> 0 (Red)
  const hue = Math.max(0, 130 - (usage * 1.3));
  const color = `hsl(${hue}, 100%, 50%)`;
  
  return {
    backgroundColor: `hsla(${hue}, 100%, 50%, 0.1)`, // Low opacity bg
    border: `1px solid ${color}`,
    boxShadow: usage > 80 ? `0 0 8px ${color}` : 'none', // Glow on high load
    color: color
  };
};
</script>

<template>
  <div class="h-full flex flex-col bg-[#111] border border-[#333] rounded-sm p-2">
    <div class="text-[10px] uppercase tracking-wider text-gray-500 font-mono mb-2">CPU Cores</div>
    
    <div class="flex-1 min-h-0" :style="gridStyle">
      <div
        v-for="(usage, index) in cores"
        :key="index"
        :style="getCoreStyle(usage)"
        class="rounded-sm flex items-center justify-center relative overflow-hidden transition-colors duration-300"
      >
        <!-- Fill bar effect (vertical) -->
        <div 
            class="absolute bottom-0 left-0 right-0 opacity-20 transition-[height] duration-300 ease-linear"
            :style="{ 
                height: `${usage}%`,
                backgroundColor: 'currentColor' 
            }"
        ></div>
        
        <!-- Text -->
        <span class="text-[9px] font-mono font-bold z-10 opacity-80">
            {{ Math.round(usage) }}%
        </span>
      </div>
    </div>
  </div>
</template>
