<script setup lang="ts">
import type { NumberFieldDecrementProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { MinusIcon } from "lucide-vue-next"
import { NumberFieldDecrement, useForwardProps } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<NumberFieldDecrementProps & { class?: HTMLAttributes["class"] }>()

const delegatedProps = reactiveOmit(props, "class")
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldDecrement
    data-slot="decrement"
    v-bind="forwarded"
    :class="
      cn(
        'absolute right-5 top-0 bottom-0 flex items-center justify-center border-l px-1 text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50',
        props.class,
      )
    "
  >
    <slot>
      <MinusIcon class="size-3" />
    </slot>
  </NumberFieldDecrement>
</template>
