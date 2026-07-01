<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { useRouter } from "vue-router";
import { MessageSquareIcon, ChevronRightIcon } from "@lucide/vue";

const router = useRouter();

function navigateTo(path: string) {
  router.push(path);
}
</script>

<template>
  <div class="bg-bg-deep relative flex h-full flex-col items-center justify-center overflow-hidden p-6">
    <!-- 背景流光装饰 -->
    <div
      class="pointer-events-none absolute top-[-20%] left-1/2 h-[500px] w-[800px] -translate-x-1/2 rounded-full opacity-20 blur-[120px]"
      style="background: radial-gradient(circle, rgba(185, 145, 71, 0.3) 0%, transparent 70%)"
    ></div>

    <div class="z-10 flex w-full max-w-4xl flex-col items-center gap-8">
      <!-- 头部介绍 -->
      <div class="text-center">
        <h1 class="font-display text-gold-bright text-4xl font-bold tracking-widest uppercase drop-shadow-[0_0_12px_rgba(212,175,92,0.3)]">
          Mock 模拟测试沙盒
        </h1>
        <p class="text-text-muted mt-3 text-sm tracking-wide max-w-lg mx-auto">
          在完全离线的环境下对 MoonLOL 桌面端的核心渲染和通信模块进行模拟调试与交互验证。
        </p>
      </div>

      <!-- 模拟测试卡片 -->
      <div class="w-full max-w-lg">
        <!-- 卡片 1: 决策对话模拟 -->
        <div
          class="group bg-bg-surface/50 border-border-subtle hover:border-gold-dimmer/50 hover:bg-bg-elevated hover:shadow-[0_12px_32px_rgba(0,0,0,0.6),0_0_24px_rgba(185,145,71,0.12)] relative flex cursor-pointer flex-col justify-between overflow-hidden rounded-xl border p-6 backdrop-blur-md transition-all duration-300 hover:-translate-y-1"
          @click="navigateTo('/mock/chat')"
        >
          <!-- 边角金色装饰条 -->
          <div class="bg-gold-default absolute top-0 left-0 h-[2px] w-0 transition-all duration-300 group-hover:w-full"></div>
          
          <div>
            <div class="border-gold-dimmer/20 bg-gold-dimmer/5 text-gold-bright flex h-12 w-12 items-center justify-center rounded-lg border transition-colors group-hover:bg-gold-dimmer/10">
              <MessageSquareIcon class="h-6 w-6" />
            </div>
            
            <h2 class="text-text-bright mt-5 text-xl font-bold tracking-wide transition-colors group-hover:text-gold-bright">
              AI 决策流渲染模拟
            </h2>
            
            <p class="text-text-muted mt-3 text-xs leading-relaxed">
              基于本地 mock.json 数据源，模拟 AI 代理的实时决策思维链生成。测试 Markdown 渲染格式、折叠面板及自定义工具调用展示的排版交互。
            </p>
          </div>

          <div class="text-gold-bright mt-6 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider">
            进入测试床
            <ChevronRightIcon class="h-4 w-4 transform transition-transform group-hover:translate-x-1" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 可添加页面微小动效 */
</style>
