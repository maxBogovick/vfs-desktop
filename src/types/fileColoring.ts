export interface ColorRule {
  id: string;
  name: string;
  pattern: string; // Regex pattern for name/path
  color: string;
  enabled: boolean;
}

export interface FileColorConfig {
  enabled: boolean;
  base: {
    file: string;
    folder: string;
    symlink: string;
    drive: string;
    system: string;
  };
  fileTypes: {
    code: string;
    image: string;
    video: string;
    audio: string;
    archive: string;
    document: string;
    pdf: string;
  };
  special: {
    favorite: string;
    recent: string; // Created/Modified < 24h
    symlink: string;
    hidden: string;
  };
  extensions: Record<string, string>; // ext -> color
  customRules: ColorRule[];
}

export const DEFAULT_COLOR_CONFIG: FileColorConfig = {
  enabled: true,
  base: {
    file: '#374151', // text-gray-700
    folder: '#1F2937', // text-gray-800
    symlink: '#8B5CF6', // text-purple-500
    drive: '#059669', // text-emerald-600
    system: '#6B7280', // text-gray-500
  },
  fileTypes: {
    code: '#2563EB', // blue-600
    image: '#D946EF', // fuchsia-500
    video: '#EF4444', // red-500
    audio: '#EC4899', // pink-500
    archive: '#F59E0B', // amber-500
    document: '#4B5563', // gray-600
    pdf: '#DC2626', // red-600
  },
  special: {
    favorite: '#F59E0B', // amber-500 (Star color)
    recent: '#10B981', // emerald-500
    symlink: '#8B5CF6',
    hidden: '#9CA3AF', // gray-400
  },
  extensions: {
    'ts': '#3178C6',
    'js': '#F7DF1E',
    'vue': '#42B883',
    'rs': '#DEA584',
    'json': '#000000',
    'md': '#000000',
    'html': '#E34F26',
    'css': '#1572B6',
    'scss': '#CC6699',
  },
  customRules: [],
};
