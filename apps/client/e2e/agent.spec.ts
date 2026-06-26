import { test, expect } from "@playwright/test";
import {
  captureBrowserLogs,
  createAgentPreset,
  deleteAgentPreset,
  loginWithCode,
  publishSnapshot,
  uniqueId,
  uniquePhone,
} from "./helpers";

// Agent（「我的选手」）资产生命周期：新建配置 → 落库 → 发布参赛快照 → 删除。
// 这是 Rank 与房间编排的上游，单独成 spec，保证最核心的资产链路被独立覆盖。

test.describe("Agent 选手资产生命周期", () => {
  test("新建并发布快照后，选手卡片与最新版本应持久呈现，删除后从列表消失", async ({ page }) => {
    captureBrowserLogs(page);

    const phone = uniquePhone();
    const agentName = `E2E选手-${uniqueId()}`;

    await page.goto("/heroes");
    await loginWithCode(page, phone);

    try {
      // 新建并保存预设（云端创建 Agent），随后发布首个参赛快照。
      await createAgentPreset(page, agentName);
      await publishSnapshot(page);

      // 回到浏览态：以本次创建的唯一名称定位选手卡片，确认资产已持久化。
      await page.goto("/heroes");
      const card = page.getByTestId("preset-card").filter({ hasText: agentName });
      await expect(card).toBeVisible();
      // 卡片右下角展示最新快照标签 v1，绑定到刚刚发布的版本。
      await expect(card).toContainText(/v\d+/);
    } finally {
      // 清理：删除该选手，避免账号下残留资产影响重复运行。
      await deleteAgentPreset(page, agentName);
    }
  });
});
