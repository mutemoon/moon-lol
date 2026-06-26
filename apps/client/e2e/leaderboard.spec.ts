import { test, expect } from "@playwright/test";
import { captureBrowserLogs, loginWithCode, uniquePhone } from "./helpers";

// 排行榜：只读页面。验证总榜/日增榜 Tab 切换的交互行为，不依赖具体榜单数据，
// 因此与后端是否已有上榜 Agent 无关，可稳定重复运行。

test.describe("排行榜", () => {
  test("可在「总排行」与「今日增量」两个榜单视图间切换", async ({ page }) => {
    captureBrowserLogs(page);

    // 排行榜接口在未登录时会触发 401 → 弹出登录框，故先以一次性账号登录。
    await page.goto("/rooms");
    await loginWithCode(page, uniquePhone());

    await page.goto("/leaderboard");

    const totalTab = page.getByRole("tab", { name: "总排行" });
    const dailyTab = page.getByRole("tab", { name: "今日增量" });

    // 默认停留在总排行。
    await expect(totalTab).toHaveAttribute("aria-selected", "true");
    await expect(dailyTab).toHaveAttribute("aria-selected", "false");

    // 切到今日增量后，选中态对调。
    await dailyTab.click();
    await expect(dailyTab).toHaveAttribute("aria-selected", "true");
    await expect(totalTab).toHaveAttribute("aria-selected", "false");

    // 切回总排行。
    await totalTab.click();
    await expect(totalTab).toHaveAttribute("aria-selected", "true");
  });
});
