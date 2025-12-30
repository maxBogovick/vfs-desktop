<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type {
  FileItem,
  BatchAttributeChange,
  PermissionsChange,
  DateChange,
  TagsChange,
} from '../types';

interface Props {
  isOpen: boolean;
  files: FileItem[];
}

interface Emits {
  (e: 'confirm', changes: BatchAttributeChange): void;
  (e: 'cancel'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// State
const activeTab = ref<'permissions' | 'dates' | 'tags'>('permissions');

// Permissions
const changePermissions = ref(false);
const readable = ref<boolean | null>(null);
const writable = ref<boolean | null>(null);
const executable = ref<boolean | null>(null);
const recursive = ref(false);

// Dates
const changeDates = ref(false);
const modifiedDate = ref('');
const modifiedTime = ref('');
const createdDate = ref('');
const createdTime = ref('');

// Tags
const changeTags = ref(false);
const tagsOperation = ref<'add' | 'remove' | 'replace'>('add');
const tagsInput = ref('');

// Computed
const tags = computed(() => {
  return tagsInput.value
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0);
});

const hasChanges = computed(() => {
  if (changePermissions.value && (readable.value !== null || writable.value !== null || executable.value !== null)) {
    return true;
  }
  if (changeDates.value && (modifiedDate.value || createdDate.value)) {
    return true;
  }
  if (changeTags.value && tags.value.length > 0) {
    return true;
  }
  return false;
});

const filesCount = computed(() => props.files.length);
const foldersCount = computed(() => props.files.filter((f) => f.type === 'folder').length);

// Actions
function handleConfirm() {
  if (!hasChanges.value) return;

  const changes: BatchAttributeChange = {};

  // Permissions
  if (changePermissions.value) {
    changes.permissions = {
      readable: readable.value !== null ? readable.value : undefined,
      writable: writable.value !== null ? writable.value : undefined,
      executable: executable.value !== null ? executable.value : undefined,
      recursive: recursive.value,
    };
  }

  // Dates
  if (changeDates.value) {
    const dateChanges: DateChange = {};

    if (modifiedDate.value) {
      const datetime = new Date(`${modifiedDate.value}T${modifiedTime.value || '00:00:00'}`);
      dateChanges.modified = Math.floor(datetime.getTime() / 1000);
    }

    if (createdDate.value) {
      const datetime = new Date(`${createdDate.value}T${createdTime.value || '00:00:00'}`);
      dateChanges.created = Math.floor(datetime.getTime() / 1000);
    }

    if (Object.keys(dateChanges).length > 0) {
      changes.dates = dateChanges;
    }
  }

  // Tags
  if (changeTags.value && tags.value.length > 0) {
    changes.tags = {
      operation: tagsOperation.value,
      tags: tags.value,
    };
  }

  emit('confirm', changes);
}

function handleCancel() {
  emit('cancel');
}

function setPermission(type: 'readable' | 'writable' | 'executable', value: boolean | null) {
  switch (type) {
    case 'readable':
      readable.value = value;
      break;
    case 'writable':
      writable.value = value;
      break;
    case 'executable':
      executable.value = value;
      break;
  }
}

function setCurrentDate(field: 'modified' | 'created') {
  const now = new Date();
  const date = now.toISOString().split('T')[0];
  const time = now.toTimeString().split(' ')[0].substring(0, 5);

  if (field === 'modified') {
    modifiedDate.value = date;
    modifiedTime.value = time;
  } else {
    createdDate.value = date;
    createdTime.value = time;
  }
}

// Reset when dialog opens
watch(
  () => props.isOpen,
  (isOpen) => {
    if (isOpen) {
      activeTab.value = 'permissions';
      changePermissions.value = false;
      readable.value = null;
      writable.value = null;
      executable.value = null;
      recursive.value = false;
      changeDates.value = false;
      modifiedDate.value = '';
      modifiedTime.value = '';
      createdDate.value = '';
      createdTime.value = '';
      changeTags.value = false;
      tagsOperation.value = 'add';
      tagsInput.value = '';
    }
  }
);
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click.self="handleCancel"
  >
    <div class="bg-white border border-[var(--border)] rounded-lg shadow-xl w-[600px] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--border)]">
        <h2 class="text-lg font-semibold">
          Batch Change Attributes
          <span class="text-sm font-normal text-gray-600">
            ({{ filesCount }} items{{ foldersCount > 0 ? `, ${foldersCount} folders` : '' }})
          </span>
        </h2>
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
          @click="activeTab = 'permissions'"
          :class="[
            'px-4 py-2 font-medium transition-colors',
            activeTab === 'permissions'
              ? 'border-b-2 border-blue-500 text-blue-600'
              : 'text-gray-600 hover:text-gray-800',
          ]"
        >
          Permissions
        </button>
        <button
          @click="activeTab = 'dates'"
          :class="[
            'px-4 py-2 font-medium transition-colors',
            activeTab === 'dates'
              ? 'border-b-2 border-blue-500 text-blue-600'
              : 'text-gray-600 hover:text-gray-800',
          ]"
        >
          Dates
        </button>
        <button
          @click="activeTab = 'tags'"
          :class="[
            'px-4 py-2 font-medium transition-colors',
            activeTab === 'tags'
              ? 'border-b-2 border-blue-500 text-blue-600'
              : 'text-gray-600 hover:text-gray-800',
          ]"
        >
          Tags
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 min-h-[300px]">
        <!-- Permissions Tab -->
        <div v-if="activeTab === 'permissions'" class="space-y-4">
          <label class="flex items-center space-x-2">
            <input type="checkbox" v-model="changePermissions" class="rounded" />
            <span class="font-medium">Change permissions</span>
          </label>

          <div v-if="changePermissions" class="space-y-4 pl-6">
            <!-- Readable -->
            <div>
              <label class="block text-sm font-medium mb-2">Readable</label>
              <div class="flex space-x-2">
                <button
                  @click="setPermission('readable', true)"
                  :class="[
                    'px-4 py-2 rounded border',
                    readable === true
                      ? 'bg-green-500 text-white border-green-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  Yes
                </button>
                <button
                  @click="setPermission('readable', false)"
                  :class="[
                    'px-4 py-2 rounded border',
                    readable === false
                      ? 'bg-red-500 text-white border-red-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No
                </button>
                <button
                  @click="setPermission('readable', null)"
                  :class="[
                    'px-4 py-2 rounded border',
                    readable === null
                      ? 'bg-gray-300 border-gray-400'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No change
                </button>
              </div>
            </div>

            <!-- Writable -->
            <div>
              <label class="block text-sm font-medium mb-2">Writable</label>
              <div class="flex space-x-2">
                <button
                  @click="setPermission('writable', true)"
                  :class="[
                    'px-4 py-2 rounded border',
                    writable === true
                      ? 'bg-green-500 text-white border-green-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  Yes
                </button>
                <button
                  @click="setPermission('writable', false)"
                  :class="[
                    'px-4 py-2 rounded border',
                    writable === false
                      ? 'bg-red-500 text-white border-red-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No
                </button>
                <button
                  @click="setPermission('writable', null)"
                  :class="[
                    'px-4 py-2 rounded border',
                    writable === null
                      ? 'bg-gray-300 border-gray-400'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No change
                </button>
              </div>
            </div>

            <!-- Executable -->
            <div>
              <label class="block text-sm font-medium mb-2">Executable</label>
              <div class="flex space-x-2">
                <button
                  @click="setPermission('executable', true)"
                  :class="[
                    'px-4 py-2 rounded border',
                    executable === true
                      ? 'bg-green-500 text-white border-green-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  Yes
                </button>
                <button
                  @click="setPermission('executable', false)"
                  :class="[
                    'px-4 py-2 rounded border',
                    executable === false
                      ? 'bg-red-500 text-white border-red-600'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No
                </button>
                <button
                  @click="setPermission('executable', null)"
                  :class="[
                    'px-4 py-2 rounded border',
                    executable === null
                      ? 'bg-gray-300 border-gray-400'
                      : 'bg-white border-gray-300 hover:bg-gray-50',
                  ]"
                >
                  No change
                </button>
              </div>
            </div>

            <!-- Recursive -->
            <label v-if="foldersCount > 0" class="flex items-center space-x-2">
              <input type="checkbox" v-model="recursive" class="rounded" />
              <span class="text-sm">Apply recursively to subdirectories</span>
            </label>
          </div>
        </div>

        <!-- Dates Tab -->
        <div v-else-if="activeTab === 'dates'" class="space-y-4">
          <label class="flex items-center space-x-2">
            <input type="checkbox" v-model="changeDates" class="rounded" />
            <span class="font-medium">Change dates</span>
          </label>

          <div v-if="changeDates" class="space-y-4 pl-6">
            <!-- Modified Date -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm font-medium">Modified Date</label>
                <button
                  @click="setCurrentDate('modified')"
                  class="text-xs text-blue-600 hover:text-blue-700"
                >
                  Set to now
                </button>
              </div>
              <div class="flex space-x-2">
                <input
                  type="date"
                  v-model="modifiedDate"
                  class="flex-1 px-3 py-2 border border-gray-300 rounded"
                />
                <input
                  type="time"
                  v-model="modifiedTime"
                  class="w-32 px-3 py-2 border border-gray-300 rounded"
                />
              </div>
            </div>

            <!-- Created Date -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm font-medium">Created Date</label>
                <button
                  @click="setCurrentDate('created')"
                  class="text-xs text-blue-600 hover:text-blue-700"
                >
                  Set to now
                </button>
              </div>
              <div class="flex space-x-2">
                <input
                  type="date"
                  v-model="createdDate"
                  class="flex-1 px-3 py-2 border border-gray-300 rounded"
                />
                <input
                  type="time"
                  v-model="createdTime"
                  class="w-32 px-3 py-2 border border-gray-300 rounded"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Tags Tab -->
        <div v-else-if="activeTab === 'tags'" class="space-y-4">
          <label class="flex items-center space-x-2">
            <input type="checkbox" v-model="changeTags" class="rounded" />
            <span class="font-medium">Change tags</span>
          </label>

          <div v-if="changeTags" class="space-y-4 pl-6">
            <div>
              <label class="block text-sm font-medium mb-2">Operation</label>
              <select
                v-model="tagsOperation"
                class="w-full px-3 py-2 border border-gray-300 rounded"
              >
                <option value="add">Add tags</option>
                <option value="remove">Remove tags</option>
                <option value="replace">Replace all tags</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium mb-2">Tags (comma-separated)</label>
              <input
                type="text"
                v-model="tagsInput"
                placeholder="work, important, project"
                class="w-full px-3 py-2 border border-gray-300 rounded"
              />
              <div v-if="tags.length > 0" class="mt-2 flex flex-wrap gap-2">
                <span
                  v-for="tag in tags"
                  :key="tag"
                  class="px-2 py-1 text-xs bg-blue-100 text-blue-700 rounded"
                >
                  {{ tag }}
                </span>
              </div>
            </div>

            <div class="text-xs text-gray-600">
              Note: Tag functionality is currently supported on macOS only.
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between p-4 border-t border-[var(--border)]">
        <div class="text-sm text-gray-600">
          Changes will be applied to {{ filesCount }} item{{ filesCount !== 1 ? 's' : '' }}
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
            :disabled="!hasChanges"
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Apply Changes
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
