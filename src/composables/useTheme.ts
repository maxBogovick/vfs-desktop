/**
 * Theme Composable
 * Manages theme state and applies theme colors to the application
 */

import { ref, onMounted, watch } from 'vue'
import { THEMES, type ThemeName, type Theme } from '../types/theme'

const currentTheme = ref<ThemeName>('luna')
const isInitialized = ref(false)

export function useTheme() {
  /**
   * Apply theme by setting CSS variables
   */
  const applyTheme = (themeName: ThemeName) => {
    const theme: Theme = THEMES[themeName]
    const root = document.documentElement

    // Set CSS variables
    root.style.setProperty('--vf-bg-primary', theme.colors.bgPrimary)
    root.style.setProperty('--vf-bg-secondary', theme.colors.bgSecondary)
    root.style.setProperty('--vf-bg-tertiary', theme.colors.bgTertiary)

    root.style.setProperty('--vf-surface-default', theme.colors.surfaceDefault)
    root.style.setProperty('--vf-surface-hover', theme.colors.surfaceHover)
    root.style.setProperty('--vf-surface-selected', theme.colors.surfaceSelected)
    root.style.setProperty('--vf-surface-active', theme.colors.surfaceActive)

    root.style.setProperty('--vf-text-primary', theme.colors.textPrimary)
    root.style.setProperty('--vf-text-secondary', theme.colors.textSecondary)
    root.style.setProperty('--vf-text-tertiary', theme.colors.textTertiary)
    root.style.setProperty('--vf-text-disabled', theme.colors.textDisabled)

    root.style.setProperty('--vf-accent-primary', theme.colors.accentPrimary)
    root.style.setProperty('--vf-accent-hover', theme.colors.accentHover)
    root.style.setProperty('--vf-accent-active', theme.colors.accentActive)

    root.style.setProperty('--vf-border-default', theme.colors.borderDefault)
    root.style.setProperty('--vf-border-subtle', theme.colors.borderSubtle)
    root.style.setProperty('--vf-border-accent', theme.colors.borderAccent)

    root.style.setProperty('--vf-shadow-sm', theme.colors.shadowSm)
    root.style.setProperty('--vf-shadow-md', theme.colors.shadowMd)
    root.style.setProperty('--vf-shadow-lg', theme.colors.shadowLg)

    root.style.setProperty('--vf-gradient-from', theme.colors.gradient.from)
    root.style.setProperty('--vf-gradient-to', theme.colors.gradient.to)

    // Set data attribute for theme-specific styles
    root.setAttribute('data-theme', themeName)

    console.log(`[useTheme] Applied theme: ${themeName}`)
  }

  /**
   * Load theme from backend config
   */
  const loadTheme = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const config = await invoke<{ theme: string }>('get_config')

      if (config && config.theme) {
        // Validate theme name
        const themeName = config.theme as ThemeName
        if (THEMES[themeName]) {
          currentTheme.value = themeName
          applyTheme(themeName)
          console.log(`[useTheme] Loaded theme from config: ${themeName}`)
        } else {
          console.warn(`[useTheme] Invalid theme in config: ${config.theme}, using default`)
          currentTheme.value = 'luna'
          applyTheme('luna')
        }
      } else {
        console.log('[useTheme] No theme in config, using default: luna')
        currentTheme.value = 'luna'
        applyTheme('luna')
      }

      isInitialized.value = true
    } catch (error) {
      console.error('[useTheme] Failed to load theme:', error)
      // Fallback to default theme
      currentTheme.value = 'luna'
      applyTheme('luna')
      isInitialized.value = true
    }
  }

  /**
   * Save theme to backend config
   */
  const saveTheme = async (themeName: ThemeName) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')

      // Get current config
      const config = await invoke<any>('get_config')

      // Update theme
      config.theme = themeName

      // Save config
      await invoke('update_config', { newConfig: config })

      console.log(`[useTheme] Saved theme to config: ${themeName}`)
    } catch (error) {
      console.error('[useTheme] Failed to save theme:', error)
      throw error
    }
  }

  /**
   * Set theme (apply + save)
   */
  const setTheme = async (themeName: ThemeName) => {
    if (!THEMES[themeName]) {
      console.error(`[useTheme] Invalid theme name: ${themeName}`)
      return
    }

    currentTheme.value = themeName
    applyTheme(themeName)

    // Save to backend
    try {
      await saveTheme(themeName)
    } catch (error) {
      console.error('[useTheme] Failed to save theme, but applied it anyway')
    }
  }

  /**
   * Get current theme object
   */
  const getTheme = () => {
    return THEMES[currentTheme.value]
  }

  /**
   * Get all available themes
   */
  const getAllThemes = () => {
    return Object.values(THEMES)
  }

  // Watch for theme changes and apply
  watch(currentTheme, (newTheme) => {
    if (isInitialized.value) {
      applyTheme(newTheme)
    }
  })

  return {
    currentTheme,
    isInitialized,
    applyTheme,
    loadTheme,
    saveTheme,
    setTheme,
    getTheme,
    getAllThemes,
  }
}
