<template>
  <div
    class="bg-acid-black text-acid-lime data-flow-bg selection:bg-acid-pink selection:text-acid-black flex h-screen flex-col pt-24 font-mono"
  >
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧面板 - AI配置和状态 -->
      <div class="border-acid-lime/30 flex w-96 flex-col bg-black/50 backdrop-blur-sm">
        <div class="border-acid-lime/30 flex flex-1 flex-col overflow-hidden p-6">
          <h2 class="text-acid-pink glitch-text mb-4 text-lg font-bold tracking-widest uppercase">AI 思维</h2>

          <Label class="text-acid-lime/70 mb-2 block text-xs font-bold tracking-widest uppercase">提示词注入</Label>
          <Textarea
            v-model="clientStore.prompt"
            class="border-acid-lime/30 text-acid-lime focus:border-acid-lime focus:ring-acid-lime/20 flex-1 bg-black/50 font-mono text-sm focus:shadow-[0_0_10px_rgba(204,255,0,0.2)]"
            rows="5"
            placeholder="> 定义行为协议..."
          />
        </div>
        <div class="border-t border-gray-800 bg-gray-900 p-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <Button @click="clientStore.step" variant="acid" class="h-8 text-xs" :disabled="clientStore.playing">
                单步执行
              </Button>
              <Button
                @click="clientStore.observe"
                variant="outline"
                class="border-acid-pink text-acid-pink hover:bg-acid-pink h-8 text-xs hover:text-black"
              >
                观察
              </Button>
            </div>

            <div class="flex items-center gap-3">
              <Button
                @click="clientStore.playing ? clientStore.stop() : clientStore.play()"
                :variant="clientStore.playing ? 'destructive' : 'acid'"
                class="min-w-[80px] shadow-[0_0_15px_rgba(204,255,0,0.3)]"
              >
                {{ clientStore.playing ? "停止" : "启动" }}
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex w-96 flex-col border-l border-gray-800 bg-gray-900/50">
        <!-- AI决策监控 -->
        <div class="flex-1 overflow-y-auto p-6">
          <h3 class="text-acid-pink glitch-text mb-4 text-lg font-bold tracking-widest uppercase">AI 决策日志</h3>

          <!-- 当前决策 -->
          <div
            v-if="clientStore.action"
            class="group border-acid-lime/50 bg-acid-black mb-6 rounded-none border p-4 shadow-[0_0_10px_rgba(204,255,0,0.2)] transition-all duration-300 hover:shadow-[0_0_20px_rgba(204,255,0,0.4)]"
          >
            <div class="text-acid-lime mb-3 flex items-center gap-2 text-xs font-bold tracking-widest uppercase">
              <span class="bg-acid-lime h-2 w-2 animate-pulse shadow-[0_0_5px_#ccff00]"></span>
              当前动作
            </div>
            <div class="text-acid-lime/80 group-hover:text-acid-lime font-mono text-sm transition-colors">
              {{ JSON.stringify(clientStore.action, null, 2) }}
            </div>
          </div>

          <!-- 观察结果 -->
          <div
            v-if="clientStore.observation"
            class="group border-acid-pink/50 bg-acid-black mb-6 rounded-none border p-4 shadow-[0_0_10px_rgba(255,0,204,0.2)] transition-all duration-300 hover:shadow-[0_0_20px_rgba(255,0,204,0.4)]"
          >
            <div class="text-acid-pink mb-3 flex items-center gap-2 text-xs font-bold tracking-widest uppercase">
              <span class="bg-acid-pink h-2 w-2 animate-pulse shadow-[0_0_5px_#ff00cc]"></span>
              观察数据
            </div>
            <div class="text-acid-pink/80 group-hover:text-acid-pink font-mono text-sm transition-colors">
              {{ JSON.stringify(clientStore.observation, null, 2) }}
            </div>
          </div>

          <!-- 思考过程 -->
          <div
            v-if="clientStore.message"
            class="group bg-acid-black mb-6 rounded-none border border-white/30 p-4 shadow-[0_0_10px_rgba(255,255,255,0.1)] transition-all duration-300 hover:shadow-[0_0_20px_rgba(255,255,255,0.2)]"
          >
            <div class="mb-3 flex items-center gap-2 text-xs font-bold tracking-widest text-white uppercase">
              <span class="h-2 w-2 animate-pulse bg-white shadow-[0_0_5px_#ffffff]"></span>
              思考过程
            </div>
            <div
              class="max-h-32 overflow-y-auto font-mono text-sm text-gray-300 transition-colors group-hover:text-white"
            >
              {{ clientStore.message }}
            </div>
          </div>

          <!-- 决策历史 -->
          <div class="space-y-3">
            <div class="mb-3 flex items-center gap-2 text-sm tracking-wider text-gray-400 uppercase">
              <span class="h-1 w-1 rounded-full bg-gray-400"></span>
              决策历史
            </div>
            <div class="space-y-2">
              <div
                class="group rounded border border-gray-700 bg-linear-to-r from-gray-800/50 to-gray-900/50 p-3 transition-all duration-200 hover:border-gray-600"
              >
                <div class="mb-2 flex items-center justify-between">
                  <div class="text-xs text-gray-500">帧数 {{ clientStore.frame }}</div>
                  <div class="h-1.5 w-1.5 animate-pulse rounded-full bg-blue-400"></div>
                </div>
                <div class="font-mono text-sm text-gray-300 transition-colors group-hover:text-white">
                  {{ clientStore.action ? "动作已执行" : "无动作" }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 底部状态栏 -->
        <div class="border-t border-gray-800 bg-gray-900 p-4">
          <div class="flex items-center justify-between text-xs text-gray-400">
            <span>AI 状态: {{ clientStore.playing ? "运行中" : "已停止" }}</span>
          </div>
        </div>
      </div>

      <!-- 右侧游戏视图 - 主要焦点区域 -->
      <div class="relative flex flex-1 flex-col bg-gray-900/50">
        <!-- 游戏画面 -->
        <div class="relative flex-1 border-r border-gray-800">
          <div class="absolute inset-0 bg-linear-to-br from-gray-900 to-black">
            <img v-if="clientStore.img" :src="clientStore.img" alt="游戏画面" class="h-full w-full object-cover" />
            <div v-else class="flex h-full items-center justify-center">
              <div class="text-center text-gray-500">
                <div class="mb-4 text-6xl opacity-50">🎮</div>
                <p class="text-lg">等待游戏数据...</p>
                <p class="mt-2 text-sm text-gray-600">点击更新图像开始</p>
              </div>
            </div>
          </div>
        </div>

        <!-- 底部控制面板 -->
        <div class="absolute bottom-0 left-0 z-10">
          <div class="flex items-center gap-4">
            <!-- Frame -->
            <div
              class="group hover:border-acid-lime/30 flex min-w-[80px] flex-col items-center rounded px-3 py-1.5 backdrop-blur-sm transition-all"
            >
              <div
                class="group-hover:text-acid-lime/70 text-[10px] font-medium tracking-widest text-gray-600 uppercase transition-colors"
              >
                帧数
              </div>
              <div class="group-hover:text-acid-lime font-mono text-lg font-bold text-gray-400 transition-colors">
                {{ clientStore.frame }}
              </div>
            </div>

            <!-- Enemy HP -->
            <div
              class="group flex min-w-[80px] flex-col items-center rounded px-3 py-1.5 backdrop-blur-sm transition-all"
            >
              <div class="text-[10px] font-medium tracking-widest text-gray-600 uppercase transition-colors">
                敌方血量
              </div>
              <div class="font-mono text-lg font-bold text-gray-400 transition-colors">
                {{ clientStore.observation?.minions.health || "N/A" }}
              </div>
            </div>

            <!-- Interval -->
            <div
              class="group hover:border-acid-lime/30 flex min-w-[80px] flex-col items-center rounded px-3 py-1.5 backdrop-blur-sm transition-all"
            >
              <div
                class="group-hover:text-acid-lime/70 text-[10px] font-medium tracking-widest text-gray-600 uppercase transition-colors"
              >
                间隔
              </div>
              <div class="group-hover:text-acid-lime font-mono text-lg font-bold text-gray-400 transition-colors">
                {{ clientStore.thinkFrame }}
                <span class="text-xs">ms</span>
              </div>
            </div>

            <!-- Status -->
            <div
              class="group hover:border-acid-lime/30 flex min-w-[80px] flex-col items-center rounded px-3 py-1.5 backdrop-blur-sm transition-all"
            >
              <div
                class="group-hover:text-acid-lime/70 text-[10px] font-medium tracking-widest text-gray-600 uppercase transition-colors"
              >
                状态
              </div>
              <div
                :class="[
                  'font-mono text-lg font-bold transition-colors',
                  clientStore.playing ? 'text-acid-lime' : 'text-gray-500',
                ]"
              >
                {{ clientStore.playing ? "运行中" : "空闲" }}
              </div>
            </div>

            <!-- Visual Feed -->
            <div class="flex min-w-[100px] flex-col items-center rounded px-3 py-1.5 backdrop-blur-sm transition-all">
              <div class="text-[10px] font-medium tracking-widest text-gray-600 uppercase">视觉反馈</div>
              <div class="flex items-center gap-2 font-mono text-lg font-bold text-gray-400">
                <span
                  :class="[
                    'h-1.5 w-1.5 rounded-full',
                    clientStore.img ? 'bg-acid-lime shadow-[0_0_5px_#ccff00]' : 'bg-gray-600',
                  ]"
                ></span>
                实时流
              </div>
            </div>

            <!-- Refresh Button -->
            <Button @click="clientStore.updateImg">刷新</Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useClientStore } from "@/composables/useClient";

const clientStore = useClientStore();
</script>

<style scoped>
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: rgba(55, 65, 81, 0.3);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.5);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.7);
}

@keyframes glow {
  0%,
  100% {
    box-shadow: 0 0 2px rgba(59, 130, 246, 0.5);
  }
  50% {
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.8);
  }
}

.glow-border {
  animation: glow 2s ease-in-out infinite;
}

@keyframes dataFlow {
  0% {
    background-position: 0% 50%;
  }
  100% {
    background-position: 100% 50%;
  }
}

.data-flow-bg {
  background: linear-gradient(
    90deg,
    rgba(204, 255, 0, 0.05) 0%,
    rgba(255, 0, 204, 0.05) 25%,
    rgba(204, 255, 0, 0.05) 50%,
    rgba(255, 0, 204, 0.05) 75%,
    rgba(204, 255, 0, 0.05) 100%
  );
  background-size: 200% 100%;
  animation: dataFlow 10s linear infinite;
}

.glitch-text {
  text-shadow:
    1px 0 #ff00cc,
    -1px 0 #00ffcc;
}
</style>
