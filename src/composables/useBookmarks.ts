import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Bookmark } from '../types';

export function useBookmarks() {
  const bookmarks = ref<Bookmark[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Load bookmarks from backend
  const loadBookmarks = async (): Promise<void> => {
    try {
      isLoading.value = true;
      error.value = null;
      bookmarks.value = await invoke<Bookmark[]>('get_bookmarks');
    } catch (e) {
      error.value = e as string;
      console.error('Failed to load bookmarks:', e);
    } finally {
      isLoading.value = false;
    }
  };

  // Add a new bookmark
  const addBookmark = async (path: string, name?: string): Promise<Bookmark | null> => {
    try {
      isLoading.value = true;
      error.value = null;
      const bookmark = await invoke<Bookmark>('add_bookmark', { path, name });
      await loadBookmarks(); // Reload to get updated list
      return bookmark;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to add bookmark:', e);
      return null;
    } finally {
      isLoading.value = false;
    }
  };

  // Remove a bookmark
  const removeBookmark = async (id: string): Promise<boolean> => {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('remove_bookmark', { id });
      await loadBookmarks(); // Reload to get updated list
      return true;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to remove bookmark:', e);
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // Rename a bookmark
  const renameBookmark = async (id: string, newName: string): Promise<boolean> => {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('rename_bookmark', { id, newName });
      await loadBookmarks(); // Reload to get updated list
      return true;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to rename bookmark:', e);
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // Check if a path is bookmarked
  const isBookmarked = (path: string): boolean => {
    return bookmarks.value.some(b => b.path === path);
  };

  return {
    bookmarks,
    isLoading,
    error,
    loadBookmarks,
    addBookmark,
    removeBookmark,
    renameBookmark,
    isBookmarked,
  };
}
