import axios from "axios";
import ollama from "ollama";
import type { ChatCompletionMessageFunctionToolCall } from "openai/resources";

const baseURL = import.meta.env.VITE_BASE_URL;

const http = axios.create({
  baseURL,
});

export function useClient() {
  const img = ref<string>();

  async function updateImg() {
    try {
      await http.get("/render");
      img.value = `${baseURL}/render?timestamp=${Date.now()}`;
    } catch (error) {
      await new Promise((resolve) => setTimeout(resolve, 50));
      updateImg();
    }
  }

  async function step() {
    try {
      const res = await http.get("/observe");
      const observe: {
        position: [number, number];
        minions: {
          entity: number;
          position: [number, number];
        };
      } = res.data;
      //   const action = { Move: [data.position[0], data.position[1] - 10] };
      //   const action = { Attack: observe.minions.entity };

      const res2 = await ollama.chat({
        model: "qwen3:4b",
        messages: [
          {
            role: "user",
            content: `${JSON.stringify(observe)} 你是剑姬，这是你观察到的游戏状态，当你在敌人的破绽方向对敌人造成伤害时，敌人会受到额外的 5% 的真实伤害，例如当破绽在上边时，你的 z 坐标需要小于敌人的 z 坐标，因为 z 越小，方位越靠上，同理 x 越小越靠左，这是你的被动技能，请你击杀目标`,
          },
        ],
        think: true,
        stream: false,
        tools: [
          {
            type: "function",
            function: {
              description: "对目标普通攻击",
              name: "Attack",
              parameters: {
                type: "object",
                properties: { entity: { type: "number", description: "目标实体ID" } },
                required: ["entity"],
              },
            },
          },
          {
            type: "function",
            function: {
              description: "移动到指定世界坐标",
              name: "Move",
              parameters: {
                type: "object",
                properties: {
                  position: { type: "array", items: { type: "number" }, description: "目标的世界坐标，格式为[x, y]" },
                },
                required: ["position"],
              },
            },
          },
        ],
      });

      console.log(res2);

      let action;
      const tool_call = res2.message.tool_calls?.at(0)!;
      console.log(tool_call);

      if (tool_call.function.name == "Attack") {
        const args = tool_call.function.arguments;
        action = {
          Attack: args.entity,
        };
      }
      if (tool_call.function.name == "Move") {
        const args = tool_call.function.arguments;
        action = {
          Move: args.position,
        };
      }
      console.log(action);

      await http.post("/step", action);
      updateImg();
    } catch (error) {
      await http.post("/step", { Stop: null });
      updateImg();
    }
  }

  async function observe() {
    const res = await http.get("/observe");
    console.log(res.data);
  }

  return { img, updateImg, step, observe };
}
