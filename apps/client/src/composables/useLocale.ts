/**
 * locale 切换与持久化的薄封装。
 *
 * - `locale` 是绑定到 i18n 全局实例的可写 computed，直接赋值即可切换。
 * - `setLocale` 会同步写入 localStorage（与 settings 页历史 key `lang` 对齐）
 *   并更新 `<html lang>` 属性。
 *
 * 在任意组件 `<script setup>` 中：`const { t, locale, setLocale } = useLocale();`
 */
import { computed } from "vue";
import type { Composer } from "vue-i18n";
import { i18n, type AppLocale } from "../i18n";

const STORAGE_KEY = "lang";

const availableLocales: { value: AppLocale; zh: string; native: string }[] = [
  { value: "zh", zh: "简体中文", native: "简体中文" },
  { value: "en", zh: "英文", native: "English" },
];

// legacy: false 时 global 为 Composer（locale 是 WritableComputedRef）
const global = i18n.global as unknown as Composer;

export function useLocale() {
  const t = global.t;

  const locale = computed<AppLocale>({
    get: () => global.locale.value as AppLocale,
    set: (val) => setLocale(val),
  });

  function setLocale(val: AppLocale) {
    global.locale.value = val;
    localStorage.setItem(STORAGE_KEY, val);
    document.documentElement.setAttribute("lang", val);
  }

  return { t, locale, setLocale, availableLocales };
}
