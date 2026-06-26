import { test, expect } from "@playwright/test";
import {
  captureBrowserLogs,
  confirmDialog,
  createAgentPreset,
  createRoom,
  deleteAgentPreset,
  dissolveRoom,
  loginWithCode,
  selectOptionByText,
  uniqueId,
  uniquePhone,
} from "./helpers";

// 端到端冒烟：登录 → 备选手 → 建房编排 → 启动对局 → 观战 → 结束。
// 这是一条贯穿房间编排与对局生命周期的完整用户路径，刻意保持单用例的端到端形态；
// 更细粒度的单一职责校验拆在 agent / room / rank 各自的 spec。

test.describe("对局端到端冒烟", () => {
  test("编排选手进入房间后可启动对局，并能在观战页结束对局", async ({ page }) => {
    captureBrowserLogs(page);

    const phone = uniquePhone();
    const agentName = `E2E对战选手-${uniqueId()}`;
    const roomName = `E2E对局房间-${uniqueId()}`;
    let roomId = "";

    await page.goto("/heroes");
    await loginWithCode(page, phone);

    try {
      // 准备一名可编排的选手（启动对局只需槽位有选手，无需发布快照）。
      await createAgentPreset(page, agentName);

      // 建房并把选手编排进蓝方（Order）。
      roomId = await createRoom(page, roomName);
      await page.getByTestId("add-slot-blue").click();
      await selectOptionByText(page, "room-add-agent-select", agentName);
      await page.getByTestId("room-confirm-add-btn").click();
      // 蓝方槽位出现该选手，确认编排落地。
      await expect(page.getByText(agentName)).toBeVisible();

      // 启动对局，跳转到对应 match 的观战页。
      await page.getByTestId("room-start-match-btn").click();
      await expect(page).toHaveURL(/\/observe\/[a-f0-9-]{36}/);
      const matchId = page.url().split("/").pop()!;

      // 观战页头部展示本场 match id 前 8 位与「直播中」状态，绑定到刚启动的对局。
      await expect(page.getByText(matchId.slice(0, 8))).toBeVisible();
      await expect(page.getByText("直播中")).toBeVisible();

      // 结束对局：经二次确认弹窗后，状态变为「已中止」。
      await page.getByTestId("stop-match-btn").click();
      await confirmDialog(page);
      await expect(page.getByText("已中止")).toBeVisible();
    } finally {
      // 清理：解散房间（若已建）与删除选手，避免残留污染后续运行。
      if (roomId) {
        await page.goto(`/rooms/${roomId}`);
        // 对局结束后房间仍存在，房主可解散；解散按钮不可见则说明房间已不在，跳过。
        const dissolveBtn = page.getByTestId("room-dissolve-btn");
        if (await dissolveBtn.isVisible().catch(() => false)) {
          await dissolveRoom(page);
        }
      }
      await deleteAgentPreset(page, agentName);
    }
  });
});
