// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { achievementApi } from "@/features/achievements/api";
import { rememberListPath } from "@/services/returnPath";
import { useUiPreferencesStore } from "@/stores/uiPreferencesStore";
import { DashboardPage } from "./DashboardPage";

const auth = vi.hoisted(() => ({
	session: {
		csrf_token: "csrf",
		user: {
			id: 1,
			display_name: "李老师",
			roles: ["teacher"],
			teacher: { id: 1, department_name: "语文组" },
			grants: [{ permission: "achievements:write", scope: "SELF" }],
		},
	},
	refresh: vi.fn(),
}));
vi.mock("@/features/auth/AuthProvider", () => ({ useAuth: () => auth }));

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	window.localStorage.clear();
	window.sessionStorage.clear();
	useUiPreferencesStore.getState().syncFromStorage();
});

it("展示待补正与草稿待办行，新建成果携带上次列表筛选地址", async () => {
	rememberListPath("/achievements", "?scope=all&status=approved&page=2");
	vi.spyOn(achievementApi, "dashboard").mockResolvedValue({
		counts: {
			draft: 2,
			pending: 1,
			returned: 1,
			approved: 3,
			archived: 0,
		},
		review_pending: 0,
		recent: [
			{
				id: 9,
				teacher_id: 1,
				teacher_name: "李老师",
				title: "退回的示范课",
				type_code: "open_class",
				type_name: "公开课",
				status: "returned",
				version: 1,
				created_at: "2026-11-10T00:00:00Z",
				updated_at: "2026-11-10T00:00:00Z",
			},
		],
	});
	const router = createMemoryRouter(
		[
			{ path: "/dashboard", element: <DashboardPage /> },
			{ path: "/achievements/new", element: <p>新建页</p> },
			{ path: "/achievements/:id/edit", element: <p>编辑页</p> },
		],
		{ initialEntries: ["/dashboard"] },
	);
	render(<RouterProvider router={router} />);

	expect(await screen.findByText(/需要您处理/)).toBeTruthy();
	expect(screen.getByText(/1 项成果被退回待补正/)).toBeTruthy();
	expect(screen.getByText(/2 项草稿尚未提交审核/)).toBeTruthy();
	expect(screen.getByRole("link", { name: "修改退回的示范课" })).toBeTruthy();

	fireEvent.click(screen.getByRole("link", { name: "新建成果" }));
	expect(router.state.location.pathname).toBe("/achievements/new");
	expect(router.state.location.state).toEqual({
		from: "/achievements?scope=all&status=approved&page=2",
	});
});

it("零成果时显示三步新手引导并支持关闭，教研组长只读角色显示权限说明", async () => {
	vi.spyOn(achievementApi, "dashboard").mockResolvedValue({
		counts: { draft: 0, pending: 0, returned: 0, approved: 0, archived: 0 },
		review_pending: 0,
		recent: [],
	});
	const router = createMemoryRouter(
		[{ path: "/dashboard", element: <DashboardPage /> }],
		{ initialEntries: ["/dashboard"] },
	);
	render(<RouterProvider router={router} />);

	expect(await screen.findByText("第一次使用？三步完成成果申报")).toBeTruthy();
	fireEvent.click(screen.getByRole("button", { name: "关闭新手引导" }));
	expect(screen.queryByText("第一次使用？三步完成成果申报")).toBeNull();

	cleanup();
	auth.session.user.grants = [];
	render(<RouterProvider router={router} />);
	expect(await screen.findByText(/当前账号为成果查阅权限/)).toBeTruthy();
});
