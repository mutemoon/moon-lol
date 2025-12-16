# 文本域组件 (Textarea)

<cite>
**本文档引用的文件**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue)
- [index.ts](file://apps/web/src/components/ui/textarea/index.ts)
- [Label.vue](file://apps/web/src/components/ui/label/Label.vue)
- [utils.ts](file://apps/web/src/lib/utils.ts)
- [play.vue](file://apps/web/src/pages/play.vue)
</cite>

## 目录
1. [简介](#简介)
2. [核心功能与行为表现](#核心功能与行为表现)
3. [Props 详细说明](#props-详细说明)
4. [事件 (Emitted Events)](#事件-emitted-events)
5. [与 Label 组件协同使用示例](#与-label-组件协同使用示例)
6. [样式覆盖与 TailwindCSS 集成](#样式覆盖与-tailwindcss-集成)
7. [表单验证流程集成](#表单验证流程集成)

## 简介

文本域组件（Textarea）是一个用于处理多行文本输入的 Vue 3 组件，基于 Composition API 和 `<script setup>` 语法构建。该组件通过 `v-model` 实现双向数据绑定，支持与 Vue 生态中的表单系统无缝集成。它被设计为可复用的 UI 原子组件，适用于需要用户输入较长文本内容的场景，如提示词注入、评论输入等。

该组件位于 `apps/web/src/components/ui/textarea/` 目录下，通过 `index.ts` 文件导出，可在项目中通过 `import { Textarea } from "@/components/ui/textarea"` 方式引入使用。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L1-L28)
- [index.ts](file://apps/web/src/components/ui/textarea/index.ts#L1-L2)

## 核心功能与行为表现

文本域组件在用户输入多行文本时表现出以下核心行为特征：

- **基础输入功能**：组件基于原生 `<textarea>` 元素构建，继承其所有默认行为，包括换行、滚动、选择等。
- **聚焦状态样式**：当组件获得焦点时，会应用预定义的视觉反馈样式，包括边框颜色变化、阴影效果以及 z-index 提升，以增强用户体验。
- **禁用状态**：通过 `disabled` prop 控制组件是否可交互，禁用状态下文本域呈现半透明外观且无法获取焦点。
- **自动高度调整**：虽然当前实现未包含 JavaScript 驱动的自动高度调整逻辑，但通过 CSS 类 `field-sizing-content` 和 `min-h-16` 实现了基于内容的弹性布局，允许文本域根据内容自然扩展高度。
- **无障碍支持**：组件通过 `aria-invalid` 属性与表单验证系统集成，能够在验证失败时提供视觉和语义上的错误提示。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L23-L27)

## Props 详细说明

文本域组件接受以下 Props 以实现灵活配置：

| Prop 名称 | 类型 | 默认值 | 说明 |
|----------|------|--------|------|
| `class` | `string` \| `object` \| `array` | - | 允许用户传入额外的 TailwindCSS 类名以覆盖默认样式 |
| `defaultValue` | `string` \| `number` | - | 设置文本域的默认值，当 `modelValue` 未提供时作为回退值使用 |
| `modelValue` | `string` \| `number` | - | 用于双向绑定的文本值，配合 `v-model` 实现数据同步 |

组件通过 `defineProps` 明确定义了这些属性，并利用 TypeScript 提供类型安全。`modelValue` 是实现 `v-model` 双向绑定的关键 prop，其值的变化会通过事件机制同步回父组件。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L6-L10)

## 事件 (Emitted Events)

文本域组件通过 `defineEmits` 显式声明了其触发的事件，确保类型安全和开发体验：

- **`update:modelValue`**：当用户输入导致文本值发生变化时触发，携带更新后的字符串或数字值作为负载。这是实现 `v-model` 双向绑定的核心事件。

组件内部使用 `@vueuse/core` 提供的 `useVModel` 工具函数来简化 `v-model` 的实现逻辑。该函数自动处理 `modelValue` 的读取与 `update:modelValue` 事件的派发，同时支持 `defaultValue` 的回退机制。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L12-L19)

## 与 Label 组件协同使用示例

文本域组件通常与 `Label` 组件配合使用，以提供更好的可访问性和用户体验。以下是一个典型的使用示例：

```vue
<template>
  <Label class="text-acid-lime/70 mb-2 block text-xs font-bold tracking-widest uppercase">
    提示词注入
  </Label>
  <Textarea
    v-model="clientStore.prompt"
    class="border-acid-lime/30 text-acid-lime focus:border-acid-lime focus:ring-acid-lime/20 flex-1 bg-black/50 font-mono text-sm focus:shadow-[0_0_10px_rgba(204,255,0,0.2)]"
    rows="5"
    placeholder="> 定义行为协议..."
  />
</template>

<script setup lang="ts">
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useClientStore } from "@/composables/useClient";

const clientStore = useClientStore();
</script>
```

在此示例中，`Label` 组件为 `Textarea` 提供了语义化的标签，两者通过视觉样式保持一致的设计语言。`v-model` 将文本域的值绑定到 `clientStore.prompt` 状态上，实现响应式更新。

**Section sources**
- [play.vue](file://apps/web/src/pages/play.vue#L11-L17)

## 样式覆盖与 TailwindCSS 集成

文本域组件的样式通过 TailwindCSS 实现，并利用 `cn` 工具函数进行类名合并，支持灵活的样式覆盖。

### 样式合并机制

组件使用 `@/lib/utils` 中的 `cn` 函数来合并默认样式与用户传入的自定义类名：

```ts
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

该函数结合 `clsx` 和 `tailwind-merge`，确保即使用户传入了与默认样式冲突的类名，也能正确合并并去重，避免样式覆盖问题。

### 默认样式

组件内置了丰富的 TailwindCSS 类名，定义了其默认外观：

- **边框与背景**：`border-input`、`bg-transparent`、`dark:bg-input/30`
- **焦点状态**：`focus-visible:border-ring`、`focus-visible:ring-ring/50`、`focus-visible:ring-[3px]`
- **禁用状态**：`disabled:cursor-not-allowed`、`disabled:opacity-50`
- **错误状态**：`aria-invalid:ring-destructive/20`、`dark:aria-invalid:ring-destructive/40`
- **排版**：`text-base`、`md:text-sm`、`placeholder:text-muted-foreground`

用户可通过 `class` prop 传入自定义类名来覆盖这些默认样式，例如修改边框颜色、字体、背景等。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L26-L27)
- [utils.ts](file://apps/web/src/lib/utils.ts#L6-L8)

## 表单验证流程集成

文本域组件通过语义化的 HTML 属性和预定义的样式类，与前端表单验证流程无缝集成。

### 验证状态表示

组件通过 `aria-invalid` 属性来表示其验证状态：
- 当表单验证失败时，父组件应将 `aria-invalid` 设置为 `true`
- 组件的默认样式中已包含对 `aria-invalid` 的响应式处理：`aria-invalid:ring-destructive/20` 和 `aria-invalid:border-destructive`

这使得在验证失败时，文本域会自动显示红色边框和浅红色阴影，提供清晰的视觉反馈。

### 集成方式

在实际应用中，文本域通常与表单验证库（如 VeeValidate 或基于 Pinia 的自定义验证逻辑）结合使用。验证结果通过响应式状态传递给 `Textarea` 组件，控制其 `aria-invalid` 属性和错误消息的显示。

例如，在 `play.vue` 中，虽然未直接展示验证逻辑，但其通过 `clientStore` 管理状态的方式为集成复杂验证流程提供了基础架构。

**Section sources**
- [Textarea.vue](file://apps/web/src/components/ui/textarea/Textarea.vue#L26-L27)
- [play.vue](file://apps/web/src/pages/play.vue#L13-L14)