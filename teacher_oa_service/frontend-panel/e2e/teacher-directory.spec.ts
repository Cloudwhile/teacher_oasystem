import { readFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "playwright/test";
import { fixtureLogin } from "./fixture-login";

const evidence = path.resolve(import.meta.dirname, "fixtures/evidence.png");

test("管理员与校级行政可查阅全校教师资料、成果记录与证明材料", async ({
	page,
}) => {
	await page.setViewportSize({ width: 1440, height: 900 });

	// 1. 普通教师填报并提交一条公开课成果（附证明材料）以及一条未提交草稿
	await page.goto("/login");
	await fixtureLogin(page, "teacher");
	const authResp = await page.request.get("/api/v1/auth/me");
	expect(authResp.ok()).toBe(true);
	const teacherAuth = (await authResp.json()).data;
	const origin = new URL(page.url()).origin;

	const uploaded = await page.request.post("/api/v1/files/upload", {
		headers: { Origin: origin, "X-CSRF-Token": teacherAuth.csrf_token },
		multipart: {
			file: {
				name: "dossier-proof.png",
				mimeType: "image/png",
				buffer: readFileSync(evidence),
			},
		},
	});
	expect(uploaded.ok()).toBe(true);
	const file = (await uploaded.json()).data;

	const stamp = Date.now();
	const submittedTitle = `档案展示公开课-${stamp}`;
	const draftTitle = `私人未提交草稿-${stamp}`;

	const createSubmitted = await page.request.post("/api/v1/achievements", {
		headers: { Origin: origin, "X-CSRF-Token": teacherAuth.csrf_token },
		data: {
			type_code: "open_class",
			schema_version: 1,
			title: submittedTitle,
			subject_code: "math",
			attributes: {},
			file_ids: [file.id],
		},
	});
	expect(createSubmitted.ok()).toBe(true);
	const created = (await createSubmitted.json()).data;
	const submitResp = await page.request.post(
		`/api/v1/achievements/${created.record.id}/submit`,
		{
			headers: { Origin: origin, "X-CSRF-Token": teacherAuth.csrf_token },
			data: { version: created.record.version },
		},
	);
	expect(submitResp.ok()).toBe(true);

	const createDraft = await page.request.post("/api/v1/achievements", {
		headers: { Origin: origin, "X-CSRF-Token": teacherAuth.csrf_token },
		data: {
			type_code: "open_class",
			schema_version: 1,
			title: draftTitle,
			subject_code: "math",
			attributes: {},
			file_ids: [],
		},
	});
	expect(createDraft.ok()).toBe(true);

	// 普通教师直接访问 /teachers 应被拒绝
	await page.goto("/teachers");
	await expect(
		page.getByText("仅系统管理员与校级行政可查阅全校教师资料"),
	).toBeVisible();

	// 2. 管理员登录，创建一位「校级行政」角色账号，并验证管理员自身的全校教师资料视野
	await fixtureLogin(page, "admin");
	await page.goto("/admin/accounts");
	await page.getByRole("button", { name: "新增账号" }).click();
	const leaderUsername = `leader_${stamp}`;
	await page.locator("#username").fill(leaderUsername);
	await page.locator("#name").fill("赵校长");
	await page.locator("#department").fill("校长室");
	await page.locator("#employee_no").fill(`EXEC-${stamp}`);
	await page.locator("#subject_code").selectOption("education_management");
	await page.locator("#role").selectOption("school_leader");
	await page.locator("#initial-password").fill("Executive-pass-123");
	await page.getByRole("button", { name: "保存账号" }).click();
	await expect(page.getByText("赵校长", { exact: true })).toBeVisible();

	await page
		.getByRole("navigation", { name: "主导航" })
		.getByRole("link", { name: "教师资料", exact: true })
		.click();
	await expect(
		page.getByRole("heading", { level: 1, name: "教师资料", exact: true }),
	).toBeVisible();
	await page.getByLabel("筛选所属部门").selectOption("91001");
	await expect(
		page.getByRole("link", { name: "测试教师", exact: true }),
	).toBeVisible();
	await page.getByRole("link", { name: "测试教师", exact: true }).click();
	await expect(
		page.getByRole("heading", { level: 1, name: "测试教师", exact: true }),
	).toBeVisible();
	await expect(
		page.getByRole("link", { name: submittedTitle, exact: true }),
	).toBeVisible();
	await expect(page.getByText(draftTitle)).toHaveCount(0);
	await expect(
		page.getByText("dossier-proof.png", { exact: true }),
	).toBeVisible();

	// 3. 退出并以新建的「校级行政」账号登录（首次登录修改密码后查阅全校教师资料与成果筛选）
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect(page).toHaveURL(/\/login$/);
	await page.locator("#username").fill(leaderUsername);
	await page.locator("#password").fill("Executive-pass-123");
	await page.getByRole("button", { name: "登录", exact: true }).click();
	await expect(page).toHaveURL(/\/account/);
	await page.locator("#current-password").fill("Executive-pass-123");
	await page.locator("#new-password").fill("Executive-pass-456");
	await page.locator("#confirm-password").fill("Executive-pass-456");
	await page.getByRole("button", { name: "更新密码" }).click();
	await expect(page).not.toHaveURL(/\/account/);

	await page
		.getByRole("navigation", { name: "主导航" })
		.getByRole("link", { name: "教师资料", exact: true })
		.click();
	await expect(page).toHaveURL(/\/teachers$/);
	await expect(
		page.getByRole("link", { name: "测试教师", exact: true }),
	).toBeVisible();
	await page.getByRole("link", { name: "测试教师", exact: true }).click();
	await expect(
		page.getByRole("link", { name: submittedTitle, exact: true }),
	).toBeVisible();
	const previewHref = await page
		.getByRole("link", { name: "预览dossier-proof.png" })
		.getAttribute("href");
	const previewResp = await page.request.get(previewHref ?? "invalid");
	expect(previewResp.status()).toBe(200);

	// 点击「在成果查询中筛选」联动成果查询页按该教师过滤
	await page.getByRole("link", { name: "在成果查询中筛选" }).click();
	await expect(page).toHaveURL(/\/achievements\?scope=all&teacher_id=91001$/);
	await expect(page.getByLabel("筛选教师")).toHaveValue("91001");
	await expect(
		page.getByRole("link", { name: submittedTitle, exact: true }),
	).toBeVisible();
});
