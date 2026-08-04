<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()
</script>

<template>
  <div
    :class="
      cn(
        'relative flex items-center rounded-md border border-input dark:bg-input/30 shadow-xs focus-within:ring-1 focus-within:ring-ring transition-[color,box-shadow]',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
