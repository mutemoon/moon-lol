import { test, expect } from "@playwright/test";
import {
  captureBrowserLogs,
  confirmDialog,
  createRoom,
  dissolveRoom,
  loginWithCode,
  uniqueId,
  uniquePhone,
} from "./helpers";

// 房间生命周期：创建、从大厅/我的房间重新进入、成员加入与离开、解散。
// 覆盖历史上「点击已加入房间卡片静默失败」的回归点（见用例内注释）。

test.describe("房间生命周期", () => {
  test("房主创建房间后，可从「我的房间」重新进入并解散", async ({ page }) => {
    captureBrowserLogs(page);

    const phone = uniquePhone();
    const roomName = `E2E房间-${uniqueId()}`;

    await page.goto("/rooms");
    await loginWithCode(page, phone);

    // 创建房间并记录其 id，后续断言均复用该 id，确保进入的是同一资源。
    const roomId = await createRoom(page, roomName);

    try {
      // 回大厅后切到「我的房间」，点击卡片应能重新进入房间详情。
      // 该路径曾因「已是成员仍重复 join」而静默失败，这里作为回归保护。
      await page.goto("/rooms");
      await page.getByTestId("rooms-refresh-btn").click();
      await page.getByTestId("rooms-tab-mine").click();

      const roomCard = page.getByTestId("room-card").filter({ hasText: roomName });
      await expect(roomCard).toBeVisible();
      await roomCard.click();
      await expect(page).toHaveURL(new RegExp(`/rooms/${roomId}$`));
    } finally {
      // 清理：解散房间，避免大厅与「我的房间」累积同名残留。
      await dissolveRoom(page);
    }
  });

  test("非房主成员可从大厅加入房间，并仅能离开而非解散", async ({ browser }) => {
    const ownerPhone = uniquePhone();
    const memberPhone = uniquePhone();
    const roomName = `E2E多人房间-${uniqueId()}`;

    const ownerContext = await browser.newContext();
    const memberContext = await browser.newContext();
    const ownerPage = await ownerContext.newPage();
    const memberPage = await memberContext.newPage();
    captureBrowserLogs(ownerPage, "OWNER");
    captureBrowserLogs(memberPage, "MEMBER");

    try {
      // 房主：登录并创建一个公开到大厅的房间。
      await ownerPage.goto("/rooms");
      await loginWithCode(ownerPage, ownerPhone);
      const roomId = await createRoom(ownerPage, roomName);

      // 成员：登录后从公开大厅找到该房间并加入。
      await memberPage.goto("/rooms");
      await loginWithCode(memberPage, memberPhone);
      await memberPage.getByTestId("rooms-tab-lobby").click();
      await memberPage.getByTestId("rooms-refresh-btn").click();
      const roomCard = memberPage.getByTestId("room-card").filter({ hasText: roomName });
      await expect(roomCard).toBeVisible();
      await roomCard.click();
      await expect(memberPage).toHaveURL(new RegExp(`/rooms/${roomId}$`));

      // 成员视角：有「离开房间」，无「解散房间」（解散是房主特权）。
      await expect(memberPage.getByTestId("room-leave-btn")).toBeVisible();
      await expect(memberPage.getByTestId("room-dissolve-btn")).not.toBeVisible();

      // 成员离开后回到大厅。
      await memberPage.getByTestId("room-leave-btn").click();
      await confirmDialog(memberPage);
      await expect(memberPage).toHaveURL(/\/rooms$/);

      // 清理：房主解散房间。
      await dissolveRoom(ownerPage);
    } finally {
      await ownerContext.close();
      await memberContext.close();
    }
  });
});
