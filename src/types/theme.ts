/**
 * Theme System Types
 * Defines all theme-related types for VFDir
 */

export type ThemeName = 'luna' | 'classic' | 'royale' | 'silver' | 'dark'

export interface ThemeColors {
  // Background colors
  bgPrimary: string
  bgSecondary: string
  bgTertiary: string

  // Surface colors (for cards, panels, etc.)
  surfaceDefault: string
  surfaceHover: string
  surfaceSelected: string
  surfaceActive: string

  // Text colors
  textPrimary: string
  textSecondary: string
  textTertiary: string
  textDisabled: string

  // Accent colors
  accentPrimary: string
  accentHover: string
  accentActive: string

  // Border colors
  borderDefault: string
  borderSubtle: string
  borderAccent: string

  // Shadow colors
  shadowSm: string
  shadowMd: string
  shadowLg: string

  // Special colors
  gradient: {
    from: string
    to: string
  }
}

export interface Theme {
  name: ThemeName
  displayName: string
  description: string
  colors: ThemeColors
}

export const THEMES: Record<ThemeName, Theme> = {
  luna: {
    name: 'luna',
    displayName: 'Luna (Default)',
    description: 'Classic Windows XP blue theme',
    colors: {
      bgPrimary: '#ECE9D8',
      bgSecondary: '#F1EFE2',
      bgTertiary: '#E3DED4',

      surfaceDefault: '#FFFFFF',
      surfaceHover: '#C1D2EE',
      surfaceSelected: '#C1D2EE',
      surfaceActive: '#0A246A',

      textPrimary: '#0B0B0B',
      textSecondary: '#555555',
      textTertiary: '#666666',
      textDisabled: '#999999',

      accentPrimary: '#0054E3',
      accentHover: '#0A246A',
      accentActive: '#001A4D',

      borderDefault: '#919B9C',
      borderSubtle: '#C0C0C0',
      borderAccent: '#0054E3',

      shadowSm: 'rgba(0, 0, 0, 0.1)',
      shadowMd: 'rgba(0, 0, 0, 0.15)',
      shadowLg: 'rgba(0, 0, 0, 0.2)',

      gradient: {
        from: '#0054E3',
        to: '#0A246A',
      },
    },
  },

  classic: {
    name: 'classic',
    displayName: 'Classic',
    description: 'Windows 95/98 gray theme',
    colors: {
      bgPrimary: '#C0C0C0',
      bgSecondary: '#D4D0C8',
      bgTertiary: '#B0B0B0',

      surfaceDefault: '#FFFFFF',
      surfaceHover: '#000080',
      surfaceSelected: '#000080',
      surfaceActive: '#000080',

      textPrimary: '#000000',
      textSecondary: '#404040',
      textTertiary: '#666666',
      textDisabled: '#808080',

      accentPrimary: '#000080',
      accentHover: '#000060',
      accentActive: '#000040',

      borderDefault: '#808080',
      borderSubtle: '#A0A0A0',
      borderAccent: '#000080',

      shadowSm: 'rgba(0, 0, 0, 0.1)',
      shadowMd: 'rgba(0, 0, 0, 0.15)',
      shadowLg: 'rgba(0, 0, 0, 0.2)',

      gradient: {
        from: '#000080',
        to: '#000060',
      },
    },
  },

  royale: {
    name: 'royale',
    displayName: 'Royale',
    description: 'Windows XP Media Center Edition theme',
    colors: {
      bgPrimary: '#EBE8D7',
      bgSecondary: '#F2EFE0',
      bgTertiary: '#DDD9C8',

      surfaceDefault: '#FFFFFF',
      surfaceHover: '#B6BDD2',
      surfaceSelected: '#B6BDD2',
      surfaceActive: '#4E5F8C',

      textPrimary: '#0B0B0B',
      textSecondary: '#555555',
      textTertiary: '#666666',
      textDisabled: '#999999',

      accentPrimary: '#4E5F8C',
      accentHover: '#3D4A6B',
      accentActive: '#2C3750',

      borderDefault: '#8B9AAA',
      borderSubtle: '#B0B8C0',
      borderAccent: '#4E5F8C',

      shadowSm: 'rgba(0, 0, 0, 0.1)',
      shadowMd: 'rgba(0, 0, 0, 0.15)',
      shadowLg: 'rgba(0, 0, 0, 0.2)',

      gradient: {
        from: '#4E5F8C',
        to: '#3D4A6B',
      },
    },
  },

  silver: {
    name: 'silver',
    displayName: 'Silver',
    description: 'Windows XP Silver theme',
    colors: {
      bgPrimary: '#E6E5E1',
      bgSecondary: '#EFEDE5',
      bgTertiary: '#D8D7D3',

      surfaceDefault: '#FFFFFF',
      surfaceHover: '#CAC1D9',
      surfaceSelected: '#CAC1D9',
      surfaceActive: '#7A5DC7',

      textPrimary: '#0B0B0B',
      textSecondary: '#555555',
      textTertiary: '#666666',
      textDisabled: '#999999',

      accentPrimary: '#7A5DC7',
      accentHover: '#644AA3',
      accentActive: '#4E3880',

      borderDefault: '#A0A0A0',
      borderSubtle: '#C0C0C0',
      borderAccent: '#7A5DC7',

      shadowSm: 'rgba(0, 0, 0, 0.1)',
      shadowMd: 'rgba(0, 0, 0, 0.15)',
      shadowLg: 'rgba(0, 0, 0, 0.2)',

      gradient: {
        from: '#7A5DC7',
        to: '#644AA3',
      },
    },
  },

  dark: {
    name: 'dark',
    displayName: 'Dark Mode',
    description: 'Modern dark theme',
    colors: {
      bgPrimary: '#1E1E1E',
      bgSecondary: '#2D2D2D',
      bgTertiary: '#404040',

      surfaceDefault: '#2A2A2A',
      surfaceHover: '#3E3E42',
      surfaceSelected: '#0E639C',
      surfaceActive: '#007ACC',

      textPrimary: '#E8E8E8',
      textSecondary: '#B8B8B8',
      textTertiary: '#9A9A9A',
      textDisabled: '#6E6E6E',

      accentPrimary: '#4FC1FF',
      accentHover: '#6DD4FF',
      accentActive: '#2AABEE',

      borderDefault: '#5A5A5A',
      borderSubtle: '#3E3E3E',
      borderAccent: '#4FC1FF',

      shadowSm: 'rgba(0, 0, 0, 0.3)',
      shadowMd: 'rgba(0, 0, 0, 0.4)',
      shadowLg: 'rgba(0, 0, 0, 0.5)',

      gradient: {
        from: '#4FC1FF',
        to: '#2AABEE',
      },
    },
  },
}
