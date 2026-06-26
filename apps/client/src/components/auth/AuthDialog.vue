<script setup lang="ts">
import { ref, watch } from "vue";
import { useAuthStore } from "../../stores/authStore";
import { useLocale } from "../../composables/useLocale";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { ShieldIcon } from "@lucide/vue";

const authStore = useAuthStore();
const { t } = useLocale();

// Three modes: code_login (default), password_login, reset_password
const mode = ref<"code_login" | "password_login" | "reset_password">("code_login");

const phone = ref("");
const password = ref("");
const code = ref("");
const errorMsg = ref("");
const successMsg = ref("");
const isSubmitting = ref(false);

// Reset fields and alerts on mode change or dialog toggle
watch([mode, () => authStore.showAuthDialog], () => {
  errorMsg.value = "";
  successMsg.value = "";
  password.value = "";
  code.value = "";
});

async function handleSubmit(e: Event) {
  e.preventDefault();
  errorMsg.value = "";
  successMsg.value = "";

  if (!phone.value || phone.value.length !== 11) {
    errorMsg.value = "请输入正确的 11 位手机号";
    return;
  }

  if (mode.value === "code_login" && !code.value) {
    errorMsg.value = "请输入验证码";
    return;
  }

  if (mode.value === "password_login" && (!password.value || password.value.length < 6)) {
    errorMsg.value = "密码长度至少为 6 位";
    return;
  }

  if (mode.value === "reset_password") {
    if (!code.value) {
      errorMsg.value = "请输入验证码";
      return;
    }
    if (!password.value || password.value.length < 6) {
      errorMsg.value = "新密码长度至少为 6 位";
      return;
    }
  }

  isSubmitting.value = true;
  try {
    if (mode.value === "code_login") {
      await authStore.loginWithCode(phone.value, code.value);
      successMsg.value = "登录/注册成功";
      setTimeout(() => {
        authStore.showAuthDialog = false;
      }, 1000);
    } else if (mode.value === "password_login") {
      await authStore.loginWithPassword(phone.value, password.value);
      successMsg.value = "登录成功";
      setTimeout(() => {
        authStore.showAuthDialog = false;
      }, 1000);
    } else if (mode.value === "reset_password") {
      await authStore.resetPassword(phone.value, code.value, password.value);
      successMsg.value = t("auth.resetSuccess");
      setTimeout(() => {
        mode.value = "code_login";
      }, 1500);
    }
  } catch (err: any) {
    errorMsg.value = err.message || "操作失败，请重试";
  } finally {
    isSubmitting.value = false;
  }
}
</script>

<template>
  <Dialog v-model:open="authStore.showAuthDialog">
    <DialogContent class="sm:max-w-[420px] bg-background/95 backdrop-blur-md border border-border shadow-2xl rounded-xl">
      <DialogHeader class="pb-2">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-primary/10 text-primary">
            <ShieldIcon class="size-5" />
          </div>
          <div>
            <DialogTitle class="text-base font-bold tracking-tight text-foreground">
              <span v-if="mode === 'code_login'">{{ t("auth.login") }} / {{ t("auth.register") }}</span>
              <span v-else-if="mode === 'password_login'">密码登录</span>
              <span v-else-if="mode === 'reset_password'">{{ t("auth.resetPassword") }}</span>
            </DialogTitle>
            <DialogDescription class="text-xs text-muted-foreground mt-0.5">
              <span v-if="mode === 'code_login'">输入手机号与验证码，未注册用户将自动创建并登录。</span>
              <span v-else-if="mode === 'password_login'">使用您的账号密码进行登录。</span>
              <span v-else-if="mode === 'reset_password'">验证手机号并设定您的新密码。</span>
            </DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <form @submit="handleSubmit" class="space-y-4 mt-2">
        <!-- Error Alert -->
        <div v-if="errorMsg" class="flex items-start gap-2 p-3 text-xs rounded-lg bg-destructive/10 border border-destructive/20 text-destructive animate-shake">
          <span class="font-bold shrink-0 mt-0.5">⚠</span>
          <span>{{ errorMsg }}</span>
        </div>

        <!-- Success Alert -->
        <div v-if="successMsg" class="flex items-start gap-2 p-3 text-xs rounded-lg bg-green-500/10 border border-green-500/20 text-green-500">
          <span class="font-bold shrink-0">✓</span>
          <span>{{ successMsg }}</span>
        </div>

        <!-- Phone field -->
        <div class="space-y-1.5">
          <Label for="phone" class="text-xs font-bold text-muted-foreground uppercase tracking-wider">
            {{ t("auth.phone") }}
          </Label>
          <Input
            id="phone"
            v-model="phone"
            type="text"
            maxlength="11"
            :placeholder="t('auth.phonePlaceholder')"
            class="bg-muted/35 border-border text-xs h-10"
            data-testid="login-phone-input"
            required
          />
        </div>

        <!-- Password field (only for Password Login & Reset Password) -->
        <div v-if="mode === 'password_login' || mode === 'reset_password'" class="space-y-1.5">
          <div class="flex justify-between items-center">
            <Label for="password" class="text-xs font-bold text-muted-foreground uppercase tracking-wider">
              {{ mode === 'reset_password' ? '新密码' : t("auth.password") }}
            </Label>
            <button
              v-if="mode === 'password_login'"
              type="button"
              class="text-[11px] text-primary/80 hover:text-primary transition-colors font-medium"
              @click="mode = 'reset_password'"
            >
              {{ t("auth.forgotPassword") }}
            </button>
          </div>
          <Input
            id="password"
            v-model="password"
            type="password"
            :placeholder="t('auth.passwordPlaceholder')"
            class="bg-muted/35 border-border text-xs h-10"
            required
          />
        </div>

        <!-- Verification Code field (only for Code Login & Reset Password) -->
        <div v-if="mode === 'code_login' || mode === 'reset_password'" class="space-y-1.5">
          <div class="flex justify-between items-center">
            <Label for="code" class="text-xs font-bold text-muted-foreground uppercase tracking-wider">
              {{ t("auth.code") }}
            </Label>
            <button
              v-if="mode === 'code_login'"
              type="button"
              class="text-[11px] text-primary/80 hover:text-primary transition-colors font-medium"
              @click="mode = 'reset_password'"
            >
              忘记密码？
            </button>
          </div>
          <div class="relative flex items-center">
            <Input
              id="code"
              v-model="code"
              type="text"
              maxlength="6"
              :placeholder="t('auth.codePlaceholder')"
              class="bg-muted/35 border-border text-xs h-10 pr-24"
              data-testid="login-code-input"
              required
            />
            <span class="absolute right-3 text-[10px] font-mono text-muted-foreground bg-muted px-2 py-1 rounded">
              111111
            </span>
          </div>
          <p class="text-[10px] text-muted-foreground/80 leading-normal">
            提示：短信服务接入中，请使用固定测试码 <code class="bg-muted font-mono px-1 py-0.5 rounded text-foreground font-bold">111111</code>
          </p>
        </div>

        <!-- Action button -->
        <div class="pt-2">
          <Button
            type="submit"
            class="w-full text-xs font-bold py-5 bg-primary hover:bg-primary/95 text-primary-foreground shadow-lg transition-transform active:scale-[0.98] h-10 flex items-center justify-center gap-2"
            :disabled="isSubmitting"
            data-testid="login-submit-btn"
          >
            <span v-if="isSubmitting" class="animate-spin size-3.5 border-2 border-current border-t-transparent rounded-full" />
            <span>
              {{
                isSubmitting
                  ? '请稍候...'
                  : (mode === 'reset_password'
                    ? t("auth.confirmReset")
                    : (mode === 'password_login' ? t("auth.confirmLogin") : '确认登录 / 注册'))
              }}
            </span>
          </Button>
        </div>

        <!-- Mode switching links -->
        <div class="text-center pt-2 border-t border-border/40 flex justify-center gap-4 text-xs font-medium">
          <button
            v-if="mode !== 'code_login'"
            type="button"
            class="text-primary hover:underline"
            @click="mode = 'code_login'"
          >
            验证码登录 / 注册
          </button>
          <button
            v-if="mode !== 'password_login'"
            type="button"
            class="text-primary hover:underline"
            @click="mode = 'password_login'"
          >
            密码登录
          </button>
        </div>
      </form>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-4px); }
  75% { transform: translateX(4px); }
}
.animate-shake {
  animation: shake 0.2s ease-in-out 0s 2;
}
</style>
