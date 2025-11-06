<template>
  <img :src="img" />

  <Button @click="updateImg">update</Button>
  <Button @click="step">step</Button>
  <Button @click="observe">observe</Button>
</template>

<script setup lang="ts">
import { useClient } from "@/composables/useClient";

const { img, updateImg, step, observe } = useClient();
</script>
