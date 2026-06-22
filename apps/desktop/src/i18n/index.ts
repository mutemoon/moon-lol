/**
 * i18n 实例与类型定义。
 *
 * 设计要点：
 * - `legacy: false` —— Composition API 模式（与项目其他部分一致）。
 * - 字典静态 import，不走懒加载（项目体量小，无需按需加载）。
 * - 类型由源语言 zh 反向推导，`t('...')` 的 key 在 TS 层有提示与校验。
 *
 * 新增一种语言的步骤：
 *   1. 在 src/locales/ 下新建 `xx.ts`，`const xx = {...} satisfies MessageSchema;`。
 *   2. 在下方 messages 与 `AppLocale` 类型里登记。
 *   3. 在 settings 页语言下拉里加入对应选项。
 */
import { createI18n } from "vue-i18n";
import zh from "../locales/zh";
import en from "../locales/en";

export type AppLocale = "zh" | "en";

/** 源语言字典的结构。其他语言文件需 `satisfies MessageSchema`。 */
export type MessageSchema = typeof zh;

const STORAGE_KEY = "lang";
const DEFAULT_LOCALE: AppLocale = "zh";

function readStoredLocale(): AppLocale {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "en" || stored === "zh" ? stored : DEFAULT_LOCALE;
}

export const i18n = createI18n<{ message: MessageSchema }, AppLocale, false>({
  legacy: false,
  locale: readStoredLocale(),
  fallbackLocale: DEFAULT_LOCALE,
  messages: {
    zh,
    en,
  },
});

export default i18n;
