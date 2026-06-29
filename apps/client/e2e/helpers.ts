import { type Page, expect } from "@playwright/test";

// 共享 E2E 辅助方法。
//
// 设计原则（见 client-dev-with-browser 技能）：
//   - 通过 data-testid 定位，避免样式/DOM 变更导致用例失效；
//   - 用本次运行唯一的业务名称（账号 / Agent / 房间）天然隔离会持久化的后端状态；
//   - 重复流程（登录、建 Agent、建房、确认弹窗、清理）抽成 helper，让用例聚焦单一用户路径。

/** 测试环境固定验证码（短信服务接入前的占位，见 AuthDialog.vue）。 */
const TEST_CODE = "111111";

/** 把浏览器 console 与未捕获异常透传到测试输出，便于失败时定位前端报错。 */
export function captureBrowserLogs(page: Page, tag = "BROWSER"): void {
  page.on("console", (msg) => console.log(`${tag} CONSOLE [${msg.type()}]:`, msg.text()));
  page.on("pageerror", (err) => console.error(`${tag} PAGE ERROR:`, err.message));
}

/**
 * 生成本次运行唯一的标识，用于拼装 Agent / 房间等业务名称。
 * 叠加随机段，避免并行 worker 在同一毫秒内碰撞。
 */
export function uniqueId(): string {
  return `${Date.now()}-${Math.floor(Math.random() * 1e4)}`;
}

/**
 * 生成本次运行唯一的 11 位测试手机号。
 *
 * 多数在线用例会污染按用户持久化、且无客户端清理入口的全局状态（典型：rank 队列
 * 没有「退出队列」的 HTTP 接口）。为每个用例分配独立账号，可把这类污染限制在一次性
 * 账号内，互不串扰，也避免「我的房间 / 我的队列」混入其它用例的残留数据。
 */
export function uniquePhone(): string {
  const t = (Date.now() % 1e4).toString().padStart(4, "0");
  const r = Math.floor(Math.random() * 1e4).toString().padStart(4, "0");
  return `139${t}${r}`;
}

/** 转义动态文本中的正则元字符，供 getByRole({ name: RegExp }) 子串匹配使用。 */
function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * 验证码登录 / 自动注册，并等待登录态在侧边栏账号区生效。
 * 调用前页面应已停留在某个会弹出登录框的受保护路由。
 */
export async function loginWithCode(page: Page, phone: string): Promise<void> {
  const dialog = page.getByRole("dialog", { name: "登录 / 注册" });
  await expect(dialog).toBeVisible();
  await page.getByTestId("login-phone-input").fill(phone);
  await page.getByTestId("login-code-input").fill(TEST_CODE);
  await page.getByTestId("login-submit-btn").click();
  // 登录成功后弹窗关闭、侧边栏展示当前手机号——作为登录态的稳定锚点。
  await expect(dialog).not.toBeVisible();
  await expect(page.getByText(phone, { exact: true })).toBeVisible();
}

/**
 * 打开 shadcn Select 触发器并按选项文本（子串）选择。
 * Select 选项文本通常是「名称 · 附加信息」，因此用子串正则而非精确名匹配。
 */
export async function selectOptionByText(
  page: Page,
  triggerTestId: string,
  optionText: string,
): Promise<void> {
  await page.getByTestId(triggerTestId).click();
  await page.getByRole("option", { name: new RegExp(escapeRegExp(optionText)) }).click();
}

/**
 * 在 /heroes 新建并保存一个 LLM Agent 预设（保存会在云端创建对应 Agent）。
 * 保存后停留在「编辑已存预设」态，可继续发布快照。
 * 调用前需已登录并位于 /heroes。
 */
export async function createAgentPreset(
  page: Page,
  name: string,
  prompt = "E2E 自动测试生成的 AI 决策 Prompt",
): Promise<void> {
  await page.getByTestId("new-preset-btn").click();
  await page.getByTestId("preset-name-input").fill(name);
  await page.getByTestId("preset-prompt-input").fill(prompt);
  await page.getByTestId("preset-save-btn").click();
  // 删除按钮仅在 editingName 存在（即预设已落库）时渲染——以它出现作为保存成功的
  // 行为信号，而非依赖「已保存」这类可能来自旧渲染的通用文案。
  await expect(page.getByTestId("preset-delete-btn")).toBeVisible();
  // 保存成功的状态文案在云端 Agent 同步（loadCloudAgents）完成后才置位；等它出现
  // 可确保后续发布快照时 currentCloudAgent 已就绪，避免发布按钮空转。
  // 用绑定到该状态 span 的 data-testid 定位，而非匹配「已保存」这类通用文案。
  await expect(page.getByTestId("preset-save-success")).toBeVisible();
}

/**
 * 在编辑态发布一个参赛快照，并确认发布 Tab 上出现版本徽章。
 * 因为编辑视图始终对应当前这个唯一 Agent，所以版本号在此作用域内绑定到本次发布。
 */
export async function publishSnapshot(page: Page): Promise<void> {
  await page.getByTestId("preset-tab-publish").click();
  await page.getByTestId("preset-publish-btn").click();
  // 发布成功后，发布 Tab 文案追加版本徽章（如「发布与快照 v1」），且历史列表顶部
  // 出现「最新」标记——后者明确绑定到刚发布的这条快照。
  await expect(page.getByTestId("preset-tab-publish")).toContainText(/v\d+/);
  // 用绑定到列表首项「最新」徽章的 data-testid 断言，而非全局匹配「当前最新」文案。
  await expect(page.getByTestId("snapshot-latest-badge")).toBeVisible();
}

/**
 * 创建房间并进入房间详情页，返回房间 id。
 * 调用前需已登录。
 */
export async function createRoom(page: Page, name: string): Promise<string> {
  await page.goto("/rooms");
  await page.getByTestId("create-room-btn").click();
  await page.getByTestId("create-room-name-input").fill(name);
  await page.getByTestId("create-room-submit-btn").click();
  await expect(page).toHaveURL(/\/rooms\/[a-f0-9-]{36}/);
  return page.url().split("/").pop()!;
}

/** 点击自定义确认弹窗的「确认」按钮。 */
export async function confirmDialog(page: Page): Promise<void> {
  await page.getByTestId("confirm-dialog-submit-btn").click();
}

/**
 * 房主解散当前房间并等待跳回 /rooms。用于用例 teardown。
 * roomId 已知时一并断言离开的是该房间。
 */
export async function dissolveRoom(page: Page): Promise<void> {
  await page.getByTestId("room-dissolve-btn").click();
  await confirmDialog(page);
  await expect(page).toHaveURL(/\/rooms$/);
}

/**
 * 删除指定 Agent 预设（同时删除云端 Agent）。用于用例 teardown。
 * 会导航到 /heroes 浏览态后按唯一名称定位卡片。
 */
export async function deleteAgentPreset(page: Page, name: string): Promise<void> {
  await page.goto("/heroes");
  await page.getByText(name, { exact: true }).click();
  await page.getByTestId("preset-delete-btn").click();
  await page.getByTestId("preset-delete-confirm-btn").click();
  await expect(page.getByText(name, { exact: true })).not.toBeVisible();
}
