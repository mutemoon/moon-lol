// 模型供应商预设目录：数据整理自 cc-switch (https://github.com/farion1231/cc-switch)
// 的 claudeProviderPresets.ts，按本产品场景裁剪。预设仅预填 Base URL / 默认模型 /
// API 格式，API Key 由用户填写。所选预设均暴露 Anthropic 兼容端点（api_format=anthropic）。

import type { ApiFormat, ProviderCategory } from "../services/types";

export interface ProviderPreset {
  name: string;
  /** 供应商类型标识，用于去重与图标。 */
  presetType: string;
  baseUrl: string;
  apiFormat: ApiFormat;
  defaultModels: string[];
  websiteUrl?: string;
  apiKeyUrl?: string;
  icon?: string;
  iconColor?: string;
  category: ProviderCategory;
}

export const providerPresets: ProviderPreset[] = [
  {
    name: "智谱 BigModel",
    presetType: "zhipu",
    baseUrl: "https://open.bigmodel.cn/api/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["glm-5.1"],
    websiteUrl: "https://open.bigmodel.cn",
    apiKeyUrl: "https://www.bigmodel.cn/claude-code",
    icon: "zhipu",
    iconColor: "#0F62FE",
    category: "preset",
  },
  {
    name: "智谱 z.ai（海外）",
    presetType: "zhipu_en",
    baseUrl: "https://api.z.ai/api/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["glm-5.1"],
    websiteUrl: "https://z.ai",
    apiKeyUrl: "https://z.ai/subscribe",
    icon: "zhipu",
    iconColor: "#0F62FE",
    category: "preset",
  },
  {
    name: "DeepSeek",
    presetType: "deepseek",
    baseUrl: "https://api.deepseek.com/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["deepseek-v4-pro"],
    websiteUrl: "https://www.deepseek.com",
    icon: "deepseek",
    iconColor: "#4D6BFE",
    category: "preset",
  },
  {
    name: "火山方舟 Agentplan",
    presetType: "volcengine",
    baseUrl: "https://ark.cn-beijing.volces.com/api/coding",
    apiFormat: "anthropic",
    defaultModels: ["ark-code-latest"],
    websiteUrl: "https://www.volcengine.com/product/ark",
    icon: "volcengine",
    iconColor: "#1664FF",
    category: "preset",
  },
  {
    name: "豆包 Seed",
    presetType: "doubao",
    baseUrl: "https://ark.cn-beijing.volces.com/api/compatible",
    apiFormat: "anthropic",
    defaultModels: ["doubao-seed-2-1-pro"],
    websiteUrl: "https://www.volcengine.com/product/doubao",
    icon: "doubao",
    iconColor: "#1664FF",
    category: "preset",
  },
  {
    name: "百度千帆 Coding",
    presetType: "qianfan",
    baseUrl: "https://qianfan.baidubce.com/anthropic/coding",
    apiFormat: "anthropic",
    defaultModels: ["qianfan-code-latest"],
    websiteUrl: "https://cloud.baidu.com/product/qianfan_modelbuilder",
    icon: "baidu",
    iconColor: "#2932E1",
    category: "preset",
  },
  {
    name: "阿里百炼",
    presetType: "bailian",
    baseUrl: "https://dashscope.aliyuncs.com/apps/anthropic",
    apiFormat: "anthropic",
    defaultModels: [],
    websiteUrl: "https://bailian.console.aliyun.com",
    icon: "bailian",
    iconColor: "#624AFF",
    category: "preset",
  },
  {
    name: "阿里百炼 For Coding",
    presetType: "bailian_coding",
    baseUrl: "https://coding.dashscope.aliyuncs.com/apps/anthropic",
    apiFormat: "anthropic",
    defaultModels: [],
    websiteUrl: "https://bailian.console.aliyun.com",
    icon: "bailian",
    iconColor: "#624AFF",
    category: "preset",
  },
  {
    name: "Kimi",
    presetType: "kimi",
    baseUrl: "https://api.moonshot.cn/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["kimi-k2.7-code"],
    websiteUrl: "https://platform.moonshot.cn",
    icon: "kimi",
    iconColor: "#1D1D1F",
    category: "preset",
  },
  {
    name: "StepFun",
    presetType: "stepfun",
    baseUrl: "https://api.stepfun.com/step_plan",
    apiFormat: "anthropic",
    defaultModels: ["step-3.5-flash-2603"],
    websiteUrl: "https://platform.stepfun.com",
    icon: "stepfun",
    iconColor: "#0066FF",
    category: "preset",
  },
  {
    name: "MiniMax",
    presetType: "minimax",
    baseUrl: "https://api.minimaxi.com/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["MiniMax-M2.7"],
    websiteUrl: "https://platform.minimaxi.com",
    icon: "minimax",
    iconColor: "#FF6B00",
    category: "preset",
  },
  {
    name: "Longcat",
    presetType: "longcat",
    baseUrl: "https://api.longcat.chat/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["LongCat-Flash-Chat"],
    websiteUrl: "https://longcat.chat",
    icon: "longcat",
    iconColor: "#7C3AED",
    category: "preset",
  },
  {
    name: "百灵 BaiLing",
    presetType: "bailing",
    baseUrl: "https://api.tbox.cn/api/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["Ling-2.5-1T"],
    websiteUrl: "https://www.tbox.cn",
    icon: "bailing",
    iconColor: "#1A73E8",
    category: "preset",
  },
  {
    name: "小米 MiMo",
    presetType: "mimo",
    baseUrl: "https://api.xiaomimimo.com/anthropic",
    apiFormat: "anthropic",
    defaultModels: ["mimo-v2.5-pro"],
    websiteUrl: "https://xiaomimimo.com",
    icon: "mimo",
    iconColor: "#FF6900",
    category: "preset",
  },
  {
    name: "KAT-Coder",
    presetType: "katcoder",
    baseUrl: "https://vanchin.streamlake.ai/api/gateway/v1/endpoints/claude-code-proxy",
    apiFormat: "anthropic",
    defaultModels: ["KAT-Coder-Pro V1"],
    websiteUrl: "https://vanchin.streamlake.ai",
    icon: "katcoder",
    iconColor: "#111827",
    category: "preset",
  },
];

/** 平台模型哨兵：走平台网关（精粹计费），不存为供应商记录。 */
export const PLATFORM_PROVIDER_ID = "__platform__";
