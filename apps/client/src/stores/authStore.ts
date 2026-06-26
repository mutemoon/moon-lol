import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { services } from "../services/provider";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<{ id: number; phone: string } | null>(null);
  const loading = ref(false);
  const showAuthDialog = ref(false);

  const isAuthenticated = computed(() => !!user.value && services.cloud.isAuthenticated());

  async function fetchMe() {
    if (!services.cloud.isAuthenticated()) {
      user.value = null;
      return;
    }
    loading.value = true;
    try {
      const u = await services.cloud.getCurrentUser();
      user.value = u;
    } catch (err) {
      console.error("[AuthStore] Failed to fetch current user:", err);
      logout();
    } finally {
      loading.value = false;
    }
  }

  async function loginWithCode(phone: string, code: string) {
    loading.value = true;
    try {
      await services.cloud.codeLogin(phone, code);
      await fetchMe();
    } finally {
      loading.value = false;
    }
  }

  async function loginWithPassword(phone: string, password: string) {
    loading.value = true;
    try {
      await services.cloud.login(phone, password);
      await fetchMe();
    } finally {
      loading.value = false;
    }
  }

  async function resetPassword(phone: string, code: string, password: string) {
    loading.value = true;
    try {
      await services.cloud.resetPassword(phone, code, password);
    } finally {
      loading.value = false;
    }
  }

  function logout() {
    services.cloud.logout();
    user.value = null;
  }

  async function init() {
    services.events.on("unauthorized", () => {
      showAuthDialog.value = true;
      user.value = null;
    });
    if (services.cloud.isAuthenticated()) {
      await fetchMe();
    }
  }

  return {
    user,
    loading,
    showAuthDialog,
    isAuthenticated,
    loginWithCode,
    loginWithPassword,
    resetPassword,
    logout,
    fetchMe,
    init,
  };
});
