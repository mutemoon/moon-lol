import { ref } from "vue";
import axios from "axios";
import ollama, { type Message, type Tool } from "ollama";
import { defineStore } from "pinia";

const baseURL = import.meta.env.VITE_BASE_URL;

const http = axios.create({
  baseURL,
});

export type Entity = number;

export type Vec2 = [number, number];

export type Action = { Move: Vec2 } | { Attack: Entity };

export type Observe = {
  position: Vec2;
  minions: {
    entity: Entity;
    position: Vec2;
    health: number;
  };
};

const tools: Tool[] = [
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
      description: "移动到指定坐标",
      name: "Move",
      parameters: {
        type: "object",
        properties: {
          position: { type: "array", items: { type: "number" }, description: "目标位置，格式为[x, y]" },
        },
        required: ["position"],
      },
    },
  },
  {
    type: "function",
    function: {
      description: "什么也不做",
      name: "Nothing",
    },
  },
];

async function getObservation() {
  const res = await http.get("/observe");
  return res.data as Observe;
}

async function getAction(content: string, onNewMessage: (message: Message) => void) {
  const res = await ollama.chat({
    model: "qwen3:8b",
    messages: [
      {
        role: "user",
        content,
      },
    ],
    think: true,
    stream: true,
    tools,
  });

  let action: Action | undefined;

  for await (const chunk of res) {
    onNewMessage(chunk.message);

    const tool_call = chunk.message.tool_calls?.at(0);

    if (tool_call) {
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
    }
  }

  return action;
}

export const useClientStore = defineStore(
  "client",
  () => {
    const frame = ref(0);
    const thinkFrame = ref(10);
    const playing = ref<boolean>(false);
    const img = ref<string>();
    const observation = ref<Observe>();
    const action = ref<Action>();
    const prompt = ref<string>(
      `你是剑姬，这是你观察到的游戏状态。

游戏坐标：[x, y] 描述的是游戏物体在水平上的坐标。

你的攻击范围是 100

你的被动技能：
- 当你在敌人的破绽方向对敌人造成伤害时，敌人会受到额外的 5% 的真实伤害。
- 破绽只在激活状态有效，也就是 active_timer 的 finish 为 true 时，才能打破破绽。
说明：
- 要判断是否处于破绽内，不能只根据 x 判断左右或者只根据 y 判断上下，而是参考下面这段代码：
pub fn is_in_direction(source: Vec2, target: Vec2, direction: &Direction) -> bool {
    let delta_x = source.x - target.x;
    let delta_y = source.y - target.y;

    let abs_delta_x = delta_x.abs();
    let abs_delta_y = delta_y.abs();

    match direction {
        Direction::Up => delta_y > 0.0 && abs_delta_y > abs_delta_x,

        Direction::Down => delta_y < 0.0 && abs_delta_y > abs_delta_x,

        Direction::Right => delta_x > 0.0 && abs_delta_x > abs_delta_y,

        Direction::Left => delta_x < 0.0 && abs_delta_x > abs_delta_y,
    }
}
- 当你处于攻击前摇时，最好不要采取行动，否则普通攻击可能会被取消。
- 尽量移动到敌人的破绽方向的地方，并且是刚好能够攻击到敌人的地方再攻击敌人。

请你尽快击杀目标。`,
    );
    const message = ref<string>();

    async function updateImg() {
      await http.get("/render");
      img.value = `${baseURL}/render?timestamp=${Date.now()}`;
    }

    async function step(think: boolean = true) {
      message.value = "";

      // const afterStep = async () => {
      //   await new Promise((resolve) => setTimeout(resolve, 200));
      //   await updateImg();
      // };

      if (!think) {
        await http.post("/step");
        return;
      }

      try {
        observation.value = await getObservation();
        action.value = await getAction(`${JSON.stringify(observation.value)} ${prompt.value}`, (msg) => {
          if (msg.thinking) {
            message.value += msg.thinking;
          }
          message.value += msg.content;
        });
        if (action.value) {
          await http.post("/step", action.value);
        }
      } catch (error) {
        await http.post("/step");
      }
    }

    async function observe() {
      const res = await http.get("/observe");
      console.log(res.data);
    }

    async function play() {
      playing.value = true;
      await _play();
    }

    async function _play() {
      if (!playing.value) return;
      const think = frame.value % thinkFrame.value == 0;
      console.log(think);

      await step(think);
      frame.value++;
      await _play();
    }

    async function stop() {
      playing.value = false;
    }

    return {
      action,
      img,
      prompt,
      message,
      observation,
      frame,
      thinkFrame,
      playing,
      updateImg,
      step,
      observe,
      play,
      stop,
    };
  },
  {
    persist: {
      pick: ["prompt"],
    },
  },
);
