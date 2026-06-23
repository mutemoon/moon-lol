<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { computed } from "vue";
import { posts } from "@/data/posts";
import { MarkdownRender } from "markstream-vue";

const route = useRoute();
const router = useRouter();
const postName = (route.params as any).name as string;
const post = computed(() => posts[postName]);

if (!post.value) {
  router.push("/#logs");
}
</script>

<template>
  <div class="min-h-screen w-full p-6 pt-32 font-mono text-white md:p-20">
    <article v-if="post" class="prose prose-invert mx-auto max-w-4xl">
      <h1 class="font-glitch text-acid-green mb-8 text-5xl md:text-7xl">{{ post.title }}</h1>

      <div class="mb-12 text-gray-400">
        <span class="text-acid-pink mr-4 text-xs tracking-widest">[{{ post.tag }}]</span>
        <span class="text-xs">{{ post.date }}</span>
      </div>

      <div class="markdown-body">
        <MarkdownRender :content="post.content" />
      </div>
    </article>
  </div>
</template>

<style>
@reference "@/style.css";

/* Custom styles for markstream-vue content if needed */
.markdown-body {
  @apply text-gray-300;
}
.markdown-body h1,
.markdown-body h2,
.markdown-body h3 {
  @apply mt-8 mb-4 font-bold text-white;
}
.markdown-body h1 {
  @apply text-acid-green text-3xl;
}
.markdown-body h2 {
  @apply text-acid-yellow text-2xl;
}
.markdown-body h3 {
  @apply text-acid-pink text-xl;
}
.markdown-body p {
  @apply mb-4 leading-relaxed;
}
.markdown-body ul,
.markdown-body ol {
  @apply mb-4 list-disc pl-6;
}
.markdown-body li {
  @apply mb-2;
}
.markdown-body strong {
  @apply font-bold text-white;
}
.markdown-body code {
  @apply text-acid-green rounded bg-black/50 px-1 py-0.5 font-mono text-sm;
}
.markdown-body pre {
  @apply border-acid-green/20 mb-6 overflow-x-auto rounded-lg border bg-black/50 p-4;
}
.markdown-body pre code {
  @apply bg-transparent p-0 text-gray-300;
}
</style>
