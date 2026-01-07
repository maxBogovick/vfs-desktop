import { ref } from 'vue';
import { getActivePanelMethods } from './useDualPanel';
import { useFileSystem } from './useFileSystem';

export function useGlobalRefresh() {
  const { loadDirectory } = useFileSystem();

  // Shared state to know current paths if needed, but usually we just want to trigger a refresh
  // of whatever is currently open.

  const refreshAllPanels = async (currentPath: string[], panelFs?: string) => {
    console.log('[GlobalRefresh] Refreshing all panels...');

    // 1. Refresh Single Panel (or Active Panel in Dual Mode context if passed path matches)
    // Actually, we should just reload the directory for the provided path/fs 
    // AND check if dual panels need refreshing.

    // Always refresh the requested path first (primary operation target)
    if (currentPath && currentPath.length >= 0) {
        let pathString = currentPath.join('/');
        if (pathString && !pathString.startsWith('/')) {
            pathString = '/' + pathString;
        }
        if (!pathString) pathString = '/'; // or home
        
        // This usually updates 'files' ref in useFileSystem which is shared
        // BUT wait, useFileSystem 'files' ref is shared? 
        // In FileList it uses 'files' from useFileSystem.
        // In DualPanel, each panel has its own state but uses useFileSystem to load.
        // If useFileSystem 'files' is global, then single panel is fine.
        // Let's check useFileSystem... it has `const files = ref<FileItem[]>([]);` locally?
        // No, it's defined inside the function: `export function useFileSystem() { const files = ref... }`
        // So state is NOT shared unless hoisted.
    }

    // Since useFileSystem state is NOT global (it's per component instance usually),
    // we need a way to tell the active components to refresh.
    
    // In App.vue (Single Panel):
    // It watches 'currentPath' and calls 'loadDirectory' which updates 'files'.
    // So 'refreshCurrentDirectory' in App.vue does exactly this:
    /*
    const refreshCurrentDirectory = async () => {
      const pathString = await getCurrentDirectoryPath();
      await loadDirectory(pathString);
    };
    */

    // In Dual Mode:
    // We have `getActivePanelMethods()`. 
    // We probably need `getLeftPanelMethods()` and `getRightPanelMethods()` or similar
    // OR just iterate over all registered panels if we had a registry.
    
    // CURRENT IMPLEMENTATION GAP:
    // We don't have easy access to non-active panel methods in Dual Mode from strictly global context
    // without some event bus or global registry.
    
    // However, `getActivePanelMethods` exists.
    // If we are in Dual Mode, we want to refresh BOTH left and right panels.
    // App.vue knows about Single Panel (it controls it directly).
    // DualPanelContainer knows about Dual Panels.
    
    // Strategy:
    // 1. Dispatch a global event 'vf-refresh-all'
    // 2. Listen for this event in Single Panel (App.vue) and DualPanel (components).
    
    window.dispatchEvent(new Event('vf-refresh-all'));
  };

  return {
    refreshAllPanels
  };
}
