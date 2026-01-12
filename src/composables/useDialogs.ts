import { ref } from 'vue';
import type { FileItem } from '../types';

// Module-level shared state (singleton)
const confirmDialog = ref<{
  isOpen: boolean;
  title: string;
  message: string;
  type: 'warning' | 'danger' | 'info';
  onConfirm: () => void;
}>({
  isOpen: false,
  title: '',
  message: '',
  type: 'warning',
  onConfirm: () => {},
});

const propertiesDialog = ref<{ isOpen: boolean; file: FileItem | null }>({
  isOpen: false,
  file: null,
});

const inputDialog = ref<{
  isOpen: boolean;
  title: string;
  label: string;
  defaultValue: string;
  placeholder: string;
  inputType: string;
  onConfirm: (value: string) => void;
}>({
  isOpen: false,
  title: '',
  label: '',
  defaultValue: '',
  placeholder: '',
  inputType: 'text',
  onConfirm: () => {},
});

export function useDialogs() {
  const showConfirm = (
    title: string,
    message: string,
    onConfirm: () => void,
    type: 'warning' | 'danger' | 'info' = 'warning'
  ) => {
    confirmDialog.value = {
      isOpen: true,
      title,
      message,
      type,
      onConfirm,
    };
  };

  const closeConfirm = () => {
    confirmDialog.value.isOpen = false;
  };

  const showProperties = (file: FileItem) => {
    propertiesDialog.value = {
      isOpen: true,
      file,
    };
  };

  const closeProperties = () => {
    propertiesDialog.value.isOpen = false;
  };

  const showInput = (
    title: string,
    label: string,
    onConfirm: (value: string) => void,
    defaultValue = '',
    placeholder = '',
    inputType = 'text'
  ) => {
    inputDialog.value = {
      isOpen: true,
      title,
      label,
      defaultValue,
      placeholder,
      inputType,
      onConfirm,
    };
  };

  const closeInput = () => {
    inputDialog.value.isOpen = false;
  };

  return {
    // Confirm Dialog
    confirmDialog,
    showConfirm,
    closeConfirm,

    // Properties Dialog
    propertiesDialog,
    showProperties,
    closeProperties,

    // Input Dialog
    inputDialog,
    showInput,
    closeInput,
  };
}
