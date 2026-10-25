import { expect, test } from "playwright/test";

test("测试身份登录、读取档案、刷新会话并退出", async ({ page }) => {
	await page.goto("/dashboard");
	await expect(page).toHaveURL(/\/login$/);
	await page.getByLabel("测试身份").selectOption("teacher");
	await page.getByRole("button", { name: "登录", exact: true }).click();
	await expect(
		page.getByRole("heading", { name: "你好，测试教师" }),
	).toBeVisible();
	await page.getByRole("link", { name: "个人档案", exact: true }).click();
	await expect(
		page.locator("dd").filter({ hasText: "DEMO-91001" }),
	).toBeVisible();
	await page.reload();
	await expect(
		page.locator("dd").filter({ hasText: "数学教研组" }),
	).toBeVisible();
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect(page).toHaveURL(/\/login$/);
	await page.goto("/dashboard");
	await expect(page).toHaveURL(/\/login$/);
});

test("移动端导航不遮挡工作区", async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await page.goto("/login");
	await page.getByLabel("测试身份").selectOption("leader");
	await page.getByRole("button", { name: "登录", exact: true }).click();
	await expect(
		page.getByRole("heading", { name: "你好，测试组长" }),
	).toBeVisible();
	await page.getByRole("button", { name: "展开导航" }).click();
	await page.getByRole("link", { name: "个人档案", exact: true }).click();
	await expect(
		page.locator("dd").filter({ hasText: "DEMO-91002" }),
	).toBeVisible();
	expect(
		await page.evaluate(
			() => document.documentElement.scrollWidth <= window.innerWidth,
		),
	).toBe(true);
});
