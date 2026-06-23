import { ref } from "vue";

export type SettingsTab =
  | "general"
  | "code_preview"
  | "model_settings"
  | "skills"
  | "mcp"
  | "plugins"
  | "commands"
  | "indexes"
  | "usage";

const currentTab = ref<SettingsTab>("general");

export function useSettingsTab() {
  return { currentTab };
}
