import { expect, type Page, test } from "playwright/test";
import { mockApi, type Role } from "./mock-api";

test.use({ serviceWorkers: "block" });

async function open(page: Page, role: Role, path = "/dashboard") {
	await mockApi(page, role);
	await page.goto(path);
}
const nav = (page: Page) => page.getByRole("navigation", { name: "主导航" });
const group = (page: Page) =>
	nav(page).getByRole("button", { name: "成果管理", exact: true });
const current = (page: Page) => nav(page).locator("[aria-current]");
const crumbs = (page: Page) => page.locator(".breadcrumb");
const heading = (page: Page, name: string) =>
	page.getByRole("heading", { level: 1, name, exact: true });
const noHorizontalOverflow = (page: Page) =>
	page.evaluate(
		() => document.documentElement.scrollWidth <= window.innerWidth,
	);

test.describe("导航结构与权限", () => {
	for (const [role, workspace, admin] of [
		["teacher", ["工作台", "成果管理", "文件资料"], false],
		["reviewer", ["工作台", "成果管理", "审核中心", "文件资料"], false],
		["admin", ["工作台", "成果管理", "审核中心", "文件资料"], true],
	] as const)
		test(`${role} 的导航结构`, async ({ page }) => {
			await open(page, role);
			await expect(heading(page, "工作台")).toBeVisible();
			await expect(
				nav(page).getByRole("link", { name: "新建成果", exact: true }),
			).toBeVisible();
			await expect(
				nav(page)
					.getByRole("list", { name: "工作空间" })
					.locator(":scope > li > :first-child"),
			).toHaveText([...workspace]);
			await expect(
				nav(page).locator(".nav-sublist").getByRole("link"),
			).toHaveText(["我的成果", "成果查询", "成果填报"]);
			const adminList = nav(page).getByRole("list", { name: "系统管理" });
			if (admin)
				await expect(adminList.getByRole("link")).toHaveText([
					"成员与账号",
					"登录设置",
					"登录页与公告",
				]);
			else await expect(adminList).toHaveCount(0);
			for (const hidden of ["统计报表", "成果类型", "学年学期"])
				await expect(nav(page).getByText(hidden)).toHaveCount(0);
			await expect(
				page.getByRole("region", { name: "个人账户" }).getByRole("button", {
					name: "账户菜单：王老师，数学教研组",
				}),
			).toBeVisible();
		});
});

test("成果管理折叠不改地址，二级导航无整页刷新且只高亮一项", async ({
	page,
}) => {
	await open(page, "teacher");
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
	await page.evaluate(() => {
		(window as unknown as { spaMarker: boolean }).spaMarker = true;
	});
	await group(page).click();
	await expect(group(page)).toHaveAttribute("aria-expanded", "false");
	await expect(
		nav(page).getByRole("link", { name: "我的成果", exact: true }),
	).toBeHidden();
	await expect(page).toHaveURL(/\/dashboard$/);
	await group(page).click();
	for (const [label, url, title] of [
		["我的成果", /\/achievements\?scope=mine$/, "我的成果"],
		["成果查询", /\/achievements\?scope=all$/, "成果查询"],
		["成果填报", /\/achievements\/new$/, "新建成果"],
	] as const) {
		await nav(page).getByRole("link", { name: label, exact: true }).click();
		await expect(page).toHaveURL(url);
		await expect(heading(page, title)).toBeVisible();
		await expect(current(page)).toHaveText([label]);
		await expect(current(page)).toHaveAttribute("aria-current", "page");
	}
	expect(
		await page.evaluate(
			() => (window as unknown as { spaMarker?: boolean }).spaMarker,
		),
	).toBe(true);
	await group(page).click();
	await expect(group(page)).toHaveAttribute("aria-expanded", "false");
	await expect(group(page)).toHaveClass(/contains-current/);
	await nav(page).getByRole("link", { name: "文件资料", exact: true }).click();
	await expect(group(page)).toHaveAttribute("aria-expanded", "false");
	await page.goBack();
	await expect(page).toHaveURL(/\/achievements\/new$/);
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
	await expect(current(page)).toHaveText(["成果填报"]);
});

test("直接打开与刷新时按地址高亮成果范围", async ({ page }) => {
	await open(page, "teacher", "/achievements?scope=all&page=2");
	await expect(heading(page, "成果查询")).toBeVisible();
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
	await expect(current(page)).toHaveText(["成果查询"]);
	await expect(crumbs(page)).toHaveText("成果管理/成果查询");
	await page.reload();
	await expect(current(page)).toHaveText(["成果查询"]);
	await page.goto("/achievements?scope=mine");
	await expect(current(page)).toHaveText(["我的成果"]);
	await page.goto("/achievements");
	await expect(current(page)).toHaveText(["我的成果"]);
});

test("详情与编辑保留列表来源，刷新、前进后退和返回列表保持查询", async ({
	page,
}) => {
	await open(page, "teacher", "/achievements?scope=all&search=教研");
	await page.getByRole("button", { name: "下一页" }).click();
	await expect(page).toHaveURL(/scope=all&search=%E6%95%99%E7%A0%94&page=2$/);
	await page
		.getByRole("link", { name: "全部成果-教研-第2页", exact: true })
		.click();
	await expect(page).toHaveURL(/\/achievements\/12$/);
	await expect(current(page)).toHaveText(["成果查询"]);
	await expect(current(page)).toHaveAttribute("aria-current", "location");
	await expect(crumbs(page)).toHaveText("成果管理/成果查询/成果详情");
	await page.reload();
	await expect(current(page)).toHaveText(["成果查询"]);
	await page.getByRole("link", { name: "编辑成果" }).click();
	await expect(page).toHaveURL(/\/achievements\/12\/edit$/);
	await expect(crumbs(page)).toHaveText("成果管理/成果查询/编辑成果");
	await expect(current(page)).toHaveText(["成果查询"]);
	await page.goBack();
	await expect(crumbs(page)).toHaveText("成果管理/成果查询/成果详情");
	await page.goForward();
	await expect(crumbs(page)).toHaveText("成果管理/成果查询/编辑成果");
	await page.goBack();
	await page.getByRole("link", { name: "返回成果列表" }).click();
	await expect(page).toHaveURL(/scope=all&search=%E6%95%99%E7%A0%94&page=2$/);
	await expect(page.getByLabel("搜索成果名称")).toHaveValue("教研");
	await expect(
		page.getByRole("link", { name: "全部成果-教研-第2页", exact: true }),
	).toBeVisible();
	await expect(current(page)).toHaveText(["成果查询"]);
	await page.getByLabel("成果范围").selectOption("mine");
	await expect(current(page)).toHaveText(["我的成果"]);
	await page
		.getByRole("link", { name: "我的成果-教研-第1页", exact: true })
		.click();
	await expect(crumbs(page)).toHaveText("成果管理/我的成果/成果详情");
	await expect(current(page)).toHaveText(["我的成果"]);
});

test("审核列表进入详情保留审核来源，直接打开详情不猜测来源", async ({
	page,
}) => {
	await open(page, "reviewer", "/reviews?status=pending");
	await page.getByRole("link", { name: "待审核的公开课", exact: true }).click();
	await expect(page).toHaveURL(/\/achievements\/31$/);
	await expect(crumbs(page)).toHaveText("审核中心/成果详情");
	await expect(current(page)).toHaveText(["审核中心"]);
	await page.reload();
	await expect(current(page)).toHaveText(["审核中心"]);
	await page.getByRole("link", { name: "返回审核列表" }).click();
	await expect(page).toHaveURL(/\/reviews\?status=pending$/);
	await expect(current(page)).toHaveText(["审核中心"]);
	await page.goto("/achievements/31");
	await expect(heading(page, "成果31")).toBeVisible();
	await expect(crumbs(page)).toHaveText("成果管理/成果详情");
	await expect(current(page)).toHaveCount(0);
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
});

test("未保存填报：折叠不确认，取消离开保留内容和抽屉，确认后才关闭", async ({
	page,
}) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await open(page, "teacher", "/achievements/new");
	await page.locator("#title").fill("尚未保存的公开课");
	let dialogs = 0;
	let accept = false;
	page.on("dialog", (dialog) => {
		dialogs++;
		void (accept ? dialog.accept() : dialog.dismiss());
	});
	await page.getByRole("button", { name: "展开导航" }).click();
	const drawer = page.getByRole("dialog", { name: "导航菜单" });
	await expect(drawer).toBeVisible();
	await group(page).click();
	await group(page).press("Enter");
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
	expect(dialogs).toBe(0);
	await nav(page).getByRole("link", { name: "我的成果", exact: true }).click();
	await expect.poll(() => dialogs).toBe(1);
	await expect(page).toHaveURL(/\/achievements\/new$/);
	await expect(drawer).toBeVisible();
	await page.getByRole("button", { name: /^账户菜单/ }).click();
	await page.getByRole("link", { name: "个人档案", exact: true }).click();
	await expect.poll(() => dialogs).toBe(2);
	await expect(drawer).toBeVisible();
	await expect(page.getByRole("link", { name: "个人档案" })).toBeVisible();
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect.poll(() => dialogs).toBe(3);
	await expect(page).toHaveURL(/\/achievements\/new$/);
	await page.keyboard.press("Escape");
	await page.keyboard.press("Escape");
	await expect(drawer).toBeHidden();
	await expect(page.locator("#title")).toHaveValue("尚未保存的公开课");
	accept = true;
	await page.getByRole("button", { name: "展开导航" }).click();
	await nav(page).getByRole("link", { name: "我的成果", exact: true }).click();
	await expect(page).toHaveURL(/\/achievements\?scope=mine$/);
	await expect(drawer).toBeHidden();
	await expect(page.locator("#main-content")).toBeFocused();
});

test("移动抽屉：焦点管理、背景锁定、遮罩、Escape 与关闭按钮", async ({
	page,
}) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await open(page, "teacher");
	const toggle = page.getByRole("button", { name: "展开导航" });
	const drawer = page.getByRole("dialog", { name: "导航菜单" });
	const closeButton = page.getByRole("button", { name: "关闭导航" });
	const focusIn = (selector: string) =>
		page.evaluate(
			(selector) => Boolean(document.activeElement?.closest(selector)),
			selector,
		);
	await expect(nav(page)).toBeHidden();
	for (let i = 0; i < 8; i++) {
		await page.keyboard.press("Tab");
		expect(await focusIn("#main-navigation")).toBe(false);
	}
	await toggle.click();
	await expect(drawer).toBeVisible();
	await expect(toggle).toHaveAttribute("aria-expanded", "true");
	await expect(toggle).toHaveAttribute("aria-controls", "main-navigation");
	await expect(closeButton).toBeFocused();
	await expect(page.locator(".workspace")).toHaveJSProperty("inert", true);
	expect(
		await page.evaluate(
			() => getComputedStyle(document.documentElement).overflowY,
		),
	).toBe("hidden");
	for (const key of ["Tab", "Shift+Tab"])
		for (let i = 0; i < 14; i++) {
			await page.keyboard.press(key);
			expect(await focusIn(".workspace")).toBe(false);
		}
	await page.keyboard.press("Escape");
	await expect(drawer).toBeHidden();
	await expect(toggle).toBeFocused();
	await expect(toggle).toHaveAttribute("aria-expanded", "false");
	await toggle.click();
	await page.mouse.click(380, 420);
	await expect(drawer).toBeHidden();
	await expect(toggle).toBeFocused();
	await toggle.click();
	await closeButton.click();
	await expect(drawer).toBeHidden();
	await expect(toggle).toBeFocused();
	await toggle.click();
	await nav(page).getByRole("link", { name: "文件资料", exact: true }).click();
	await expect(page).toHaveURL(/\/files$/);
	await expect(drawer).toBeHidden();
	await expect(page.locator("#main-content")).toBeFocused();
	await toggle.click();
	await page.getByRole("link", { name: /TeacherOA/ }).click();
	await expect(page).toHaveURL(/\/dashboard$/);
	await expect(drawer).toBeHidden();
	await expect(page.locator("#main-content")).toBeFocused();
	await toggle.click();
	await page.goBack();
	await expect(page).toHaveURL(/\/files$/);
	await expect(drawer).toBeHidden();
	await page.goForward();
	await expect(drawer).toBeHidden();
	await expect(page.locator(".workspace")).toHaveJSProperty("inert", false);
});

test("减少动画模式下抽屉无过渡且功能正常", async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await page.emulateMedia({ reducedMotion: "reduce" });
	await open(page, "teacher");
	const sidebar = page.locator("#main-navigation");
	expect(
		await sidebar.evaluate((el) => getComputedStyle(el).transitionDuration),
	).toMatch(/^0s(, 0s)*$/);
	await page.getByRole("button", { name: "展开导航" }).click();
	await expect(page.getByRole("dialog", { name: "导航菜单" })).toBeVisible();
	expect(await sidebar.evaluate((el) => getComputedStyle(el).transform)).toBe(
		"none",
	);
	await page.keyboard.press("Escape");
	await expect(page.getByRole("dialog", { name: "导航菜单" })).toBeHidden();
	await page.emulateMedia({ reducedMotion: "no-preference" });
	expect(
		await sidebar.evaluate((el) => getComputedStyle(el).transitionDuration),
	).toContain("0.2s");
});

test.describe("响应式布局", () => {
	test.use({ reducedMotion: "reduce" });
	for (const [width, height] of [
		[320, 740],
		[390, 844],
		[768, 1024],
		[1024, 768],
		[1366, 768],
	] as const)
		test(`${width}×${height} 无横向溢出，账号区可见，主内容不被推移`, async ({
			page,
		}) => {
			await page.setViewportSize({ width, height });
			await open(page, "admin", "/achievements/new");
			await expect(heading(page, "新建成果")).toBeVisible();
			expect(await noHorizontalOverflow(page)).toBe(true);
			const account = page.getByRole("button", { name: /^账户菜单/ });
			const content = page.locator("#main-content");
			const before = await content.boundingBox();
			if (width > 720) {
				await expect(
					page.getByRole("button", { name: "展开导航" }),
				).toBeHidden();
				const sidebar = await page.locator("#main-navigation").boundingBox();
				expect(sidebar?.width).toBe(244);
				expect(before?.x).toBeGreaterThanOrEqual(244);
				await expect(account).toBeInViewport({ ratio: 1 });
				await page.mouse.wheel(0, 2000);
				await expect(account).toBeInViewport({ ratio: 1 });
				await expect(page.locator(".brand")).toBeInViewport({ ratio: 1 });
				return;
			}
			await expect(account).toBeHidden();
			await page.getByRole("button", { name: "展开导航" }).click();
			const drawer = await page.locator("#main-navigation").boundingBox();
			expect(drawer).toMatchObject({ x: 0, y: 0, height });
			expect(drawer?.width).toBeCloseTo(Math.min(280, width * 0.85), 0);
			await expect(account).toBeInViewport({ ratio: 1 });
			expect(await content.boundingBox()).toEqual(before);
			expect(await noHorizontalOverflow(page)).toBe(true);
		});

	test("长菜单仅导航区滚动，品牌区和账号区固定", async ({ page }) => {
		await page.setViewportSize({ width: 1024, height: 520 });
		await open(page, "admin");
		const list = page.locator(".navigation");
		const brand = page.locator(".brand");
		const account = page.getByRole("button", { name: /^账户菜单/ });
		const scrollable = await list.evaluate(
			(el) => el.scrollHeight > el.clientHeight,
		);
		expect(scrollable).toBe(true);
		const brandBefore = await brand.boundingBox();
		await list.hover();
		await page.mouse.wheel(0, 600);
		await expect
			.poll(() => list.evaluate((el) => el.scrollTop))
			.toBeGreaterThan(0);
		expect(await brand.boundingBox()).toEqual(brandBefore);
		await expect(account).toBeInViewport({ ratio: 1 });
		await expect(
			nav(page).getByRole("link", { name: "登录页与公告" }),
		).toBeInViewport();
		expect(await page.evaluate(() => window.scrollY)).toBe(0);
	});
});

test("键盘：Tab 顺序、Enter/Space 折叠、隐藏项不可聚焦", async ({ page }) => {
	await page.setViewportSize({ width: 1366, height: 900 });
	await open(page, "teacher");
	await expect(heading(page, "工作台")).toBeVisible();
	await page.locator("body").focus();
	const focused = () =>
		page.evaluate(
			() =>
				document.activeElement?.getAttribute("aria-label") ??
				document.activeElement?.textContent?.trim(),
		);
	const order = [];
	for (let i = 0; i < 9; i++) {
		await page.keyboard.press("Tab");
		order.push(await focused());
	}
	expect(order).toEqual([
		"TeacherOA教师综合 OA",
		"新建成果",
		"工作台",
		"成果管理",
		"我的成果",
		"成果查询",
		"成果填报",
		"文件资料",
		"账户菜单：王老师，数学教研组",
	]);
	await group(page).focus();
	await page.keyboard.press("Enter");
	await expect(group(page)).toHaveAttribute("aria-expanded", "false");
	await page.keyboard.press("Tab");
	expect(await focused()).toBe("文件资料");
	await page.keyboard.press("Shift+Tab");
	await expect(group(page)).toBeFocused();
	await page.keyboard.press("Space");
	await expect(group(page)).toHaveAttribute("aria-expanded", "true");
	await page.keyboard.press("Tab");
	expect(await focused()).toBe("我的成果");
	await page.keyboard.press("Enter");
	await expect(page).toHaveURL(/scope=mine$/);
	await expect(
		nav(page).getByRole("link", { name: "我的成果", exact: true }),
	).toHaveAttribute("aria-current", "page");
	await expect(page.getByRole("button", { name: "关闭导航" })).toBeHidden();
});

test("账户菜单：长名称省略可读全称，移动抽屉内不超出屏幕", async ({ page }) => {
	await page.setViewportSize({ width: 320, height: 740 });
	const department = "高中部数学与信息技术综合教研组（含竞赛辅导）";
	await mockApi(page, "admin", { departmentName: department });
	await page.goto("/dashboard");
	await page.getByRole("button", { name: "展开导航" }).click();
	const trigger = page.getByRole("button", { name: /^账户菜单/ });
	await expect(trigger).toHaveAccessibleName(`账户菜单：王老师，${department}`);
	const small = trigger.locator("small");
	await expect(small).toHaveAttribute("title", department);
	expect(await small.evaluate((el) => el.scrollWidth > el.clientWidth)).toBe(
		true,
	);
	await trigger.click();
	await expect(trigger).toHaveAttribute("aria-expanded", "true");
	expect(await small.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(
		true,
	);
	await expect(small).toBeInViewport({ ratio: 1 });
	for (const name of ["个人档案", "账号安全"])
		await expect(page.getByRole("link", { name, exact: true })).toBeInViewport({
			ratio: 1,
		});
	await expect(page.getByRole("button", { name: "退出登录" })).toBeInViewport({
		ratio: 1,
	});
	expect(await noHorizontalOverflow(page)).toBe(true);
	await page.getByRole("link", { name: "个人档案", exact: true }).click();
	await expect(page).toHaveURL(/\/teachers\/me$/);
	await expect(page.getByRole("dialog", { name: "导航菜单" })).toBeHidden();
	await expect(crumbs(page)).toHaveText("个人账户/个人档案");
	await expect(page.locator("#main-content")).toBeFocused();
});

test("退出失败提示错误，重试后回到登录页", async ({ page }) => {
	const options = { logoutFails: true };
	await mockApi(page, "teacher", options);
	await page.goto("/dashboard");
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect(page.getByRole("alert")).toHaveText("退出未完成，请重试。");
	await expect(page).toHaveURL(/\/dashboard$/);
	options.logoutFails = false;
	await page.getByRole("button", { name: "退出登录" }).click();
	await expect(page).toHaveURL(/\/login$/);
});
