<script setup lang="ts"></script>

<template>
  <div class="bg-acid-dark selection:bg-acid-blue min-h-screen w-full overflow-hidden text-white selection:text-black">
    <div class="noise-overlay"></div>

    <!-- Y2K Background Gradient -->
    <div class="pointer-events-none fixed inset-0 z-0 opacity-30">
      <div
        class="bg-acid-blue animate-float absolute top-[-20%] left-[-10%] h-[50%] w-[50%] rounded-full mix-blend-screen blur-[150px]"
      ></div>
      <div
        class="bg-acid-pink animate-float absolute right-[-10%] bottom-[-20%] h-[50%] w-[50%] rounded-full mix-blend-screen blur-[150px]"
        style="animation-delay: -3s"
      ></div>
    </div>

    <!-- Header -->
    <header class="glass-y2k fixed top-4 right-4 left-4 z-50 rounded-2xl border-white/10">
      <div class="flex h-16 items-center justify-between px-6">
        <!-- Brand -->
        <router-link to="/" class="hover-trigger group flex items-center gap-3">
          <div
            class="group-hover:border-acid-blue relative flex h-10 w-10 items-center justify-center overflow-hidden rounded-full border-2 border-white/20 bg-black/50 shadow-[0_0_15px_var(--color-acid-blue)] transition-all group-hover:scale-110"
          >
            <div
              class="from-acid-blue/20 to-acid-pink/20 absolute inset-0 bg-linear-to-tr opacity-0 transition-opacity group-hover:opacity-100"
            ></div>
            <span class="font-glitch group-hover:text-acid-blue text-lg text-white transition-colors">M</span>
          </div>
          <div class="flex flex-col">
            <span
              class="font-glitch text-outline text-2xl leading-none tracking-wider text-white drop-shadow-[0_0_5px_rgba(255,255,255,0.5)]"
            >
              MOON LOL
            </span>
            <span class="text-acid-blue font-mono text-[10px] tracking-[0.2em]">SIMULATION ENV</span>
          </div>
        </router-link>

        <!-- Navigation -->
        <nav class="hidden h-full items-center gap-2 md:flex">
          <a
            v-for="item in [
              { name: '首页', path: '/' },
              { name: '模拟', path: '/play' },
              { name: '特性', path: '/#stack' },
              { name: '日志', path: '/#logs' },
            ]"
            :key="item.path"
            :href="item.path"
            class="group relative flex h-8 items-center rounded-full px-4 font-mono text-sm font-bold text-gray-300 transition-all hover:bg-white/10 hover:text-white hover:shadow-[0_0_10px_var(--color-acid-pink)]"
          >
            <span class="group-hover:text-glow-pink relative z-10">{{ item.name }}</span>
          </a>
        </nav>

        <!-- Actions -->
        <div class="flex items-center gap-4">
          <a
            href="https://github.com/mutemoon/moon-lol"
            target="_blank"
            class="glass-y2k hover:bg-acid-blue/20 hover:border-acid-blue hover:text-acid-blue flex items-center gap-2 rounded-xl px-4 py-2 font-mono text-xs font-bold transition-all hover:scale-105 hover:shadow-[0_0_15px_var(--color-acid-blue)]"
          >
            <div class="h-2 w-2 animate-pulse rounded-full bg-green-400"></div>
            GITHUB_REPO
          </a>
        </div>
      </div>
    </header>

    <main class="relative z-10">
      <router-view />
    </main>

    <!-- FOOTER -->
    <footer
      id="contact"
      class="glass-y2k relative overflow-hidden border-t border-white/10 pt-20 pb-10 backdrop-blur-xl"
    >
      <div class="relative z-10 mx-auto max-w-7xl px-6">
        <div class="mb-20 grid grid-cols-1 gap-12 md:grid-cols-4">
          <!-- Brand -->
          <div class="col-span-1 md:col-span-1">
            <h2
              class="font-glitch text-acid-blue text-outline mb-6 text-4xl drop-shadow-[0_0_10px_var(--color-acid-blue)]"
            >
              MOON LOL
            </h2>
            <p class="font-mono text-sm leading-relaxed text-gray-300">
              基于 Rust Bevy 引擎的高性能 MOBA 游戏环境。
              <br />
              <span class="text-acid-pink">Y2K RUSH EDITION</span>
            </p>
          </div>

          <!-- Navigation -->
          <div class="col-span-1">
            <h3 class="font-glitch text-acid-pink text-glow-pink mb-6 text-xl">导航</h3>
            <ul class="space-y-3 font-mono text-sm text-gray-400">
              <li>
                <router-link to="/" class="hover:text-acid-blue inline-block transition-all hover:translate-x-2">
                  首页
                </router-link>
              </li>
              <li>
                <a href="/#logs" class="hover:text-acid-blue inline-block transition-all hover:translate-x-2">
                  开发日志
                </a>
              </li>
              <li>
                <a href="/#stack" class="hover:text-acid-blue inline-block transition-all hover:translate-x-2">特性</a>
              </li>
              <li>
                <a href="/#stack" class="hover:text-acid-blue inline-block transition-all hover:translate-x-2">
                  技术栈
                </a>
              </li>
            </ul>
          </div>

          <!-- Resources -->
          <div class="col-span-1">
            <h3 class="font-glitch text-acid-yellow text-outline mb-6 text-xl">资源</h3>
            <ul class="space-y-3 font-mono text-sm text-gray-400">
              <li>
                <a
                  href="https://github.com/mutemoon/moon-lol"
                  target="_blank"
                  class="hover:text-acid-yellow inline-block transition-all hover:translate-x-2"
                >
                  GITHUB 仓库
                </a>
              </li>
              <li>
                <span class="cursor-not-allowed opacity-50">文档 (WIP)</span>
              </li>
              <li>
                <span class="cursor-not-allowed opacity-50">API 参考 (WIP)</span>
              </li>
            </ul>
          </div>

          <!-- Status -->
          <div class="col-span-1">
            <h3 class="font-glitch text-outline mb-6 text-xl text-white">系统状态</h3>
            <div
              class="glass-y2k hover:border-acid-green/50 space-y-4 rounded-xl p-4 font-mono text-xs transition-colors"
            >
              <div class="flex justify-between">
                <span class="text-gray-400">版本</span>
                <span class="text-acid-green font-bold">v0.1.0-alpha</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-400">核心</span>
                <span class="text-acid-pink font-bold">Rust/Bevy</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-400">状态</span>
                <span class="text-acid-blue animate-pulse font-bold">● 在线</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Bottom Bar -->
        <div
          class="flex flex-col items-center justify-between border-t border-white/10 pt-8 font-mono text-xs text-gray-500 md:flex-row"
        >
          <p>&copy; 2025 MOON LOL. OPEN SOURCE MIT LICENSE.</p>
          <div class="mt-4 flex gap-6 md:mt-0">
            <span class="hover:text-acid-blue cursor-pointer transition-colors">PRIVACY</span>
            <span class="hover:text-acid-blue cursor-pointer transition-colors">TERMS</span>
          </div>
        </div>
      </div>

      <!-- Background Element -->
      <div
        class="font-glitch pointer-events-none absolute -right-10 -bottom-10 text-[15rem] leading-none text-white/5 mix-blend-overlay select-none"
      >
        VOID
      </div>
    </footer>
  </div>
</template>
