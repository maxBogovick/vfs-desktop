<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useProgrammerMode } from '../composables/useProgrammerMode'
import { useTemplates } from '../composables/useTemplates'
import type { FileTemplate } from '../types'

interface Props {
  isOpen: boolean
  currentPath: string[]
  mode?: 'file' | 'folder'
}

interface Emits {
  (e: 'create', payload: { name: string; isFolder: boolean; templateId?: string }): void
  (e: 'batch-create', names: string[]): void
  (e: 'cancel'): void
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'file',
})

const emit = defineEmits<Emits>()

// Composables
const { isProgrammerMode, enableBatchCreate, enableSmartDefaults } = useProgrammerMode()
const {
  templates,
  loadTemplates,
  getContextualTemplates,
  findTemplateByFilename,
  suggestExtension,
  selectedTemplate,
  selectTemplate,
} = useTemplates()

// Local state
const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const creationMode = ref<'file' | 'folder'>('file')
const showTemplateDropdown = ref(false)
const suggestedExtension = ref<string | null>(null)

// Computed
const currentPathString = computed(() => '/' + props.currentPath.join('/'))

const contextualTemplates = computed(() => {
  if (!isProgrammerMode.value) return []
  return getContextualTemplates(currentPathString.value)
})

const displayIcon = computed(() => {
  return creationMode.value === 'folder' ? '📁' : '📄'
})

const placeholder = computed(() => {
  if (creationMode.value === 'folder') {
    return 'New Folder'
  }

  if (suggestedExtension.value && enableSmartDefaults.value) {
    return `New file (suggested: ${suggestedExtension.value})`
  }

  if (enableBatchCreate.value) {
    return 'filename.ext or file1.txt, file2.txt, ...'
  }

  return 'filename.ext'
})

// Watchers
watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    inputValue.value = ''
    creationMode.value = props.mode
    showTemplateDropdown.value = false
    selectTemplate(null)

    // Load templates if programmer mode
    if (isProgrammerMode.value && templates.value.length === 0) {
      await loadTemplates()
    }

    // Suggest extension if enabled
    if (enableSmartDefaults.value && creationMode.value === 'file') {
      suggestedExtension.value = await suggestExtension(currentPathString.value)
    }

    await nextTick()
    inputRef.value?.focus()
  }
})

watch(inputValue, (newValue) => {
  if (!isProgrammerMode.value || creationMode.value === 'folder') return

  // Auto-detect template based on filename
  if (newValue.includes('.')) {
    const template = findTemplateByFilename(newValue)
    if (template) {
      selectTemplate(template)
    }
  }
})

// Methods
const handleConfirm = () => {
  const value = inputValue.value.trim()
  if (!value) return

  // Check for batch create (comma-separated)
  if (enableBatchCreate.value && value.includes(',')) {
    const names = value.split(',').map(n => n.trim()).filter(n => n.length > 0)
    if (names.length > 1) {
      emit('batch-create', names)
      return
    }
  }

  // Apply smart defaults
  let finalName = value
  if (
    enableSmartDefaults.value &&
    creationMode.value === 'file' &&
    !value.includes('.') &&
    suggestedExtension.value
  ) {
    finalName = value + suggestedExtension.value
  }

  emit('create', {
    name: finalName,
    isFolder: creationMode.value === 'folder',
    templateId: selectedTemplate.value?.id,
  })
}

const handleCancel = () => {
  emit('cancel')
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    event.preventDefault()
    handleConfirm()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    handleCancel()
  } else if (event.key === 'Tab') {
    event.preventDefault()
    // Toggle between file and folder
    creationMode.value = creationMode.value === 'file' ? 'folder' : 'file'
  }
}

const handleTemplateSelect = (template: FileTemplate) => {
  selectTemplate(template)
  showTemplateDropdown.value = false

  // Auto-fill extension if needed
  if (template.fileExtensions.length > 0 && !inputValue.value.includes('.')) {
    const ext = template.fileExtensions[0]
    if (ext.startsWith('.')) {
      // Suggest the extension
      suggestedExtension.value = ext
    }
  }
}

const toggleTemplateDropdown = () => {
  showTemplateDropdown.value = !showTemplateDropdown.value
}

onMounted(() => {
  // Load templates on mount if programmer mode
  if (isProgrammerMode.value) {
    loadTemplates()
  }
})
</script>

<template>
  <div
    v-if="isOpen"
    class="inline-file-creator bg-white border border-[#7F9DB9] px-2 py-1 flex items-center gap-2"
  >
    <!-- Icon -->
    <span class="text-base">{{ displayIcon }}</span>

    <!-- Input -->
    <input
      ref="inputRef"
      v-model="inputValue"
      type="text"
      :placeholder="placeholder"
      @keydown="handleKeydown"
      class="flex-1 px-1 py-0.5 text-[11px] border border-[#7F9DB9] focus:outline-none focus:border-[#0A246A]"
    />

    <!-- Template selector button (Programmer Mode only) -->
    <button
      v-if="isProgrammerMode && creationMode === 'file'"
      @click="toggleTemplateDropdown"
      class="px-2 py-0.5 text-[10px] bg-[#ECE9D8] border border-[#7F9DB9] hover:bg-[#DDD]"
      title="Choose template"
    >
      📋
    </button>

    <!-- Mode indicator -->
    <span class="text-[9px] text-gray-500">
      {{ creationMode === 'file' ? 'File' : 'Folder' }} • Tab to switch
    </span>

    <!-- Template dropdown -->
    <div
      v-if="showTemplateDropdown && contextualTemplates.length > 0"
      class="absolute mt-1 bg-white border border-[#919B9C] shadow-lg z-50 max-h-60 overflow-y-auto"
      style="top: 100%; left: 0; min-width: 300px"
    >
      <!-- Contextual templates section -->
      <div class="border-b border-[#D0D0BF] p-2">
        <div class="text-[9px] text-gray-500 mb-1">Suggested for this folder:</div>
        <div
          v-for="template in contextualTemplates.slice(0, 5)"
          :key="template.id"
          @click="handleTemplateSelect(template)"
          class="px-2 py-1 text-[11px] hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
        >
          <span>{{ template.icon }}</span>
          <span class="flex-1">{{ template.name }}</span>
          <span class="text-[9px] text-gray-400">{{ template.category }}</span>
        </div>
      </div>

      <!-- All templates -->
      <div class="p-2">
        <div class="text-[9px] text-gray-500 mb-1">All templates:</div>
        <div
          v-for="template in templates"
          :key="template.id"
          @click="handleTemplateSelect(template)"
          class="px-2 py-1 text-[11px] hover:bg-[#C1D2EE] cursor-pointer flex items-center gap-2"
          :class="{ 'bg-[#C1D2EE]': selectedTemplate?.id === template.id }"
        >
          <span>{{ template.icon }}</span>
          <span class="flex-1">{{ template.name }}</span>
          <span class="text-[9px] text-gray-400">{{ template.category }}</span>
        </div>
      </div>
    </div>

    <!-- Selected template indicator -->
    <div
      v-if="selectedTemplate"
      class="text-[9px] text-blue-600 flex items-center gap-1"
    >
      <span>{{ selectedTemplate.icon }}</span>
      <span>{{ selectedTemplate.name }}</span>
    </div>
  </div>
</template>

<style scoped>
.inline-file-creator {
  position: relative;
  animation: slideDown 0.15s ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
