import { create } from "zustand";

interface UiState {
  commandOpen: boolean;
  sidebarCollapsed: boolean;
  setCommandOpen: (open: boolean) => void;
  toggleSidebar: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  commandOpen: false,
  sidebarCollapsed: false,
  setCommandOpen: (commandOpen) => set({ commandOpen }),
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
}));
