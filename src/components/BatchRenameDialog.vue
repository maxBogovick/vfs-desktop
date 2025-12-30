<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type {
  FileItem,
  BatchRenamePattern,
  PrefixPattern,
  SuffixPattern,
  ReplacePattern,
  RegexPattern,
  NumberingPattern,
  CasePattern,
  CaseChangeType,
  RenamePreviewItem,
  BatchRenameConfig,
} from '../types';
import {
  generateRenamePreview,
  validateRenameOperation,
  createDefaultPatterns,
  PATTERN_PRESETS,
} from '../utils/batchRenamePatterns';

interface Props {
  isOpen: boolean;
  files: FileItem[];
}

interface Emits {
  (e: 'confirm', config: BatchRenameConfig): void;
  (e: 'cancel'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// State
const patterns = ref<BatchRenamePattern[]>([]);
const applyToFolders = ref(true);
const applyToFiles = ref(true);
const preserveExtension = ref(true);
const activeTab = ref<'patterns' | 'preview'>('patterns');

// Pattern factories
const defaults = createDefaultPatterns();

// Preview
const preview = computed<RenamePreviewItem[]>(() => {
  if (!props.files.length) return [];

  const config: BatchRenameConfig = {
    patterns: patterns.value,
    applyToFolders: applyToFolders.value,
    applyToFiles: applyToFiles.value,
    preserveExtension: preserveExtension.value,
  };

  return generateRenamePreview(props.files, config);
});

const validation = computed(() => {
  return validateRenameOperation(preview.value);
});

const changesCount = computed(() => {
  return preview.value.filter(
    (item) => !item.hasError && item.originalName !== item.newName
  ).length;
});

// Pattern management
function addPattern(type: string) {
  switch (type) {
    case 'prefix':
      patterns.value.push(defaults.prefix());
      break;
    case 'suffix':
      patterns.value.push(defaults.suffix());
      break;
    case 'replace':
      patterns.value.push(defaults.replace());
      break;
    case 'regex':
      patterns.value.push(defaults.regex());
      break;
    case 'numbering':
      patterns.value.push(defaults.numbering());
      break;
    case 'case':
      patterns.value.push(defaults.case());
      break;
  }
}

function removePattern(index: number) {
  patterns.value.splice(index, 1);
}

function movePatternUp(index: number) {
  if (index > 0) {
    const temp = patterns.value[index];
    patterns.value[index] = patterns.value[index - 1];
    patterns.value[index - 1] = temp;
  }
}

function movePatternDown(index: number) {
  if (index < patterns.value.length - 1) {
    const temp = patterns.value[index];
    patterns.value[index] = patterns.value[index + 1];
    patterns.value[index + 1] = temp;
  }
}

function applyPreset(presetName: string) {
  const preset = PATTERN_PRESETS[presetName as keyof typeof PATTERN_PRESETS];
  if (preset) {
    patterns.value = preset();
  }
}

// Actions
function handleConfirm() {
  if (!validation.value.isValid || changesCount.value === 0) return;

  const config: BatchRenameConfig = {
    patterns: patterns.value,
    applyToFolders: applyToFolders.value,
    applyToFiles: applyToFiles.value,
    preserveExtension: preserveExtension.value,
  };

  emit('confirm', config);
}

function handleCancel() {
  emit('cancel');
}

// Reset when dialog opens
watch(
  () => props.isOpen,
  (isOpen) => {
    if (isOpen) {
      patterns.value = [];
      applyToFolders.value = true;
      applyToFiles.value = true;
      preserveExtension.value = true;
      activeTab.value = 'patterns';
    }
  }
);
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/30 flex items-center justify-center z-50"
    @click.self="handleCancel"
  >
    <div class="bg-white bg-(--window-bg) border border-(--border) rounded-lg shadow-xl w-[900px] max-h-[80vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-(--border)">
        <h2 class="text-lg font-semibold">Batch Rename ({{ files.length }} items)</h2>
        <button
          @click="handleCancel"
          class="text-gray-500 hover:text-gray-700 text-xl leading-none"
        >
          ×
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-[var(--border)]">
        <button
          @click="activeTab = 'patterns'"
          :class="[
            'px-4 py-2 font-medium transition-colors',
            activeTab === 'patterns'
              ? 'border-b-2 border-blue-500 text-blue-600'
              : 'text-gray-600 hover:text-gray-800',
          ]"
        >
          Patterns
        </button>
        <button
          @click="activeTab = 'preview'"
          :class="[
            'px-4 py-2 font-medium transition-colors',
            activeTab === 'preview'
              ? 'border-b-2 border-blue-500 text-blue-600'
              : 'text-gray-600 hover:text-gray-800',
          ]"
        >
          Preview ({{ changesCount }} changes)
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-auto p-4">
        <!-- Patterns Tab -->
        <div v-if="activeTab === 'patterns'" class="space-y-4">
          <!-- Presets -->
          <div>
            <label class="block text-sm font-medium mb-2">Quick Presets</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="(preset, name) in PATTERN_PRESETS"
                :key="name"
                @click="applyPreset(name)"
                class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded border border-gray-300"
              >
                {{ name }}
              </button>
            </div>
          </div>

          <!-- Options -->
          <div class="grid grid-cols-3 gap-4 p-3 bg-gray-50 rounded">
            <label class="flex items-center space-x-2">
              <input type="checkbox" v-model="applyToFiles" class="rounded" />
              <span class="text-sm">Apply to files</span>
            </label>
            <label class="flex items-center space-x-2">
              <input type="checkbox" v-model="applyToFolders" class="rounded" />
              <span class="text-sm">Apply to folders</span>
            </label>
            <label class="flex items-center space-x-2">
              <input type="checkbox" v-model="preserveExtension" class="rounded" />
              <span class="text-sm">Preserve extension</span>
            </label>
          </div>

          <!-- Add Pattern Buttons -->
          <div>
            <label class="block text-sm font-medium mb-2">Add Pattern</label>
            <div class="flex flex-wrap gap-2">
              <button
                @click="addPattern('prefix')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Prefix
              </button>
              <button
                @click="addPattern('suffix')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Suffix
              </button>
              <button
                @click="addPattern('replace')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Replace
              </button>
              <button
                @click="addPattern('regex')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Regex
              </button>
              <button
                @click="addPattern('numbering')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Numbering
              </button>
              <button
                @click="addPattern('case')"
                class="px-3 py-1 text-sm bg-blue-500 text-white hover:bg-blue-600 rounded"
              >
                + Case Change
              </button>
            </div>
          </div>

          <!-- Patterns List -->
          <div class="space-y-3">
            <div
              v-for="(pattern, index) in patterns"
              :key="index"
              class="p-3 border border-gray-300 rounded bg-white"
            >
              <div class="flex items-start justify-between mb-2">
                <label class="flex items-center space-x-2">
                  <input type="checkbox" v-model="pattern.enabled" class="rounded" />
                  <span class="font-medium capitalize">{{ pattern.type }}</span>
                </label>
                <div class="flex space-x-1">
                  <button
                    @click="movePatternUp(index)"
                    :disabled="index === 0"
                    class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-50"
                  >
                    ↑
                  </button>
                  <button
                    @click="movePatternDown(index)"
                    :disabled="index === patterns.length - 1"
                    class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-50"
                  >
                    ↓
                  </button>
                  <button
                    @click="removePattern(index)"
                    class="px-2 py-1 text-xs bg-red-100 hover:bg-red-200 text-red-700 rounded"
                  >
                    Remove
                  </button>
                </div>
              </div>

              <!-- Prefix Pattern -->
              <div v-if="pattern.type === 'prefix'" class="space-y-2">
                <input
                  type="text"
                  v-model="(pattern as PrefixPattern).text"
                  placeholder="Enter prefix text..."
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                />
              </div>

              <!-- Suffix Pattern -->
              <div v-else-if="pattern.type === 'suffix'" class="space-y-2">
                <input
                  type="text"
                  v-model="(pattern as SuffixPattern).text"
                  placeholder="Enter suffix text..."
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                />
                <label class="flex items-center space-x-2 text-sm">
                  <input
                    type="checkbox"
                    v-model="(pattern as SuffixPattern).beforeExtension"
                    class="rounded"
                  />
                  <span>Add before file extension</span>
                </label>
              </div>

              <!-- Replace Pattern -->
              <div v-else-if="pattern.type === 'replace'" class="space-y-2">
                <input
                  type="text"
                  v-model="(pattern as ReplacePattern).searchText"
                  placeholder="Search for..."
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                />
                <input
                  type="text"
                  v-model="(pattern as ReplacePattern).replaceText"
                  placeholder="Replace with..."
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                />
                <div class="flex space-x-4 text-sm">
                  <label class="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      v-model="(pattern as ReplacePattern).caseSensitive"
                      class="rounded"
                    />
                    <span>Case sensitive</span>
                  </label>
                  <label class="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      v-model="(pattern as ReplacePattern).wholeWord"
                      class="rounded"
                    />
                    <span>Whole word</span>
                  </label>
                </div>
              </div>

              <!-- Regex Pattern -->
              <div v-else-if="pattern.type === 'regex'" class="space-y-2">
                <input
                  type="text"
                  v-model="(pattern as RegexPattern).pattern"
                  placeholder="Regex pattern..."
                  class="w-full px-3 py-2 border border-gray-300 rounded font-mono text-sm"
                />
                <input
                  type="text"
                  v-model="(pattern as RegexPattern).replacement"
                  placeholder="Replacement..."
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                />
                <input
                  type="text"
                  v-model="(pattern as RegexPattern).flags"
                  placeholder="Flags (g, i, gi, etc.)"
                  class="w-32 px-3 py-2 border border-gray-300 rounded text-sm"
                />
              </div>

              <!-- Numbering Pattern -->
              <div v-else-if="pattern.type === 'numbering'" class="space-y-2">
                <div class="grid grid-cols-4 gap-2">
                  <div>
                    <label class="block text-xs mb-1">Start</label>
                    <input
                      type="number"
                      v-model.number="(pattern as NumberingPattern).startNumber"
                      class="w-full px-2 py-1 border border-gray-300 rounded"
                    />
                  </div>
                  <div>
                    <label class="block text-xs mb-1">Increment</label>
                    <input
                      type="number"
                      v-model.number="(pattern as NumberingPattern).increment"
                      class="w-full px-2 py-1 border border-gray-300 rounded"
                    />
                  </div>
                  <div>
                    <label class="block text-xs mb-1">Padding</label>
                    <input
                      type="number"
                      v-model.number="(pattern as NumberingPattern).padding"
                      class="w-full px-2 py-1 border border-gray-300 rounded"
                    />
                  </div>
                  <div>
                    <label class="block text-xs mb-1">Separator</label>
                    <input
                      type="text"
                      v-model="(pattern as NumberingPattern).separator"
                      class="w-full px-2 py-1 border border-gray-300 rounded"
                    />
                  </div>
                </div>
                <select
                  v-model="(pattern as NumberingPattern).position"
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                >
                  <option value="prefix">Prefix</option>
                  <option value="suffix">Suffix</option>
                  <option value="replace">Replace entire name</option>
                </select>
              </div>

              <!-- Case Pattern -->
              <div v-else-if="pattern.type === 'case'" class="space-y-2">
                <select
                  v-model="(pattern as CasePattern).caseType"
                  class="w-full px-3 py-2 border border-gray-300 rounded"
                >
                  <option value="uppercase">UPPERCASE</option>
                  <option value="lowercase">lowercase</option>
                  <option value="titlecase">Title Case</option>
                  <option value="camelcase">camelCase</option>
                  <option value="snakecase">snake_case</option>
                  <option value="kebabcase">kebab-case</option>
                </select>
              </div>
            </div>

            <div v-if="patterns.length === 0" class="text-center py-8 text-gray-500">
              No patterns added. Click a button above to add a pattern.
            </div>
          </div>
        </div>

        <!-- Preview Tab -->
        <div v-else-if="activeTab === 'preview'">
          <!-- Validation Errors -->
          <div v-if="validation.errors.length > 0" class="mb-4 p-3 bg-red-50 border border-red-300 rounded">
            <div class="font-semibold text-red-700 mb-2">Errors:</div>
            <ul class="list-disc list-inside space-y-1 text-sm text-red-600">
              <li v-for="(error, index) in validation.errors" :key="index">
                {{ error.error }}
              </li>
            </ul>
          </div>

          <!-- Validation Warnings -->
          <div v-if="validation.warnings.length > 0" class="mb-4 p-3 bg-yellow-50 border border-yellow-300 rounded">
            <div class="font-semibold text-yellow-700 mb-2">Warnings:</div>
            <ul class="list-disc list-inside space-y-1 text-sm text-yellow-600">
              <li v-for="(warning, index) in validation.warnings" :key="index">
                {{ warning.error }}
              </li>
            </ul>
          </div>

          <!-- Preview Table -->
          <div class="overflow-auto max-h-[400px] border border-gray-300 rounded">
            <table class="w-full text-sm">
              <thead class="bg-gray-100 sticky top-0">
                <tr>
                  <th class="px-3 py-2 text-left">Original Name</th>
                  <th class="px-3 py-2 text-center">→</th>
                  <th class="px-3 py-2 text-left">New Name</th>
                  <th class="px-3 py-2 text-left">Status</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(item, index) in preview"
                  :key="index"
                  :class="[
                    'border-t border-gray-200',
                    item.hasError ? 'bg-red-50' : item.originalName === item.newName ? 'bg-gray-50' : 'bg-green-50',
                  ]"
                >
                  <td class="px-3 py-2">{{ item.originalName }}</td>
                  <td class="px-3 py-2 text-center text-gray-400">→</td>
                  <td class="px-3 py-2 font-medium">{{ item.newName }}</td>
                  <td class="px-3 py-2 text-xs">
                    <span
                      v-if="item.hasError"
                      class="text-red-600"
                      :title="item.errorMessage"
                    >
                      Error
                    </span>
                    <span v-else-if="item.originalName === item.newName" class="text-gray-500">
                      No change
                    </span>
                    <span v-else class="text-green-600">Will rename</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between p-4 border-t border-[var(--border)]">
        <div class="text-sm text-gray-600">
          {{ changesCount }} of {{ files.length }} items will be renamed
        </div>
        <div class="flex space-x-2">
          <button
            @click="handleCancel"
            class="px-4 py-2 border border-gray-300 rounded hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            @click="handleConfirm"
            :disabled="!validation.isValid || changesCount === 0"
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Rename
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
