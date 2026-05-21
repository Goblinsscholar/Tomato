import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Tag } from '@/types';
import { showError } from '@/lib/toast';

interface TagStore {
  tags: Tag[];
  isLoading: boolean;
  error: string | null;
  fetch: () => Promise<void>;
  createTag: (name: string) => Promise<void>;
  deleteTag: (name: string) => Promise<void>;
  updateTag: (currentName: string, newName: string, newColor: string) => Promise<void>;
  getColor: (tagName: string) => string;
}

export const useTagStore = create<TagStore>((set, get) => ({
  tags: [],
  isLoading: false,
  error: null,

  fetch: async () => {
    set({ isLoading: true, error: null });
    try {
      const tags = await invoke<Tag[]>('get_all_tags');
      set({ tags });
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  getColor: (tagName: string) => {
    const tag = get().tags.find((t) => t.name === tagName);
    return tag?.color ?? '#6366f1';
  },

  createTag: async (name: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke<boolean>('create_tag', { name });
      await get().fetch();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  deleteTag: async (name: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('delete_tag', { name });
      await get().fetch();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  updateTag: async (currentName: string, newName: string, newColor: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke<boolean>('update_tag', {
        currentName,
        newName,
        newColor,
      });
      await get().fetch();
    } catch (e) {
      showError(String(e));
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },
}));
