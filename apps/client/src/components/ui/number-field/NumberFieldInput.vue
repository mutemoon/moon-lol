<script setup lang="ts">
import { NumberFieldInput } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
  placeholder?: string
}>()
</script>

<template>
  <NumberFieldInput
    data-slot="input"
    :placeholder="placeholder"
    :class="
      cn(
        'flex h-7 w-full rounded-md border-0 bg-transparent px-2.5 py-1 text-xs font-mono shadow-none transition-colors placeholder:text-muted-foreground focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
        props.class,
      )
    "
  />
</template>
