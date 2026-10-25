import path from "node:path";
import { expect, test } from "playwright/test";

const evidence = path.resolve(import.meta.dirname, "fixtures/evidence.png");
async function login(page: import("playwright/test").Page) {
	await page.goto("/login");
	await page.getByLabel("测试身份").selectOption("teacher");
	await page.getByRole("button", { name: "登录", exact: true }).click();
	await expect(page).toHaveURL(/\/dashboard$/);
}

test("公开课：保存、修改、上传、提交、材料预览与列表筛选", async ({ page }) => {
	await login(page);
	await page.goto("/achievements/new");
	const title = `P1公开课-${Date.now()}`;
	await page.locator("#title").fill(title);
	await page.locator("#subject_code").selectOption("math");
	await page.locator("#occurred_at").fill("2026-10-18");
	await page
		.locator("#academic_term_id")
		.selectOption({ label: "2026—2027 第一学期" });
	await page.locator("#venue").fill("综合楼402");
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	await page.getByRole("button", { name: "保存草稿" }).click();
	await expect(page.getByRole("heading", { name: title })).toBeVisible();
	await page.getByRole("link", { name: "编辑成果" }).click();
	await page.locator("#venue").fill("学术报告厅");
	await page.getByRole("button", { name: "保存草稿" }).click();
	await expect(
		page.getByText("学术报告厅", { exact: true }).first(),
	).toBeVisible();
	await page.getByRole("button", { name: "提交审核" }).click();
	await expect(
		page.locator(".record-meta").getByText("审核中", { exact: true }),
	).toBeVisible();
	await expect(page.locator(".revision-row")).toHaveCount(3);
	await expect(page.getByRole("link", { name: "编辑成果" })).toHaveCount(0);
	const preview = await page
		.getByRole("link", { name: "预览evidence.png" })
		.getAttribute("href");
	const proof = await page.request.get(preview ?? "invalid");
	expect(proof.status()).toBe(200);
	expect(proof.headers()["content-type"]).toContain("image/png");
	await page.getByRole("link", { name: "我的成果", exact: true }).click();
	await page.getByLabel("搜索成果名称").fill(title);
	await page.getByLabel("搜索成果名称").press("Enter");
	await expect(
		page.getByRole("link", { name: title, exact: true }),
	).toBeVisible();
});

test("课题研究：金额与人员顺序完整往返", async ({ page }) => {
	await login(page);
	await page.goto("/achievements/new?type=research_project");
	const title = `P1课题-${Date.now()}`;
	await page.locator("#title").fill(title);
	await page.getByLabel("项目经费（元）").fill("123456789.01");
	await page.getByLabel("项目编号").fill("P1-2026-001");
	await page.getByRole("button", { name: "添加相关人员" }).click();
	await page.getByLabel("关联教师").selectOption("91001");
	await page.getByLabel("人员角色").selectOption("host");
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	await page.getByRole("button", { name: "提交审核" }).click();
	await expect(page.getByRole("heading", { name: title })).toBeVisible();
	await expect(
		page.locator(".record-meta").getByText("审核中", { exact: true }),
	).toBeVisible();
	await expect(
		page.locator("dd").filter({ hasText: "123456789.01" }).first(),
	).toBeVisible();
	await expect(
		page.getByRole("cell", { name: "主持人", exact: true }),
	).toBeVisible();
});

test("手机端可以选择全部类型并上传材料", async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await login(page);
	await page.goto("/achievements/new");
	await expect(page.getByLabel("成果类型").locator("option")).toHaveCount(15);
	await page.getByLabel("成果类型").selectOption("student_award");
	await page.locator("#title").fill(`手机申报-${Date.now()}`);
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	expect(
		await page.evaluate(
			() => document.documentElement.scrollWidth <= window.innerWidth,
		),
	).toBe(true);
	await page.getByRole("button", { name: "提交审核" }).click();
	await expect(
		page.locator(".record-meta").getByText("审核中", { exact: true }),
	).toBeVisible();
});
