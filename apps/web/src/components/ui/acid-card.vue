<script setup lang="ts">
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { HTMLAttributes } from "vue";

interface Props {
  class?: HTMLAttributes["class"];
}

const props = defineProps<Props>();
</script>

<template>
  <Card
    :class="
      cn(
        'bg-acid-black border-acid-lime/50 text-acid-lime bg-opacity-80 shadow-[0_0_15px_rgba(204,255,0,0.1)] backdrop-blur-md transition-all duration-300 hover:shadow-[0_0_25px_rgba(204,255,0,0.2)]',
        props.class,
      )
    "
  >
    <slot />
  </Card>
</template>
