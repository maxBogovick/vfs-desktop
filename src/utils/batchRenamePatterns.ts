/**
 * Batch Rename Patterns Service
 *
 * Professional implementation of batch rename patterns with full validation,
 * error handling, and pattern application logic.
 */

import type {
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
  FileItem,
  ValidationError,
  BatchValidationResult,
} from '../types';

/**
 * Applies all enabled patterns to a filename
 */
export function applyRenamePatterns(
  originalName: string,
  patterns: BatchRenamePattern[],
  index: number,
  preserveExtension: boolean = true
): { newName: string; error?: string } {
  try {
    let name = originalName;
    let extension = '';

    // Extract extension if preserveExtension is true
    if (preserveExtension) {
      const lastDot = originalName.lastIndexOf('.');
      if (lastDot > 0 && lastDot < originalName.length - 1) {
        extension = originalName.substring(lastDot);
        name = originalName.substring(0, lastDot);
      }
    }

    // Apply each enabled pattern in order
    for (const pattern of patterns) {
      if (!pattern.enabled) continue;

      try {
        switch (pattern.type) {
          case 'prefix':
            name = applyPrefix(name, pattern);
            break;
          case 'suffix':
            if (pattern.beforeExtension || !preserveExtension) {
              name = applySuffix(name, pattern);
            } else {
              // Will be added after extension is re-attached
            }
            break;
          case 'replace':
            name = applyReplace(name, pattern);
            break;
          case 'regex':
            name = applyRegex(name, pattern);
            break;
          case 'numbering':
            name = applyNumbering(name, pattern, index);
            break;
          case 'case':
            name = applyCaseChange(name, pattern);
            break;
        }
      } catch (err) {
        return {
          newName: originalName,
          error: `Failed to apply ${pattern.type} pattern: ${err instanceof Error ? err.message : 'Unknown error'}`,
        };
      }
    }

    // Re-attach extension
    let finalName = name + extension;

    // Apply suffix after extension if needed
    const suffixPattern = patterns.find(
      (p) => p.type === 'suffix' && p.enabled && !(p as SuffixPattern).beforeExtension && preserveExtension
    ) as SuffixPattern | undefined;

    if (suffixPattern) {
      finalName = name + extension + suffixPattern.text;
    }

    // Validate final name
    const validation = validateFileName(finalName);
    if (!validation.isValid) {
      return {
        newName: originalName,
        error: validation.error,
      };
    }

    return { newName: finalName };
  } catch (err) {
    return {
      newName: originalName,
      error: err instanceof Error ? err.message : 'Unknown error occurred',
    };
  }
}

/**
 * Apply prefix pattern
 */
function applyPrefix(name: string, pattern: PrefixPattern): string {
  return pattern.text + name;
}

/**
 * Apply suffix pattern
 */
function applySuffix(name: string, pattern: SuffixPattern): string {
  return name + pattern.text;
}

/**
 * Apply replace pattern
 */
function applyReplace(name: string, pattern: ReplacePattern): string {
  const { searchText, replaceText, caseSensitive, wholeWord } = pattern;

  if (!searchText) return name;

  let search = searchText;
  let flags = caseSensitive ? 'g' : 'gi';

  if (wholeWord) {
    // Escape special regex characters
    search = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`\\b${search}\\b`, flags);
    return name.replace(regex, replaceText);
  } else {
    // Escape special regex characters
    search = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(search, flags);
    return name.replace(regex, replaceText);
  }
}

/**
 * Apply regex pattern
 */
function applyRegex(name: string, pattern: RegexPattern): string {
  const { pattern: regexPattern, replacement, flags } = pattern;

  if (!regexPattern) return name;

  try {
    const regex = new RegExp(regexPattern, flags);
    return name.replace(regex, replacement);
  } catch (err) {
    throw new Error(`Invalid regex pattern: ${err instanceof Error ? err.message : 'Unknown error'}`);
  }
}

/**
 * Apply numbering pattern
 */
function applyNumbering(name: string, pattern: NumberingPattern, index: number): string {
  const { startNumber, increment, padding, position, separator } = pattern;

  const number = startNumber + index * increment;
  const paddedNumber = number.toString().padStart(padding, '0');

  switch (position) {
    case 'prefix':
      return `${paddedNumber}${separator}${name}`;
    case 'suffix':
      return `${name}${separator}${paddedNumber}`;
    case 'replace':
      return paddedNumber;
    default:
      return name;
  }
}

/**
 * Apply case change pattern
 */
function applyCaseChange(name: string, pattern: CasePattern): string {
  const { caseType } = pattern;

  switch (caseType) {
    case 'uppercase':
      return name.toUpperCase();

    case 'lowercase':
      return name.toLowerCase();

    case 'titlecase':
      return name
        .toLowerCase()
        .split(' ')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');

    case 'camelcase':
      return name
        .replace(/[_\s-]+(.)?/g, (_, char) => (char ? char.toUpperCase() : ''))
        .replace(/^(.)/, (char) => char.toLowerCase());

    case 'snakecase':
      return name
        .replace(/([A-Z])/g, '_$1')
        .toLowerCase()
        .replace(/^_/, '')
        .replace(/[\s-]+/g, '_');

    case 'kebabcase':
      return name
        .replace(/([A-Z])/g, '-$1')
        .toLowerCase()
        .replace(/^-/, '')
        .replace(/[\s_]+/g, '-');

    default:
      return name;
  }
}

/**
 * Validate filename for illegal characters and reserved names
 */
function validateFileName(name: string): { isValid: boolean; error?: string } {
  if (!name || name.trim() === '') {
    return { isValid: false, error: 'Filename cannot be empty' };
  }

  // Check for illegal characters (OS-specific)
  const illegalChars = /[<>:"/\\|?*\x00-\x1F]/;
  if (illegalChars.test(name)) {
    return { isValid: false, error: 'Filename contains illegal characters' };
  }

  // Check for reserved names (Windows)
  const reservedNames = /^(con|prn|aux|nul|com[0-9]|lpt[0-9])$/i;
  const nameWithoutExt = name.includes('.') ? name.substring(0, name.lastIndexOf('.')) : name;
  if (reservedNames.test(nameWithoutExt)) {
    return { isValid: false, error: 'Filename is a reserved system name' };
  }

  // Check for names that are just dots
  if (/^\.+$/.test(name)) {
    return { isValid: false, error: 'Filename cannot consist only of dots' };
  }

  // Check length (most filesystems have 255 byte limit)
  if (name.length > 255) {
    return { isValid: false, error: 'Filename is too long (max 255 characters)' };
  }

  return { isValid: true };
}

/**
 * Generate rename preview for multiple files
 */
export function generateRenamePreview(
  files: FileItem[],
  config: BatchRenameConfig
): RenamePreviewItem[] {
  const { patterns, applyToFolders, applyToFiles, preserveExtension } = config;

  // Filter files based on config
  const filteredFiles = files.filter((file) => {
    if (file.type === 'folder' || file.type === 'drive') {
      return applyToFolders;
    }
    return applyToFiles;
  });

  return filteredFiles.map((file, index) => {
    const { newName, error } = applyRenamePatterns(file.name, patterns, index, preserveExtension);

    return {
      originalPath: file.path,
      originalName: file.name,
      newName,
      hasError: !!error,
      errorMessage: error,
      fileItem: file,
    };
  });
}

/**
 * Validate rename operation for potential conflicts
 */
export function validateRenameOperation(
  previewItems: RenamePreviewItem[]
): BatchValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationError[] = [];

  // Check for duplicate new names
  const nameMap = new Map<string, string[]>();

  for (const item of previewItems) {
    if (item.hasError) {
      errors.push({
        path: item.originalPath,
        name: item.originalName,
        error: item.errorMessage || 'Unknown error',
      });
      continue;
    }

    // Track duplicate names
    const lowerName = item.newName.toLowerCase();
    if (!nameMap.has(lowerName)) {
      nameMap.set(lowerName, []);
    }
    nameMap.get(lowerName)!.push(item.originalName);
  }

  // Report duplicates
  for (const [newName, originalNames] of nameMap.entries()) {
    if (originalNames.length > 1) {
      errors.push({
        path: '',
        name: newName,
        error: `Duplicate name conflict: ${originalNames.length} files would be renamed to "${newName}"`,
      });
    }
  }

  // Check if any files have no changes
  let unchangedCount = 0;
  for (const item of previewItems) {
    if (!item.hasError && item.originalName === item.newName) {
      unchangedCount++;
    }
  }

  if (unchangedCount > 0) {
    warnings.push({
      path: '',
      name: '',
      error: `${unchangedCount} file(s) will not be renamed (no changes)`,
    });
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Create default patterns for common use cases
 */
export function createDefaultPatterns() {
  return {
    prefix: (): PrefixPattern => ({
      type: 'prefix',
      enabled: false,
      text: '',
    }),

    suffix: (): SuffixPattern => ({
      type: 'suffix',
      enabled: false,
      text: '',
      beforeExtension: true,
    }),

    replace: (): ReplacePattern => ({
      type: 'replace',
      enabled: false,
      searchText: '',
      replaceText: '',
      caseSensitive: false,
      wholeWord: false,
    }),

    regex: (): RegexPattern => ({
      type: 'regex',
      enabled: false,
      pattern: '',
      replacement: '',
      flags: 'g',
    }),

    numbering: (): NumberingPattern => ({
      type: 'numbering',
      enabled: false,
      startNumber: 1,
      increment: 1,
      padding: 3,
      position: 'suffix',
      separator: '-',
    }),

    case: (): CasePattern => ({
      type: 'case',
      enabled: false,
      caseType: 'lowercase',
    }),
  };
}

/**
 * Example pattern presets
 */
export const PATTERN_PRESETS = {
  'Add Date Prefix': (): BatchRenamePattern[] => [
    {
      type: 'prefix',
      enabled: true,
      text: new Date().toISOString().split('T')[0] + '_',
    } as PrefixPattern,
  ],

  'Sequential Numbering': (): BatchRenamePattern[] => [
    {
      type: 'numbering',
      enabled: true,
      startNumber: 1,
      increment: 1,
      padding: 3,
      position: 'prefix',
      separator: '_',
    } as NumberingPattern,
  ],

  'Remove Spaces': (): BatchRenamePattern[] => [
    {
      type: 'replace',
      enabled: true,
      searchText: ' ',
      replaceText: '_',
      caseSensitive: false,
      wholeWord: false,
    } as ReplacePattern,
  ],

  'Lowercase All': (): BatchRenamePattern[] => [
    {
      type: 'case',
      enabled: true,
      caseType: 'lowercase',
    } as CasePattern,
  ],

  'Title Case': (): BatchRenamePattern[] => [
    {
      type: 'case',
      enabled: true,
      caseType: 'titlecase',
    } as CasePattern,
  ],
};
