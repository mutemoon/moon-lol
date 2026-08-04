<script setup lang="ts">
import type { NumberFieldIncrementProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { PlusIcon } from "lucide-vue-next"
import { NumberFieldIncrement, useForwardProps } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<NumberFieldIncrementProps & { class?: HTMLAttributes["class"] }>()

const delegatedProps = reactiveOmit(props, "class")
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldIncrement
    data-slot="increment"
    v-bind="forwarded"
    :class="
      cn(
        'absolute right-0 top-0 bottom-0 flex items-center justify-center border-l px-1 text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50',
        props.class,
      )
    "
  >
    <slot>
      <PlusIcon class="size-3" />
    </slot>
  </NumberFieldIncrement>
</template>
