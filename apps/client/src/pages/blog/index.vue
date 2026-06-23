<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

const posts = [
  {
    title: "工程架构",
    desc: "Moon LoL 的高层系统设计：Rust Core, Python Interface, Vue Frontend.",
    date: "2025.11.28",
    path: "/blog/architecture",
    tag: "ARCHITECTURE",
  },
  {
    title: "数据流转",
    desc: "从 Bevy ECS 到 Python Gym 再到 Web 前端的数据管线。",
    date: "2025.11.28",
    path: "/blog/data-flow",
    tag: "DATA",
  },
  {
    title: "ECS 组件与系统",
    desc: "深入解析游戏核心逻辑：插件系统与实体组件设计。",
    date: "2025.11.28",
    path: "/blog/ecs",
    tag: "CORE",
  },
];
</script>

<template>
  <div class="min-h-screen w-full p-6 pt-32 font-mono text-white md:p-20">
    <header class="mb-20 flex items-center justify-between">
      <h1 class="font-glitch text-6xl md:text-8xl">
        DEV
        <br />
        <span class="text-acid-green">LOGS</span>
      </h1>
    </header>

    <div class="grid gap-12">
      <article
        v-for="post in posts"
        :key="post.path"
        class="group border-acid-green/30 hover:border-acid-green relative cursor-pointer border-t py-8 transition-colors"
        @click="router.push(post.path)"
      >
        <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div class="flex-1">
            <div class="mb-2 flex items-center gap-4">
              <span class="text-acid-pink text-xs tracking-widest">[{{ post.tag }}]</span>
              <span class="text-xs text-gray-500">{{ post.date }}</span>
            </div>
            <h2 class="group-hover:text-acid-green mb-4 text-3xl font-bold transition-colors">
              {{ post.title }}
            </h2>
            <p class="max-w-2xl text-gray-400">{{ post.desc }}</p>
          </div>
          <div
            class="text-acid-green text-2xl opacity-0 transition-all duration-300 group-hover:translate-x-4 group-hover:opacity-100"
          >
            ->
          </div>
        </div>
      </article>
    </div>
  </div>
</template>
