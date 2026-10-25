import { readFileSync } from "node:fs";
import path from "node:path";
import { expect, type Page, test } from "playwright/test";

const evidence = path.resolve(import.meta.dirname, "fixtures/evidence.png");
async function login(page: Page) {
	await page.goto("/login");
	await page.getByLabel("测试身份").selectOption("teacher");
	await page.getByRole("button", { name: "登录", exact: true }).click();
	await expect(page).toHaveURL(/\/dashboard$/);
}
async function session(page: Page) {
	const response = await page.request.get("/api/v1/auth/me");
	expect(response.ok()).toBe(true);
	return (await response.json()).data;
}
async function createDraft(page: Page, title: string) {
	const auth = await session(page);
	const response = await page.request.post("/api/v1/achievements", {
		headers: {
			Origin: new URL(page.url()).origin,
			"X-CSRF-Token": auth.csrf_token,
		},
		data: { type_code: "open_class", schema_version: 1, title, attributes: {} },
	});
	expect(response.ok()).toBe(true);
	return (await response.json()).data;
}

test("未保存内容在取消切换、离开、后退和退出时保留，类型跟随地址", async ({
	page,
}) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	await page.locator("#title").fill("尚未保存的公开课");
	page.once("dialog", (dialog) => dialog.dismiss());
	await page.getByLabel("成果类型").selectOption("research_project");
	await expect(page.getByLabel("成果类型")).toHaveValue("open_class");
	await expect(page.locator("#title")).toHaveValue("尚未保存的公开课");
	page.once("dialog", (dialog) => dialog.dismiss());
	await page.getByRole("link", { name: "我的成果", exact: true }).click();
	await expect(page).toHaveURL(/\/achievements\/new$/);
	page.once("dialog", (dialog) => dialog.dismiss());
	await page.goBack();
	await expect(page).toHaveURL(/\/achievements\/new$/);
	page.once("dialog", (dialog) => dialog.dismiss());
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect(page.locator("#title")).toHaveValue("尚未保存的公开课");
	page.once("dialog", (dialog) => dialog.accept());
	await page.getByLabel("成果类型").selectOption("research_project");
	await expect(page.getByLabel("成果类型")).toHaveValue("research_project");
	await expect(page.locator("#title")).toHaveValue("");
	await expect(page.getByLabel("项目经费（元）")).toBeVisible();
	await page.goBack();
	await expect(page.getByLabel("成果类型")).toHaveValue("open_class");
	await expect(page.locator("#venue")).toBeVisible();
});

test("搜索图标提交查询，后退恢复搜索词，特殊符号按文字搜索", async ({
	page,
}) => {
	await login(page);
	const stamp = Date.now();
	const prefix = `交互检查-${stamp}`;
	const first = `${prefix}-%_!`;
	const second = `${prefix}-普通`;
	await createDraft(page, first);
	await createDraft(page, second);
	await page.getByRole("link", { name: "我的成果", exact: true }).click();
	await page.getByLabel("搜索成果名称").fill(prefix);
	await page.getByRole("button", { name: "搜索成果", exact: true }).click();
	await expect(
		page.getByRole("link", { name: second, exact: true }),
	).toBeVisible();
	await page.getByLabel("搜索成果名称").fill(first);
	await page.getByRole("button", { name: "搜索成果", exact: true }).click();
	await expect(
		page.getByRole("link", { name: second, exact: true }),
	).toHaveCount(0);
	await page.goBack();
	await expect(page.getByLabel("搜索成果名称")).toHaveValue(prefix);
	await expect(
		page.getByRole("link", { name: second, exact: true }),
	).toBeVisible();
	await page.getByLabel("搜索成果名称").fill(`${prefix}-不存在`);
	await page.getByRole("button", { name: "搜索成果", exact: true }).click();
	await expect(
		page.getByText("没有符合条件的成果。", { exact: true }),
	).toBeVisible();
});

test("恢复空值后切换类型无需放弃内容确认", async ({ page }) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	await page.locator("#subject_code").selectOption("math");
	await page.locator("#subject_code").selectOption("");
	await page.getByLabel("成果类型").selectOption("research_project");
	await expect(page.getByLabel("项目经费（元）")).toBeVisible();
});

test("缺少证明材料时定位错误且不创建草稿", async ({ page }) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	await page.locator("#title").fill("尚无证明材料");
	let creates = 0;
	page.on("request", (request) => {
		if (
			request.method() === "POST" &&
			new URL(request.url()).pathname === "/api/v1/achievements"
		)
			creates++;
	});
	await page.getByRole("button", { name: "提交审核", exact: true }).click();
	await expect(page.locator("#proofs-error")).toHaveText("请填写证明材料");
	await expect(
		page.getByRole("button", { name: "上传证明材料", exact: true }),
	).toBeFocused();
	expect(creates).toBe(0);
});

test("提交失败重试复用已保存成果，不增加无变化的修订", async ({ page }) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	const title = `提交重试-${Date.now()}`;
	await page.locator("#title").fill(title);
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	let attempts = 0;
	await page.route("**/api/v1/achievements/*/submit", async (route) => {
		attempts++;
		if (attempts === 1)
			await route.fulfill({
				status: 500,
				contentType: "application/json",
				body: JSON.stringify({ msg: "提交暂时失败" }),
			});
		else await route.continue();
	});
	await page.getByRole("button", { name: "提交审核", exact: true }).click();
	await expect(
		page.getByText("填报内容已保存。提交暂时失败", { exact: true }),
	).toBeVisible();
	await page.getByRole("button", { name: "提交审核", exact: true }).click();
	await expect(
		page.getByRole("heading", { name: title, exact: true }),
	).toBeVisible();
	await expect(
		page.locator(".record-meta").getByText("审核中", { exact: true }),
	).toBeVisible();
	await expect(page.locator(".revision-row")).toHaveCount(2);
	expect(attempts).toBe(2);
});

test("旧版本保存保留本地内容，确认重载后读取最新成果", async ({ page }) => {
	await login(page);
	const draft = await createDraft(page, `版本冲突-${Date.now()}`);
	await page.goto(`/achievements/${draft.record.id}/edit`);
	await page.locator("#title").fill("本地尚未保存的修改");
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	const auth = await session(page);
	const response = await page.request.patch(
		`/api/v1/achievements/${draft.record.id}`,
		{
			headers: {
				Origin: new URL(page.url()).origin,
				"X-CSRF-Token": auth.csrf_token,
			},
			data: {
				version: draft.record.version,
				data: { ...draft.data, title: "其他窗口已保存的修改" },
			},
		},
	);
	expect(response.ok()).toBe(true);
	await page.getByRole("button", { name: "提交审核", exact: true }).click();
	await expect(
		page.getByRole("button", { name: "重新加载成果", exact: true }),
	).toBeVisible();
	await expect(page.locator(".validation-summary")).not.toContainText(
		"填报内容已保存",
	);
	await expect(page.locator("#title")).toHaveValue("本地尚未保存的修改");
	page.once("dialog", (dialog) => dialog.accept());
	await page.getByRole("button", { name: "重新加载成果", exact: true }).click();
	await expect(page.locator("#title")).toHaveValue("其他窗口已保存的修改");
});

test("上传中的导航被阻止，上传完成后可以保存", async ({ page }) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	const title = `慢上传-${Date.now()}`;
	await page.locator("#title").fill(title);
	let release: () => void = () => {};
	const gate = new Promise<void>((resolve) => {
		release = resolve;
	});
	await page.route("**/api/v1/files/upload", async (route) => {
		await gate;
		await route.continue();
	});
	try {
		await page.getByLabel("选择证明材料").setInputFiles(evidence);
		await expect(page.getByText("材料上传中…", { exact: true })).toBeVisible();
		await page.getByRole("link", { name: "我的成果", exact: true }).click();
		await expect(page).toHaveURL(/\/achievements\/new$/);
		await expect(
			page.getByRole("button", { name: "保存草稿", exact: true }),
		).toBeDisabled();
	} finally {
		release();
	}
	await expect(page.getByText("evidence.png", { exact: true })).toBeVisible();
	await page.getByRole("button", { name: "保存草稿", exact: true }).click();
	await expect(
		page.getByRole("heading", { name: title, exact: true }),
	).toBeVisible();
});

test("上传时会话失效会回到登录页", async ({ page }) => {
	await login(page);
	await page.getByRole("link", { name: "成果填报", exact: true }).click();
	await expect(page.locator("#title")).toBeVisible();
	const auth = await session(page);
	const response = await page.request.post("/api/v1/auth/logout", {
		headers: {
			Origin: new URL(page.url()).origin,
			"X-CSRF-Token": auth.csrf_token,
		},
	});
	expect(response.ok()).toBe(true);
	await page.getByLabel("选择证明材料").setInputFiles(evidence);
	await expect(page).toHaveURL(/\/login$/);
});

test("手机混合上传保留成功材料，长文件名的错误不会撑宽页面", async ({
	page,
}) => {
	await page.setViewportSize({ width: 320, height: 740 });
	await login(page);
	await page.goto("/achievements/new");
	await page.getByLabel("选择证明材料").setInputFiles([
		{
			name: `${"failed-proof".repeat(18)}.png`,
			mimeType: "image/png",
			buffer: Buffer.from("invalid PNG"),
		},
		{
			name: "good-proof.png",
			mimeType: "image/png",
			buffer: readFileSync(evidence),
		},
	]);
	await expect(page.getByText("good-proof.png", { exact: true })).toBeVisible();
	await expect(page.locator(".inline-error")).toContainText("failed-proof");
	expect(
		await page.evaluate(
			() => document.documentElement.scrollWidth <= window.innerWidth,
		),
	).toBe(true);
});

test("320px 下导航可关闭、人员可排序，长标题无页面溢出", async ({ page }) => {
	await page.setViewportSize({ width: 320, height: 740 });
	await login(page);
	await page.getByRole("button", { name: "展开导航", exact: true }).click();
	const close = page.getByRole("button", { name: "关闭导航", exact: true });
	await expect(close).toBeFocused();
	const box = await close.boundingBox();
	expect(box?.y).toBeLessThan(200);
	await page.keyboard.press("Escape");
	await expect(
		page.getByRole("button", { name: "展开导航", exact: true }),
	).toBeFocused();
	await page.goto("/achievements/new?type=research_project");
	const title = `${"长标题验证".repeat(30)}-${Date.now()}`;
	await page.locator("#title").fill(title);
	await page.getByRole("button", { name: "添加相关人员", exact: true }).click();
	await page.locator("#name-0").fill("甲老师");
	await page.getByRole("button", { name: "添加相关人员", exact: true }).click();
	await page.locator("#name-1").fill("乙老师");
	await page
		.getByRole("button", { name: "上移第2位人员", exact: true })
		.click();
	await expect(page.locator("#name-0")).toHaveValue("乙老师");
	await page.getByRole("button", { name: "保存草稿", exact: true }).click();
	await expect(
		page.getByRole("heading", { name: title, exact: true }),
	).toBeVisible();
	expect(
		await page.evaluate(
			() => document.documentElement.scrollWidth <= window.innerWidth,
		),
	).toBe(true);
});
