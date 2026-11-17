// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { AppShell } from "@/components/layout/AppShell";
import { AchievementDetailPage } from "./AchievementDetailPage";
import { type AchievementDetail, achievementApi, type Catalog } from "./api";

const auth = vi.hoisted(() => ({
	session: {
		csrf_token: "csrf",
		user: {
			id: 3,
			display_name: "审核老师",
			teacher: { department_name: "教研组" },
			grants: [{ permission: "reviews:write" }],
		},
	},
	refresh: vi.fn(),
	logout: vi.fn(),
}));
vi.mock("@/features/auth/AuthProvider", () => ({ useAuth: () => auth }));
const detail = {
	record: {
		id: 12,
		title: "公开课",
		type_name: "公开课",
		teacher_name: "填报老师",
		version: 2,
		status: "pending",
	},
	schema: { fields: [] },
	data: { title: "公开课", participants: [] },
	files: [],
	events: [],
	can_review: true,
	can_archive: false,
	can_edit: false,
} as unknown as AchievementDetail;
function show(overrideDetail: AchievementDetail = detail) {
	vi.spyOn(achievementApi, "detail").mockResolvedValue(overrideDetail);
	vi.spyOn(achievementApi, "catalog").mockResolvedValue({
		dictionaries: [],
		types: [],
		terms: [],
		teachers: [],
	} as Catalog);
	vi.spyOn(achievementApi, "revisions").mockResolvedValue([]);
	const router = createMemoryRouter(
		[
			{
				element: <AppShell />,
				children: [
					{ path: "/achievements/:id", element: <AchievementDetailPage /> },
					{ path: "/achievements/:id/edit", element: <p>编辑成果页</p> },
					{ path: "/reviews", element: <p>审核队列</p> },
				],
			},
		],
		{
			initialEntries: [
				{
					pathname: "/achievements/12",
					state: { from: "/reviews?status=pending&page=3&search=公开课" },
				},
			],
		},
	);
	render(<RouterProvider router={router} />);
	return router;
}
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	auth.logout.mockClear();
	window.sessionStorage.clear();
});

it("浏览器刷新后按审核人与成果恢复意见草稿", async () => {
	show();
	fireEvent.change(await screen.findByLabelText("办理意见"), {
		target: { value: "刷新后继续补充" },
	});
	await waitFor(() =>
		expect(
			window.sessionStorage.getItem("teacher-oa:review-opinion:3:12"),
		).toBe("刷新后继续补充"),
	);
	cleanup();
	show();
	expect(await screen.findByLabelText("办理意见")).toHaveProperty(
		"value",
		"刷新后继续补充",
	);
});

it("刷新成果和冲突重载保留意见；取消返回和退出保护输入，成功办理清空草稿", async () => {
	const router = show();
	const approvedDetail = {
		...detail,
		record: { ...detail.record, status: "approved", version: 3 },
		can_review: false,
		can_archive: true,
	} as AchievementDetail;
	vi.mocked(achievementApi.detail).mockResolvedValue(approvedDetail);
	const send = vi
		.spyOn(achievementApi, "review")
		.mockResolvedValue(approvedDetail);
	const input = await screen.findByLabelText("办理意见");
	fireEvent.change(input, { target: { value: "请补充日期\n保留第二行" } });
	vi.mocked(achievementApi.detail).mockResolvedValueOnce(detail);
	fireEvent.click(screen.getByRole("button", { name: "刷新成果" }));
	await waitFor(() =>
		expect(
			(screen.getByLabelText("办理意见") as HTMLTextAreaElement).value,
		).toBe("请补充日期\n保留第二行"),
	);
	fireEvent.click(screen.getByRole("link", { name: "返回审核列表" }));
	fireEvent.click(await screen.findByRole("button", { name: "继续编辑" }));
	expect(router.state.location.pathname).toBe("/achievements/12");
	fireEvent.click(screen.getByRole("button", { name: "退出登录" }));
	fireEvent.click(await screen.findByRole("button", { name: "继续编辑" }));
	expect(auth.logout).not.toHaveBeenCalled();
	expect(
		(screen.getByLabelText("办理意见") as HTMLTextAreaElement).value,
	).toContain("保留第二行");
	fireEvent.click(screen.getByRole("button", { name: "审核通过" }));
	fireEvent.click(await screen.findByRole("button", { name: "确认通过" }));
	await waitFor(() =>
		expect(send).toHaveBeenCalledWith(
			12,
			"approve",
			2,
			"请补充日期\n保留第二行",
			"csrf",
		),
	);
	await waitFor(() =>
		expect(
			(screen.getByLabelText("办理意见") as HTMLTextAreaElement).value,
		).toBe(""),
	);
	fireEvent.click(screen.getByRole("link", { name: "返回审核列表" }));
	await screen.findByText("审核队列");
	expect(router.state.location.search).toBe(
		"?status=pending&page=3&search=公开课",
	);
});

it("审核详情保持审核导航和可点击面包屑，历史展示当时人员与材料", async () => {
	const router = show();
	vi.mocked(achievementApi.revisions).mockResolvedValue([
		{
			version: 1,
			created_at: "2026-10-30T02:00:00Z",
			snapshot: {
				status: "draft",
				data: {
					title: "旧标题",
					participants: [
						{ external_name: "旧署名老师", role_code: "member", sort_order: 0 },
					],
				},
			},
			files: [
				{
					id: 99,
					filename: "旧证明.png",
					size_bytes: 100,
					mime_type: "image/png",
				},
			],
		},
	] as never);
	fireEvent.click(await screen.findByRole("button", { name: "刷新成果" }));
	await screen.findByText("旧署名老师");
	const breadcrumb = document.querySelector(".breadcrumb");
	expect(breadcrumb?.textContent).toContain("审核中心");
	const nav = screen.getByRole("navigation", { name: "主导航" });
	expect(
		within(nav).getByRole("link", { name: "审核中心" }).className,
	).toContain("active");
	expect(
		within(nav).getByRole("link", { name: "我的成果" }).className,
	).not.toContain("active");
	expect(
		screen
			.getByRole("link", { name: "预览旧证明.png", hidden: true })
			.getAttribute("href"),
	).toBe("/api/v1/files/99/preview");
	// Clickable breadcrumb navigates back to the filtered review list
	const crumbLink = within(breadcrumb as HTMLElement).getByRole("link", {
		name: "审核中心",
	});
	fireEvent.click(crumbLink);
	await screen.findByText("审核队列");
	expect(router.state.location.search).toBe(
		"?status=pending&page=3&search=公开课",
	);
});

it("退回补正状态在退回意见区提供去修改并重新提交直达入口", async () => {
	const router = show({
		...detail,
		record: { ...detail.record, status: "returned" },
		can_review: false,
		can_edit: true,
		events: [
			{
				id: 1,
				reviewer_id: 3,
				reviewer_name: "审核员",
				action: "return",
				old_status: "pending",
				new_status: "returned",
				opinion: "请补充获奖证书原件扫描件",
				created_at: "2026-11-01T08:00:00Z",
			},
		],
	});
	const editCta = await screen.findByRole("link", {
		name: "去修改并重新提交",
	});
	fireEvent.click(editCta);
	await screen.findByText("编辑成果页");
	expect(router.state.location.pathname).toBe("/achievements/12/edit");
});
