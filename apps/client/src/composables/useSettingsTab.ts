import { ref } from "vue";

export type SettingsTab = "general" | "model_settings";

const currentTab = ref<SettingsTab>("general");

export function useSettingsTab() {
  return { currentTab };
}
