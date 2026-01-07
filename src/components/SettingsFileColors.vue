<script setup lang="ts">
import { ref, computed } from 'vue';
import { useFileColoring } from '../composables/useFileColoring';

const { config, updateConfig, resetConfig } = useFileColoring();

// Local state for adding new extension
const newExtension = ref('');
const newExtensionColor = ref('#000000');

// Local state for adding new custom rule
const newRuleName = ref('');
const newRulePattern = ref('');
const newRuleColor = ref('#000000');

const handleAddExtension = () => {
  if (newExtension.value && newExtensionColor.value) {
    const ext = newExtension.value.trim().toLowerCase().replace(/^\./, '');
    if (ext) {
      config.extensions[ext] = newExtensionColor.value;
      updateConfig({ extensions: { ...config.extensions } }); // Trigger save
      newExtension.value = '';
    }
  }
};

const handleRemoveExtension = (ext: string) => {
  delete config.extensions[ext];
  updateConfig({ extensions: { ...config.extensions } });
};

const handleAddRule = () => {
  if (newRuleName.value && newRulePattern.value && newRuleColor.value) {
    config.customRules.push({
      id: Date.now().toString(),
      name: newRuleName.value,
      pattern: newRulePattern.value,
      color: newRuleColor.value,
      enabled: true
    });
    updateConfig({ customRules: [...config.customRules] });
    newRuleName.value = '';
    newRulePattern.value = '';
  }
};

const handleRemoveRule = (index: number) => {
  config.customRules.splice(index, 1);
  updateConfig({ customRules: [...config.customRules] });
};

const handleToggleRule = (index: number) => {
  config.customRules[index].enabled = !config.customRules[index].enabled;
  updateConfig({ customRules: [...config.customRules] });
};
</script>

<template>
  <div class="h-full overflow-y-auto p-6 text-[var(--vf-text-primary)]">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold">File Coloring Configuration</h2>
      <div class="flex gap-2">
        <button 
          @click="resetConfig" 
          class="px-3 py-1 bg-red-100 text-red-600 rounded hover:bg-red-200 text-sm transition-colors"
        >
          Reset to Defaults
        </button>
      </div>
    </div>

    <!-- Main Toggle -->
    <div class="mb-6 flex items-center gap-2">
      <input 
        type="checkbox" 
        v-model="config.enabled" 
        @change="updateConfig({ enabled: config.enabled })"
        id="enableColoring"
        class="w-4 h-4"
      />
      <label for="enableColoring" class="font-semibold select-none cursor-pointer">Enable File Coloring</label>
    </div>

    <div :class="[config.enabled ? 'opacity-100' : 'opacity-50 pointer-events-none', 'transition-opacity']">
      
      <!-- Base Colors -->
      <section class="mb-8">
        <h3 class="text-lg font-semibold mb-4 border-b border-gray-200 pb-2">Base Colors</h3>
        <div class="grid grid-cols-2 gap-4">
          <div v-for="(color, key) in config.base" :key="key" class="flex items-center justify-between p-2 bg-[var(--vf-bg-secondary)] rounded">
            <span class="capitalize">{{ key }}</span>
            <input 
              type="color" 
              v-model="config.base[key as keyof typeof config.base]" 
              @change="updateConfig({ base: config.base })"
              class="cursor-pointer"
            />
          </div>
        </div>
      </section>

      <!-- Semantic Types -->
      <section class="mb-8">
        <h3 class="text-lg font-semibold mb-4 border-b border-gray-200 pb-2">File Types</h3>
        <div class="grid grid-cols-2 gap-4">
          <div v-for="(color, key) in config.fileTypes" :key="key" class="flex items-center justify-between p-2 bg-[var(--vf-bg-secondary)] rounded">
            <span class="capitalize">{{ key }}</span>
            <input 
              type="color" 
              v-model="config.fileTypes[key as keyof typeof config.fileTypes]" 
              @change="updateConfig({ fileTypes: config.fileTypes })"
              class="cursor-pointer"
            />
          </div>
        </div>
      </section>

      <!-- Special States -->
      <section class="mb-8">
        <h3 class="text-lg font-semibold mb-4 border-b border-gray-200 pb-2">Special Status</h3>
        <div class="grid grid-cols-2 gap-4">
          <div v-for="(color, key) in config.special" :key="key" class="flex items-center justify-between p-2 bg-[var(--vf-bg-secondary)] rounded">
            <span class="capitalize">{{ key }}</span>
            <input 
              type="color" 
              v-model="config.special[key as keyof typeof config.special]" 
              @change="updateConfig({ special: config.special })"
              class="cursor-pointer"
            />
          </div>
        </div>
      </section>

      <!-- Extensions -->
      <section class="mb-8">
        <h3 class="text-lg font-semibold mb-4 border-b border-gray-200 pb-2">Extensions</h3>
        
        <!-- Add new extension -->
        <div class="flex gap-2 mb-4">
          <input 
            v-model="newExtension" 
            placeholder="Extension (e.g. 'ts')" 
            class="px-3 py-1 border rounded text-sm bg-white text-black flex-1"
            @keyup.enter="handleAddExtension"
          />
          <input 
            type="color" 
            v-model="newExtensionColor" 
            class="h-8 w-12 cursor-pointer"
          />
          <button 
            @click="handleAddExtension"
            class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm"
          >
            Add
          </button>
        </div>

        <div class="grid grid-cols-3 gap-2 max-h-40 overflow-y-auto">
          <div v-for="(color, ext) in config.extensions" :key="ext" class="flex items-center gap-2 p-1.5 bg-[var(--vf-bg-secondary)] rounded text-sm">
            <div :style="{ backgroundColor: color }" class="w-3 h-3 rounded-full border border-gray-300"></div>
            <span class="flex-1 font-mono">.{{ ext }}</span>
            <button @click="handleRemoveExtension(ext)" class="text-red-500 hover:text-red-700 px-1">×</button>
          </div>
        </div>
      </section>

      <!-- Custom Rules -->
      <section class="mb-8">
        <h3 class="text-lg font-semibold mb-4 border-b border-gray-200 pb-2">Custom Rules (Regex)</h3>
        
        <!-- Add new rule -->
        <div class="flex gap-2 mb-4 items-end">
          <div class="flex-1">
            <label class="block text-xs text-gray-500 mb-1">Name</label>
            <input 
              v-model="newRuleName" 
              placeholder="Rule Name" 
              class="w-full px-3 py-1 border rounded text-sm bg-white text-black"
            />
          </div>
          <div class="flex-[2]">
            <label class="block text-xs text-gray-500 mb-1">Regex Pattern</label>
            <input 
              v-model="newRulePattern" 
              placeholder="e.g. ^temp_.*" 
              class="w-full px-3 py-1 border rounded text-sm bg-white text-black font-mono"
            />
          </div>
          <div>
            <input 
              type="color" 
              v-model="newRuleColor" 
              class="h-8 w-12 cursor-pointer mb-0.5"
            />
          </div>
          <button 
            @click="handleAddRule"
            class="px-3 py-1.5 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm mb-0.5"
          >
            Add
          </button>
        </div>

        <div class="space-y-2">
          <div v-if="config.customRules.length === 0" class="text-sm text-gray-400 italic">No custom rules defined</div>
          <div v-for="(rule, index) in config.customRules" :key="rule.id" class="flex items-center gap-3 p-2 bg-[var(--vf-bg-secondary)] rounded border border-gray-200">
            <input 
              type="checkbox" 
              :checked="rule.enabled" 
              @change="handleToggleRule(index)"
              class="w-4 h-4 cursor-pointer"
            />
            <div :style="{ backgroundColor: rule.color }" class="w-4 h-4 rounded-full border border-gray-300 flex-shrink-0"></div>
            <div class="flex-1 min-w-0">
              <div class="font-medium text-sm">{{ rule.name }}</div>
              <div class="text-xs text-gray-500 font-mono truncate">{{ rule.pattern }}</div>
            </div>
            <button @click="handleRemoveRule(index)" class="text-red-500 hover:text-red-700 p-1">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </section>

    </div>
  </div>
</template>
