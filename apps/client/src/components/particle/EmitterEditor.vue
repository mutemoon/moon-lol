<script setup lang="ts">
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { NumberField, NumberFieldContent, NumberFieldInput } from "@/components/ui/number-field";
import StochasticSamplerInput from "./StochasticSamplerInput.vue";
import VecInput from "./VecInput.vue";
import AlphaErosionEditor from "./AlphaErosionEditor.vue";
import { RotateCcwIcon, SlidersIcon, SparklesIcon, ImageIcon, LayersIcon, PlayIcon, BoxIcon } from "lucide-vue-next";

const props = defineProps<{
  emitter: any;
  initialEmitter?: any;
}>();

const emit = defineEmits<{
  (e: "change"): void;
  (e: "reset"): void;
  (e: "playSingle"): void;
}>();

function onFieldChange() {
  emit("change");
}

function resetEmitter() {
  emit("reset");
}

function onNumberFieldInput(key: string, val: number | undefined | null) {
  if (val === undefined || val === null || isNaN(val)) {
    props.emitter[key] = undefined;
  } else {
    props.emitter[key] = val;
  }
  onFieldChange();
}

function updateTexturePath(key: string, pathVal: string) {
  if (!props.emitter[key]) {
    props.emitter[key] = { path: pathVal };
  } else {
    props.emitter[key].path = pathVal;
  }
  onFieldChange();
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden bg-background">
    <!-- 工具条 -->
    <div class="flex items-center justify-between px-4 py-2 border-b bg-muted/30 shrink-0">
      <div class="flex items-center gap-2 text-xs font-semibold">
        <SlidersIcon class="size-4 text-primary" />
        <span>{{ $t('particles.emitterEditorTitle') }}</span>
        <span v-if="emitter.emitter_name" class="text-muted-foreground font-mono text-[11px]">
          ({{ emitter.emitter_name }})
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" class="h-7 px-2 text-xs gap-1 text-primary hover:text-primary hover:bg-primary/10 border-primary/30" @click="emit('playSingle')">
          <PlayIcon class="size-3.5" />
          {{ $t('particles.playSingleEmitter') }}
        </Button>
        <Button variant="ghost" size="sm" class="h-7 px-2 text-xs gap-1 text-muted-foreground hover:text-foreground" @click="resetEmitter">
          <RotateCcwIcon class="size-3.5" />
          {{ $t('particles.resetEmitter') }}
        </Button>
      </div>
    </div>

    <!-- 表单卡片主体 -->
    <ScrollArea class="flex-1 min-h-0">
      <div class="p-4 space-y-6 max-w-4xl mx-auto">
        <!-- 1. 基本参数 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 border-b pb-1 text-xs font-semibold text-primary uppercase tracking-wider">
            <SparklesIcon class="size-3.5" />
            {{ $t('particles.basicControls') }}
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 text-xs">
            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.emitterName') }}</Label>
              <Input
                v-model="emitter.emitter_name"
                class="h-7 text-xs"
                placeholder="Fire_Particle"
                @input="onFieldChange"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.emitterLifetime') }}</Label>
              <NumberField
                :model-value="emitter.lifetime ?? undefined"
                :step="0.1"
                :min="0"
                @update:model-value="onNumberFieldInput('lifetime', $event)"
              >
                <NumberFieldContent class="h-7">
                  <NumberFieldInput class="h-7 text-xs font-mono" :placeholder="$t('particles.lifetimePlaceholder')" />
                </NumberFieldContent>
              </NumberField>
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.numFrames') }}</Label>
              <NumberField
                :model-value="emitter.num_frames ?? 1"
                :step="1"
                :min="1"
                @update:model-value="onNumberFieldInput('num_frames', $event)"
              >
                <NumberFieldContent class="h-7">
                  <NumberFieldInput class="h-7 text-xs font-mono" />
                </NumberFieldContent>
              </NumberField>
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.blendMode') }}</Label>
              <NumberField
                :model-value="emitter.blend_mode ?? 0"
                :step="1"
                :min="0"
                @update:model-value="onNumberFieldInput('blend_mode', $event)"
              >
                <NumberFieldContent class="h-7">
                  <NumberFieldInput class="h-7 text-xs font-mono" />
                </NumberFieldContent>
              </NumberField>
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.alphaRef') }}</Label>
              <NumberField
                :model-value="emitter.alpha_ref ?? 0"
                :step="1"
                :min="0"
                :max="255"
                @update:model-value="onNumberFieldInput('alpha_ref', $event)"
              >
                <NumberFieldContent class="h-7">
                  <NumberFieldInput class="h-7 text-xs font-mono" />
                </NumberFieldContent>
              </NumberField>
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.rate') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.rate"
                type="number"
                :step="0.5"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.particleLifetime') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.particle_lifetime"
                type="number"
                :step="0.1"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.bindWeight') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.bind_weight"
                type="number"
                :step="0.1"
                @update:model-value="onFieldChange"
              />
            </div>
          </div>
        </section>

        <!-- 2. 标志与布尔开关 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 border-b pb-1 text-xs font-semibold text-primary uppercase tracking-wider">
            <LayersIcon class="size-3.5" />
            {{ $t('particles.renderFlags') }}
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3 bg-muted/20 p-3 rounded-md border">
            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_is_single_particle`"
                :model-value="!!emitter.is_single_particle"
                @update:model-value="emitter.is_single_particle = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_is_single_particle`" class="text-xs cursor-pointer">
                {{ $t('particles.singleParticle') }}
              </Label>
            </div>

            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_is_uniform_scale`"
                :model-value="!!emitter.is_uniform_scale"
                @update:model-value="emitter.is_uniform_scale = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_is_uniform_scale`" class="text-xs cursor-pointer">
                {{ $t('particles.uniformScale') }}
              </Label>
            </div>

            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_is_random_start_frame`"
                :model-value="!!emitter.is_random_start_frame"
                @update:model-value="emitter.is_random_start_frame = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_is_random_start_frame`" class="text-xs cursor-pointer">
                {{ $t('particles.randomStartFrame') }}
              </Label>
            </div>

            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_is_local_orientation`"
                :model-value="!!emitter.is_local_orientation"
                @update:model-value="emitter.is_local_orientation = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_is_local_orientation`" class="text-xs cursor-pointer">
                {{ $t('particles.localOrientation') }}
              </Label>
            </div>

            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_is_direction_oriented`"
                :model-value="!!emitter.is_direction_oriented"
                @update:model-value="emitter.is_direction_oriented = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_is_direction_oriented`" class="text-xs cursor-pointer">
                {{ $t('particles.directionOriented') }}
              </Label>
            </div>

            <div class="flex items-center gap-2">
              <Checkbox
                :id="`em_${emitter.emitter_name}_soft_particle_definition`"
                :model-value="!!emitter.soft_particle_definition"
                @update:model-value="emitter.soft_particle_definition = Boolean($event); onFieldChange()"
              />
              <Label :for="`em_${emitter.emitter_name}_soft_particle_definition`" class="text-xs cursor-pointer">
                {{ $t('particles.softParticle') }}
              </Label>
            </div>
          </div>
        </section>

        <!-- 3. 动态采样器动态调整 (Samplers) -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 border-b pb-1 text-xs font-semibold text-primary uppercase tracking-wider">
            <SlidersIcon class="size-3.5" />
            {{ $t('particles.samplers') }}
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.emitterPosition') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.emitter_position"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthVelocity') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_velocity"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthAcceleration') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_acceleration"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthRotation0') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_rotation0"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthScale0') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_scale0"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.scale0') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.scale0"
                type="vec3"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthColor') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_color"
                type="vec4"
                :is-color="true"
                :label="$t('particles.birthColor')"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.color') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.color"
                type="vec4"
                :is-color="true"
                :label="$t('particles.color')"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthUvOffset') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_uv_offset"
                type="vec2"
                @update:model-value="onFieldChange"
              />
            </div>

            <div class="space-y-1 p-2.5 rounded border bg-muted/10">
              <Label class="text-[11px] font-semibold text-foreground">{{ $t('particles.birthUvScrollRate') }}</Label>
              <StochasticSamplerInput
                v-model="emitter.birth_uv_scroll_rate"
                type="vec2"
                @update:model-value="onFieldChange"
              />
            </div>
          </div>
        </section>

        <!-- 4. 贴图资源路径 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 border-b pb-1 text-xs font-semibold text-primary uppercase tracking-wider">
            <ImageIcon class="size-3.5" />
            {{ $t('particles.texturesAndMaterials') }}
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.texture') }}</Label>
              <Input
                :model-value="emitter.texture?.path ?? ''"
                class="h-7 text-xs font-mono"
                placeholder="ASSETS/Textures/particles/fire.dds"
                @update:model-value="updateTexturePath('texture', String($event))"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.particleColorTexture') }}</Label>
              <Input
                :model-value="emitter.particle_color_texture?.path ?? ''"
                class="h-7 text-xs font-mono"
                placeholder="ASSETS/Textures/particles/color.dds"
                @update:model-value="updateTexturePath('particle_color_texture', String($event))"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.paletteDefinition') }}</Label>
              <Input
                :model-value="emitter.palette_definition?.path ?? ''"
                class="h-7 text-xs font-mono"
                placeholder="ASSETS/Textures/particles/palette.dds"
                @update:model-value="updateTexturePath('palette_definition', String($event))"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.reflectionDefinition') }}</Label>
              <Input
                :model-value="emitter.reflection_definition?.path ?? ''"
                class="h-7 text-xs font-mono"
                placeholder="ASSETS/Textures/particles/reflection.dds"
                @update:model-value="updateTexturePath('reflection_definition', String($event))"
              />
            </div>

            <div class="space-y-1">
              <Label class="text-[11px] text-muted-foreground">{{ $t('particles.texDiv') }}</Label>
              <VecInput
                :model-value="emitter.tex_div ?? [1, 1]"
                :dimensions="2"
                :labels="['U', 'V']"
                :step="1"
                @update:model-value="emitter.tex_div = $event; onFieldChange()"
              />
            </div>
          </div>
        </section>

        <!-- 5. 高级通道与子对象定义 -->
        <section class="space-y-3">
          <div class="flex items-center gap-2 border-b pb-1 text-xs font-semibold text-primary uppercase tracking-wider">
            <BoxIcon class="size-3.5" />
            {{ $t('particles.advancedDefinitions') }}
          </div>

          <div class="grid grid-cols-1 gap-4">
            <!-- Alpha 侵蚀消融定义卡片 -->
            <AlphaErosionEditor
              v-model="emitter.alpha_erosion_definition"
              @change="onFieldChange"
            />
          </div>
        </section>
      </div>
    </ScrollArea>
  </div>
</template>
