import { create } from "zustand";
import { invokeCommand, type ConfigStatus } from "@/lib/tauri";

interface ConfigState {
  loaded: boolean;
  availableModels: string[];
  errorMessage: string | null;
  loading: boolean;
  loadConfig: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set) => ({
  loaded: false,
  availableModels: [],
  errorMessage: null,
  loading: false,

  loadConfig: async () => {
    set({ loading: true });
    try {
      const status = await invokeCommand<ConfigStatus>("load_config");
      set({
        loaded: status.loaded,
        availableModels: status.available_models,
        errorMessage: status.error_message ?? null,
        loading: false,
      });
    } catch (err) {
      set({
        loaded: false,
        availableModels: [],
        errorMessage:
          err instanceof Error ? err.message : "Failed to load configuration",
        loading: false,
      });
    }
  },
}));
