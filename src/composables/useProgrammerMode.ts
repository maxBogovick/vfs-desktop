import { ref, computed, watch } from 'vue'

const STORAGE_KEY = 'vfdir_programmer_mode'

// Глобальное состояние режима программиста
const isProgrammerMode = ref(false)

// Инициализация из localStorage
const init = () => {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored !== null) {
    isProgrammerMode.value = stored === 'true'
  }
}

// Сохранение в localStorage при изменении
watch(isProgrammerMode, (value) => {
  localStorage.setItem(STORAGE_KEY, value.toString())
})

export function useProgrammerMode() {
  // Инициализация при первом использовании
  if (!isProgrammerMode.value && localStorage.getItem(STORAGE_KEY) === 'true') {
    init()
  }

  // Computed свойства для фич
  const showTemplates = computed(() => isProgrammerMode.value)
  const showPreview = computed(() => isProgrammerMode.value)
  const enableBatchCreate = computed(() => isProgrammerMode.value)
  const enableSmartDefaults = computed(() => isProgrammerMode.value)

  const toggleProgrammerMode = () => {
    isProgrammerMode.value = !isProgrammerMode.value
  }

  const enableProgrammerMode = () => {
    isProgrammerMode.value = true
  }

  const disableProgrammerMode = () => {
    isProgrammerMode.value = false
  }

  return {
    isProgrammerMode,
    showTemplates,
    showPreview,
    enableBatchCreate,
    enableSmartDefaults,
    toggleProgrammerMode,
    enableProgrammerMode,
    disableProgrammerMode,
  }
}
