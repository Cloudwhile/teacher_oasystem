import { describe, expect, it } from "vitest";
import { resolveNavigation } from "./navigationState";

describe("resolveNavigation", () => {
	it.each([
		["/achievements", "", "achievements-mine", "我的成果"],
		["/achievements", "?scope=mine", "achievements-mine", "我的成果"],
		["/achievements", "?scope=all&page=2", "achievements-all", "成果查询"],
		["/achievements/", "?scope=all", "achievements-all", "成果查询"],
		[
			"/achievements/new",
			"?type=research_project",
			"achievements-new",
			"成果填报",
		],
	])("marks %s%s as the %s page", (pathname, search, key, title) => {
		expect(resolveNavigation(pathname, search, undefined)).toEqual({
			activeKey: key,
			activeMatch: "page",
			activeGroup: "achievements",
			title,
			trail: ["成果管理", title],
		});
	});

	it.each([
		["/achievements?scope=mine", "achievements-mine", "我的成果"],
		["/achievements?scope=all&search=公开课", "achievements-all", "成果查询"],
		["/achievements", "achievements-mine", "我的成果"],
	])("keeps list %s as the source of a detail page", (from, key, label) => {
		expect(resolveNavigation("/achievements/12", "", from)).toEqual({
			activeKey: key,
			activeMatch: "location",
			activeGroup: "achievements",
			title: "成果详情",
			trail: ["成果管理", label, "成果详情"],
		});
	});

	it("attributes detail and edit pages opened from reviews to the review center", () => {
		for (const [path, title] of [
			["/achievements/12", "成果详情"],
			["/achievements/12/edit", "编辑成果"],
		])
			expect(
				resolveNavigation(path, "", "/reviews?status=pending&page=3"),
			).toEqual({
				activeKey: "reviews",
				activeMatch: "location",
				activeGroup: null,
				title,
				trail: ["审核中心", title],
			});
	});

	it.each([
		undefined,
		null,
		42,
		"/dashboard",
		"https://example.com/reviews",
		"//example.com/reviews",
		"/reviewsx",
	])("does not invent a source for a deep link (from=%s)", (from) => {
		expect(resolveNavigation("/achievements/12/edit", "", from)).toEqual({
			activeKey: null,
			activeMatch: "location",
			activeGroup: "achievements",
			title: "编辑成果",
			trail: ["成果管理", "编辑成果"],
		});
	});

	it.each([
		["/dashboard", "dashboard", ["工作台"]],
		["/reviews", "reviews", ["审核中心"]],
		["/files", "files", ["文件资料"]],
		["/teachers/me", "profile", ["个人账户", "个人档案"]],
		["/account", "account", ["个人账户", "账号安全"]],
		["/admin/accounts", "admin-accounts", ["系统管理", "成员与账号"]],
		["/admin/login", "admin-login", ["系统管理", "登录设置"]],
		["/admin/login-page", "admin-login-page", ["系统管理", "登录页与公告"]],
	])("maps %s to a single navigation item", (pathname, key, trail) => {
		const state = resolveNavigation(pathname, "?any=1", "/reviews");
		expect(state.activeKey).toBe(key);
		expect(state.activeMatch).toBe("page");
		expect(state.activeGroup).toBeNull();
		expect(state.trail).toEqual(trail);
	});

	it("leaves unknown pages without an active item", () => {
		expect(resolveNavigation("/missing", "", undefined)).toMatchObject({
			activeKey: null,
			activeGroup: null,
			title: "页面未找到",
		});
	});
});
