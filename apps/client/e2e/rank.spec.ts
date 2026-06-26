import { test, expect } from "@playwright/test";
import {
  captureBrowserLogs,
  createAgentPreset,
  deleteAgentPreset,
  loginWithCode,
  publishSnapshot,
  selectOptionByText,
  uniqueId,
  uniquePhone,
} from "./helpers";

// Rank 报名：选手发布快照后进入匹配池。
//
// 隔离说明：rank 队列按用户持久化，且没有面向客户端的「退出队列」HTTP 接口
// （后端仅在撮合/清理时内部 dequeue）。因此本用例用一次性账号报名——该账号的
// 队列初始为空，入队后唯一的队列项必然是本次创建的选手；用例末尾删除选手以释放
// 云端资产，残留的队列项也仅挂在这个一次性账号下。

test.describe("Rank 报名匹配池", () => {
  test("发布快照的选手报名后，应出现在当前账号的排队列表中", async ({ page }) => {
    captureBrowserLogs(page);

    const phone = uniquePhone();
    const agentName = `E2E排队选手-${uniqueId()}`;

    await page.goto("/heroes");
    await loginWithCode(page, phone);

    try {
      // 先在 /heroes 备好一名已发布参赛快照的选手。
      await createAgentPreset(page, agentName);
      await publishSnapshot(page);

      // 报名：选择该选手与其首个快照（v1），加入匹配池。
      await page.goto("/rank");
      // 报名前队列应为空，作为后续入队断言的基线。
      await expect(page.getByText("当前未在任何队列中。")).toBeVisible();

      await selectOptionByText(page, "rank-agent-select", agentName);
      await selectOptionByText(page, "rank-snapshot-select", "v1");
      await page.getByTestId("rank-enqueue-btn").click();

      // 入队成功：一次性账号队列中出现唯一一项，可见退出按钮且模式为 top_solo。
      await expect(page.getByTestId("rank-dequeue-btn")).toHaveCount(1);
      await expect(page.getByTestId("rank-dequeue-btn")).toBeVisible();
      await expect(page.getByText("当前未在任何队列中。")).not.toBeVisible();
      await expect(page.getByText("top_solo")).toBeVisible();
    } finally {
      // 清理：删除选手，释放云端 Agent。
      await deleteAgentPreset(page, agentName);
    }
  });
});
