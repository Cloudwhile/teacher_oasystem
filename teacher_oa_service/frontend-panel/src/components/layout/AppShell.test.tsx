// @vitest-environment jsdom
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import { AppShell } from "./AppShell";

const auth = vi.hoisted(() => ({
	session: {
		csrf_token: "csrf",
		user: {
			id: 1,
			display_name: "王老师",
			teacher: { department_name: "数学教研组" },
			grants: [] as { permission: string; scope: string }[],
		},
	},
	logout: vi.fn(),
}));
vi.mock("@/features/auth/AuthProvider", () => ({ useAuth: () => auth }));

function DirtyPage() {
	useUnsavedChanges(true, false);
	return <p>未保存的填报</p>;
}
function show(path: string, permissions: string[] = [], state?: unknown) {
	auth.session.user.grants = permissions.map((permission) => ({
		permission,
		scope: "ALL",
	}));
	const page = (name: string) => <p>{name}</p>;
	const { pathname, search } = new URL(path, "http://oa.test");
	const router = createMemoryRouter(
		[
			{
				element: <AppShell />,
				children: [
					{ path: "/dashboard", element: page("工作台页") },
					{ path: "/achievements", element: page("成果列表页") },
					{ path: "/achievements/new", element: <DirtyPage /> },
					{ path: "/achievements/:id", element: page("成果详情页") },
					{ path: "/reviews", element: page("审核页") },
					{ path: "/files", element: page("文件页") },
					{ path: "/teachers/me", element: page("档案页") },
					{ path: "/account", element: page("账号页") },
					{ path: "/admin/*", element: page("管理页") },
				],
			},
		],
		{ initialEntries: [{ pathname, search, state }] },
	);
	render(<RouterProvider router={router} />);
	return router;
}
const nav = () => screen.getByRole("navigation", { name: "主导航" });
const group = () => within(nav()).getByRole("button", { name: "成果管理" });
const current = () =>
	within(nav())
		.queryAllByRole("link")
		.filter((link) => link.hasAttribute("aria-current"))
		.map((link) => link.textContent);
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	window.localStorage.clear();
});

it.each([
	[[], ["工作台", "成果管理", "文件资料"], false],
	[["reviews:write"], ["工作台", "成果管理", "审核中心", "文件资料"], false],
	[
		["reviews:write", "admin:manage"],
		["工作台", "成果管理", "审核中心", "文件资料"],
		true,
	],
])("按权限 %j 显示导航", (permissions, workspace, admin) => {
	show("/dashboard", permissions);
	const lists = within(nav()).getAllByRole("list");
	const top = within(lists[0])
		.getAllByRole("listitem")
		.filter((item) => item.parentElement === lists[0])
		.map((item) => item.firstElementChild?.textContent);
	expect(top).toEqual(workspace);
	expect(within(nav()).queryByRole("list", { name: "系统管理" }) !== null).toBe(
		admin,
	);
	if (admin)
		expect(
			within(within(nav()).getByRole("list", { name: "系统管理" }))
				.getAllByRole("link")
				.map((link) => link.textContent),
		).toEqual(["成员与账号", "登录设置", "登录页与公告"]);
	for (const missing of ["统计报表", "成果类型", "学年学期", "个人档案"])
		expect(within(nav()).queryByText(missing)).toBeNull();
});

it("展开收起成果管理不改变地址，二级入口指向成果范围", async () => {
	const router = show("/dashboard");
	expect(group().getAttribute("aria-expanded")).toBe("true");
	const sublist = document.getElementById(
		group().getAttribute("aria-controls") ?? "",
	);
	expect(sublist).not.toBeNull();
	fireEvent.click(group());
	expect(group().getAttribute("aria-expanded")).toBe("false");
	expect(sublist?.hidden).toBe(true);
	expect(router.state.location.pathname).toBe("/dashboard");
	expect(within(nav()).queryByRole("link", { name: "我的成果" })).toBeNull();
	fireEvent.click(group());
	expect(
		within(nav())
			.getAllByRole("link")
			.filter((link) => sublist?.contains(link))
			.map((link) => [link.textContent, link.getAttribute("href")]),
	).toEqual([
		["我的成果", "/achievements?scope=mine"],
		["成果查询", "/achievements?scope=all"],
		["成果填报", "/achievements/new"],
	]);
});

it("成果页默认展开且只高亮一个入口，可手动收起，再进入时自动展开", async () => {
	const router = show("/achievements?scope=all");
	expect(group().getAttribute("aria-expanded")).toBe("true");
	expect(current()).toEqual(["成果查询"]);
	expect(
		within(nav())
			.getByRole("link", { name: "成果查询" })
			.getAttribute("aria-current"),
	).toBe("page");
	fireEvent.click(group());
	expect(group().getAttribute("aria-expanded")).toBe("false");
	expect(group().className).toContain("contains-current");
	await act(() => router.navigate("/files"));
	expect(group().getAttribute("aria-expanded")).toBe("false");
	expect(current()).toEqual(["文件资料"]);
	await act(() => router.navigate("/achievements?scope=mine"));
	expect(group().getAttribute("aria-expanded")).toBe("true");
	expect(current()).toEqual(["我的成果"]);
});

it("收起偏好在非成果页保留，但不隐藏当前成果页", () => {
	window.localStorage.setItem("teacher-oa:nav-group:achievements", "closed");
	show("/dashboard");
	expect(group().getAttribute("aria-expanded")).toBe("false");
	cleanup();
	show("/achievements/new");
	expect(group().getAttribute("aria-expanded")).toBe("true");
});

it("审核来源的详情只高亮审核中心，来源缺失时不猜测列表", () => {
	show("/achievements/9", ["reviews:write"], {
		from: "/reviews?status=pending",
	});
	expect(current()).toEqual(["审核中心"]);
	expect(
		within(nav())
			.getByRole("link", { name: "审核中心" })
			.getAttribute("aria-current"),
	).toBe("location");
	expect(document.querySelector(".breadcrumb")?.textContent).toBe(
		"审核中心/成果详情",
	);
	cleanup();
	show("/achievements/9");
	expect(current()).toEqual([]);
	expect(group().getAttribute("aria-expanded")).toBe("true");
	expect(document.querySelector(".breadcrumb")?.textContent).toBe(
		"成果管理/成果详情",
	);
});

it("账户菜单提供档案与安全入口，被拦截的导航保持页面与菜单", async () => {
	const router = show("/achievements/new");
	const trigger = screen.getByRole("button", {
		name: "账户菜单：王老师，数学教研组",
	});
	expect(screen.queryByRole("link", { name: "个人档案" })).toBeNull();
	fireEvent.click(trigger);
	expect(trigger.getAttribute("aria-expanded")).toBe("true");
	const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
	fireEvent.click(screen.getByRole("link", { name: "个人档案" }));
	await waitFor(() => expect(confirm).toHaveBeenCalledOnce());
	expect(router.state.location.pathname).toBe("/achievements/new");
	expect(screen.getByText("未保存的填报")).toBeTruthy();
	expect(trigger.getAttribute("aria-expanded")).toBe("true");
	confirm.mockReturnValue(true);
	fireEvent.click(screen.getByRole("link", { name: "账号安全" }));
	await screen.findByText("账号页");
	expect(trigger.getAttribute("aria-expanded")).toBe("false");
	fireEvent.click(trigger);
	expect(
		screen.getByRole("link", { name: "账号安全" }).getAttribute("aria-current"),
	).toBe("page");
	fireEvent.keyDown(screen.getByRole("link", { name: "账号安全" }), {
		key: "Escape",
	});
	expect(trigger.getAttribute("aria-expanded")).toBe("false");
	expect(document.activeElement).toBe(trigger);
});

it("展开收起成果管理不触发离开确认，退出仍走离开保护", () => {
	show("/achievements/new");
	const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
	fireEvent.click(group());
	fireEvent.click(group());
	expect(confirm).not.toHaveBeenCalled();
	fireEvent.click(screen.getByRole("button", { name: "退出登录" }));
	expect(confirm).toHaveBeenCalledOnce();
	expect(auth.logout).not.toHaveBeenCalled();
});

it("退出失败时显示错误", async () => {
	auth.logout.mockRejectedValueOnce(new Error("offline"));
	show("/dashboard");
	fireEvent.click(screen.getByRole("button", { name: "退出登录" }));
	expect((await screen.findByRole("alert")).textContent).toBe(
		"退出未完成，请重试。",
	);
});
