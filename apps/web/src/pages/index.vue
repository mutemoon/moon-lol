<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();

// Glitch Text Logic
const glitchText = ref("MOON\nLOL");
const originalText = "MOON\nLOL";
const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%&";

const handleGlitchOver = () => {
  let iterations = 0;
  const interval = setInterval(() => {
    glitchText.value = originalText
      .split("")
      .map((letter, index) => {
        if (letter === "\n") return "\n";
        if (index < iterations) return originalText[index];
        return chars[Math.floor(Math.random() * chars.length)];
      })
      .join("");

    if (iterations >= originalText.length) clearInterval(interval);
    iterations += 1 / 3;
  }, 30);
};

const posts = [
  {
    title: "工程架构",
    desc: "Moon LoL 的高层系统设计：Rust Core 与 Web Frontend.",
    date: "2025.11.28",
    path: "/blog/architecture",
    tag: "ARCHITECTURE",
  },
  {
    title: "数据流转",
    desc: "从 Bevy ECS 到 Web 前端的数据管线。",
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

onMounted(() => {
  // Scroll Reveal Observer
  const observerOptions = {
    threshold: 0.1,
    rootMargin: "0px",
  };

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add("visible");
      }
    });
  }, observerOptions);

  document.querySelectorAll(".reveal-on-scroll").forEach((el) => {
    observer.observe(el);
  });
});
</script>

<template>
  <!-- HERO SECTION -->
  <section class="relative flex min-h-screen flex-col items-center justify-center overflow-hidden">
    <!-- Floating Background Elements -->
    <div class="pointer-events-none absolute inset-0 overflow-hidden">
      <!-- Floating Bubbles/Blobs -->
      <div
        class="from-acid-blue/40 animate-float absolute top-[20%] left-[10%] h-32 w-32 rounded-full bg-linear-to-br to-transparent blur-xl"
      ></div>
      <div
        class="from-acid-pink/40 animate-float absolute right-[15%] bottom-[30%] h-48 w-48 rounded-full bg-linear-to-tr to-transparent blur-xl"
        style="animation-delay: -2s"
      ></div>

      <!-- Pixel Stars -->
      <div class="text-acid-yellow animate-spin-slow absolute top-[15%] right-[20%] text-4xl">✦</div>
      <div
        class="text-acid-blue animate-spin-slow absolute bottom-[20%] left-[20%] text-2xl"
        style="animation-direction: reverse"
      >
        ✦
      </div>
    </div>

    <div
      class="pointer-events-none absolute inset-0 flex items-center justify-center opacity-80"
      style="perspective: 1400px"
    >
      <div
        class="relative h-[50vw] w-[50vw]"
        style="transform: rotateX(55deg) rotateZ(45deg) scale(0.8); transform-style: preserve-3d"
      >
        <!-- Map Base Grid -->
        <svg viewBox="0 0 100 100" class="h-full w-full overflow-visible drop-shadow-[0_0_30px_var(--color-acid-blue)]">
          <defs>
            <pattern id="grid" width="10" height="10" patternUnits="userSpaceOnUse">
              <path
                d="M 10 0 L 0 0 0 10"
                fill="none"
                stroke="currentColor"
                stroke-width="0.1"
                class="text-acid-blue/50"
              />
            </pattern>
            <filter id="glow">
              <feGaussianBlur stdDeviation="2" result="coloredBlur" />
              <feMerge>
                <feMergeNode in="coloredBlur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
            <circle id="turret" r="1.2" class="fill-current" />
          </defs>

          <!-- Base Plate -->
          <rect
            x="0"
            y="0"
            width="100"
            height="100"
            rx="15"
            fill="rgba(0, 240, 255, 0.05)"
            stroke="var(--color-acid-blue)"
            stroke-width="0.5"
            class="backdrop-blur-sm"
          />

          <!-- Grid Overlay -->
          <rect x="0" y="0" width="100" height="100" rx="15" fill="url(#grid)" class="text-acid-blue/30" />

          <!-- River -->
          <path
            d="M 20,10 L 90,80 L 80,90 L 10,20 Z"
            fill="rgba(0, 240, 255, 0.1)"
            stroke="none"
            class="animate-pulse"
          />

          <!-- Lanes -->
          <g
            class="stroke-current text-white drop-shadow-[0_0_5px_rgba(255,255,255,0.8)]"
            fill="none"
            stroke-width="0.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            filter="url(#glow)"
          >
            <path d="M 10,90 L 10,10 L 88,10" />
            <path d="M 12,90 L 90,90 L 90,10" />
            <path d="M 15,85 L 85,15" class="text-acid-yellow" />
          </g>

          <!-- Structures -->
          <g class="text-acid-blue">
            <circle cx="10" cy="90" r="4" stroke="currentColor" stroke-width="1" class="fill-black" />
            <!-- Turrets as diamonds -->
            <!-- top -->
            <use href="#turret" x="10" y="25" />
            <use href="#turret" x="10" y="45" />
            <use href="#turret" x="10" y="70" />
            <!-- mid -->
            <use href="#turret" x="25" y="75" />
            <use href="#turret" x="35" y="65" />
            <use href="#turret" x="45" y="55" />
            <!-- bottom -->
            <use href="#turret" x="30" y="90" />
            <use href="#turret" x="50" y="90" />
            <use href="#turret" x="75" y="90" />
          </g>

          <g class="text-acid-pink">
            <circle cx="90" cy="10" r="4" stroke="currentColor" stroke-width="1" class="fill-black" />
            <!-- top -->
            <use href="#turret" x="25" y="10" />
            <use href="#turret" x="45" y="10" />
            <use href="#turret" x="70" y="10" />
            <!-- mid -->
            <use href="#turret" x="75" y="25" />
            <use href="#turret" x="65" y="35" />
            <use href="#turret" x="55" y="45" />
            <!-- bottom -->
            <use href="#turret" x="90" y="30" />
            <use href="#turret" x="90" y="50" />
            <use href="#turret" x="90" y="75" />
          </g>
        </svg>

        <!-- Floating Holographic Elements -->
        <div
          class="border-acid-blue/30 absolute inset-0 animate-pulse rounded-[15%] border-2 shadow-[0_0_30px_var(--color-acid-blue)]"
          style="transform: translateZ(0px)"
        ></div>
        <div
          class="border-acid-pink/30 absolute inset-0 rounded-[15%] border-2 shadow-[0_0_30px_var(--color-acid-pink)]"
          style="transform: translateZ(-30px) scale(1.05)"
        ></div>
      </div>
    </div>

    <div class="relative z-10 px-4 text-center">
      <div class="border-acid-blue/50 mb-4 inline-block rounded-full border bg-black/50 px-4 py-1 backdrop-blur-md">
        <p class="text-acid-blue reveal-on-scroll text-glow-blue font-mono text-sm tracking-[0.2em] md:text-base">
          高性能游戏环境
        </p>
      </div>
      <h1
        class="font-glitch chrome-text reveal-on-scroll mb-8 text-[4rem] leading-[0.8] drop-shadow-[0_10px_20px_rgba(0,0,0,0.5)] md:text-[8rem] lg:text-[10rem]"
        @mouseover="handleGlitchOver"
      >
        <pre class="font-glitch">{{ glitchText }}</pre>
      </h1>

      <div class="reveal-on-scroll mt-8 flex justify-center gap-6">
        <button
          class="glass-y2k hover:bg-acid-blue group relative overflow-hidden rounded-full px-8 py-3 font-mono font-bold text-white transition-all hover:scale-105 hover:text-black hover:shadow-[0_0_20px_var(--color-acid-blue)]"
        >
          <span class="relative z-10">开始模拟 -></span>
        </button>
      </div>
    </div>

    <!-- Scroll Hint -->
    <div class="absolute bottom-10 left-1/2 flex -translate-x-1/2 flex-col items-center gap-2 opacity-70">
      <div class="from-acid-blue h-16 w-px animate-pulse bg-linear-to-b to-transparent"></div>
      <span class="text-acid-blue animate-pulse text-xs tracking-widest">SCROLL</span>
    </div>
  </section>

  <!-- Scrolling Tech Stack -->
  <div class="relative z-20 flex overflow-hidden border-y border-white/10 bg-black/50 py-4 backdrop-blur-md">
    <div
      class="font-retro text-acid-blue animate-marquee text-outline flex gap-12 text-2xl font-bold whitespace-nowrap drop-shadow-[0_0_5px_var(--color-acid-blue)]"
    >
      <span>RUST</span>
      <span class="text-acid-pink">///</span>
      <span>BEVY</span>
      <span class="text-acid-pink">///</span>
      <span>ECS</span>
      <span class="text-acid-pink">///</span>
      <span>WEBGL</span>
      <span class="text-acid-pink">///</span>
      <span>WGPU</span>
      <span class="text-acid-pink">///</span>
      <span>VUE</span>
      <span class="text-acid-pink">///</span>
      <span>TAILWIND</span>
      <span class="text-acid-pink">///</span>
      <span>RUST</span>
      <span class="text-acid-pink">///</span>
      <span>BEVY</span>
      <span class="text-acid-pink">///</span>
      <span>ECS</span>
      <span class="text-acid-pink">///</span>
      <span>WEBGL</span>
      <span class="text-acid-pink">///</span>
    </div>
  </div>

  <!-- STACK SECTION -->
  <section id="stack" class="relative overflow-hidden py-20 text-white">
    <!-- Background Grid -->
    <div
      class="absolute inset-0 z-0 opacity-20"
      style="
        background-image:
          linear-gradient(var(--color-acid-blue) 1px, transparent 1px),
          linear-gradient(90deg, var(--color-acid-blue) 1px, transparent 1px);
        background-size: 50px 50px;
      "
    ></div>

    <div class="relative z-10 mx-auto max-w-7xl px-6">
      <h2 class="font-glitch text-outline mb-12 inline-block text-6xl drop-shadow-[0_0_10px_var(--color-acid-pink)]">
        <span class="text-acid-pink">技术</span>
        栈
      </h2>

      <div class="grid grid-cols-1 gap-8 md:grid-cols-3">
        <div
          class="glass-y2k hover-trigger group rounded-2xl p-8 transition-all duration-300 hover:-translate-y-2 hover:shadow-[0_0_30px_rgba(0,240,255,0.3)]"
        >
          <h3
            class="text-acid-blue mb-4 flex items-center gap-2 text-2xl font-bold transition-colors group-hover:text-white"
          >
            <span class="text-xl opacity-50">01.</span>
            引擎
          </h3>
          <p class="font-mono leading-relaxed text-gray-300 group-hover:text-white">
            基于 Bevy 构建，Rust 编写的数据驱动游戏引擎。
          </p>
          <ul class="mt-6 space-y-2 font-mono text-sm text-gray-400">
            <li class="flex items-center gap-2">
              <span class="bg-acid-green h-1.5 w-1.5 rounded-full"></span>
              Bevy 0.15
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-green h-1.5 w-1.5 rounded-full"></span>
              WGPU 渲染
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-green h-1.5 w-1.5 rounded-full"></span>
              跨平台
            </li>
          </ul>
        </div>

        <div
          class="glass-y2k hover-trigger group rounded-2xl p-8 transition-all duration-300 hover:-translate-y-2 hover:shadow-[0_0_30px_rgba(255,0,255,0.3)]"
        >
          <h3
            class="text-acid-pink mb-4 flex items-center gap-2 text-2xl font-bold transition-colors group-hover:text-white"
          >
            <span class="text-xl opacity-50">02.</span>
            架构
          </h3>
          <p class="font-mono leading-relaxed text-gray-300 group-hover:text-white">
            采用 Entity Component System (ECS) 架构。
          </p>
          <ul class="mt-6 space-y-2 font-mono text-sm text-gray-400">
            <li class="flex items-center gap-2">
              <span class="bg-acid-pink h-1.5 w-1.5 rounded-full"></span>
              高并发
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-pink h-1.5 w-1.5 rounded-full"></span>
              内存友好
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-pink h-1.5 w-1.5 rounded-full"></span>
              模块化插件
            </li>
          </ul>
        </div>

        <div
          class="glass-y2k hover-trigger group rounded-2xl p-8 transition-all duration-300 hover:-translate-y-2 hover:shadow-[0_0_30px_rgba(250,255,0,0.3)]"
        >
          <h3
            class="text-acid-yellow mb-4 flex items-center gap-2 text-2xl font-bold transition-colors group-hover:text-white"
          >
            <span class="text-xl opacity-50">03.</span>
            可视化
          </h3>
          <p class="font-mono leading-relaxed text-gray-300 group-hover:text-white">实时可视化和调试工具。</p>
          <ul class="mt-6 space-y-2 font-mono text-sm text-gray-400">
            <li class="flex items-center gap-2">
              <span class="bg-acid-yellow h-1.5 w-1.5 rounded-full"></span>
              Vue 3 前端
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-yellow h-1.5 w-1.5 rounded-full"></span>
              WebSockets
            </li>
            <li class="flex items-center gap-2">
              <span class="bg-acid-yellow h-1.5 w-1.5 rounded-full"></span>
              实时指标
            </li>
          </ul>
        </div>
      </div>
    </div>
    <!-- Background Decoration -->
    <div
      class="font-glitch pointer-events-none absolute top-0 right-0 h-full w-full overflow-hidden text-9xl leading-none break-all text-white opacity-5 mix-blend-overlay select-none"
    >
      SYSTEM SYSTEM SYSTEM SYSTEM SYSTEM SYSTEM
    </div>
  </section>

  <!-- LOGS SECTION -->
  <section id="logs" class="mx-auto max-w-7xl px-6 py-32">
    <div class="reveal-on-scroll mb-20 flex items-end justify-between">
      <h2
        class="font-glitch text-outline text-6xl text-white drop-shadow-[0_0_10px_var(--color-acid-blue)] md:text-8xl"
      >
        开发
        <br />
        <span class="text-acid-blue">日志</span>
      </h2>
      <span class="text-acid-pink hidden animate-pulse font-mono md:block">/// LOGS</span>
    </div>

    <div class="grid gap-8">
      <article
        v-for="post in posts"
        :key="post.path"
        class="glass-y2k group hover-trigger relative cursor-pointer rounded-2xl p-8 transition-all duration-300 hover:scale-[1.02] hover:bg-white/10 hover:shadow-[0_0_30px_rgba(0,240,255,0.2)]"
        @click="router.push(post.path)"
      >
        <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div class="flex-1">
            <div class="mb-4 flex items-center gap-4">
              <span
                class="bg-acid-pink/20 text-acid-pink border-acid-pink/50 rounded-full border px-3 py-1 text-xs font-bold"
              >
                {{ post.tag }}
              </span>
              <span class="font-mono text-xs text-gray-500">{{ post.date }}</span>
            </div>
            <h2 class="group-hover:text-acid-blue mb-4 text-3xl font-bold text-white transition-colors">
              {{ post.title }}
            </h2>
            <p class="max-w-2xl font-mono text-gray-400">{{ post.desc }}</p>
          </div>
          <div
            class="text-acid-blue group-hover:bg-acid-blue flex h-12 w-12 items-center justify-center rounded-full border border-white/20 bg-black/50 transition-all duration-300 group-hover:-rotate-45 group-hover:text-black"
          >
            ->
          </div>
        </div>
      </article>
    </div>
  </section>
</template>
