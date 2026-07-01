import { ref } from "vue";
import { defineStore } from "pinia";
import { services } from "../services/provider";
import type { ModelProvider, ModelProviderInput } from "../services/types";

// 模型供应商：Web 走云端 CRUD，桌面端走本地 providers.json 整存整取（离线可用）。
// 桌面端在线时也可用云端，但本地 providers.json 是桌面端对局运行时读取的来源，
// 故桌面端统一以本地为准，云端同步留作后续 todo。

function emptyInput(): ModelProviderInput {
  return {
    name: "",
    category: "custom",
    preset_type: "",
    base_url: "",
    api_key: "",
    api_format: "anthropic",
    models: [],
    enabled: true,
    website_url: "",
    api_key_url: "",
    icon: "",
    icon_color: "",
    sort_order: 0,
  };
}


export const useProviders = defineStore("providers", () => {
  const providers = ref<ModelProvider[]>([]);
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      providers.value = await services.cloud.listModelProviders();
    } catch (e) {
      console.error("加载模型供应商失败", e);
      providers.value = [];
    } finally {
      loading.value = false;
    }
  }


  /** 创建或更新。返回落库后的供应商。 */
  async function save(input: ModelProviderInput, id?: string): Promise<ModelProvider | undefined> {
    if (id) {
      await services.cloud.updateModelProvider(id, input);
      await load();
      return providers.value.find((p) => p.id === id);
    }
    const created = await services.cloud.createModelProvider(input);
    providers.value.push(created);
    return created;
  }

  async function remove(id: string) {
    await services.cloud.deleteModelProvider(id);
    providers.value = providers.value.filter((p) => p.id !== id);
  }

  return { providers, loading, load, save, remove, emptyInput };
});
