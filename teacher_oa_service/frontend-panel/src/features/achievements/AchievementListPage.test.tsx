// @vitest-environment jsdom
import { cleanup, fireEvent, screen } from "@testing-library/react";
import { useLocation } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import {
	identity,
	json,
	mockHttp,
	showPage,
} from "@/features/admin/adminTestHarness";
import { AchievementEditorPage } from "./AchievementEditorPage";
import { AchievementListPage } from "./AchievementListPage";

function Page() {
	return useLocation().pathname === "/achievements/new" ? (
		<AchievementEditorPage />
	) : (
		<AchievementListPage />
	);
}
afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});
it("成果查询新建后返回恢复筛选与分页", async () => {
	mockHttp((url) => {
		if (url.endsWith("/auth/me")) return json(identity);
		if (url.includes("/schema"))
			return json({
				code: "open_class",
				name: "公开课",
				version: 1,
				category: "teaching",
				fields: [],
				json_schema: {},
			});
		if (url.startsWith("/api/v1/achievements?"))
			return json({ items: [], page: 2, page_size: 20, total: 40 });
		return json([]);
	});
	const path = "/achievements?scope=all&type_code=research_project&page=2";
	const router = showPage(<Page />, path);
	fireEvent.click(await screen.findByRole("link", { name: "新建成果" }));
	fireEvent.click(await screen.findByRole("link", { name: "返回成果列表" }));
	expect(router.state.location.pathname + router.state.location.search).toBe(
		path,
	);
	expect(await screen.findByRole("heading", { name: "成果查询" })).toBeTruthy();
});
it("切换下拉筛选时保留已输入的搜索词", async () => {
	mockHttp((url) => {
		if (url.endsWith("/auth/me")) return json(identity);
		if (url.startsWith("/api/v1/achievements?"))
			return json({ items: [], page: 1, page_size: 20, total: 0 });
		return json([]);
	});
	const router = showPage(<Page />, "/achievements");
	const search = await screen.findByLabelText("搜索成果名称");
	fireEvent.change(search, { target: { value: "人工智能教学" } });
	fireEvent.change(screen.getByLabelText("筛选状态"), {
		target: { value: "approved" },
	});
	const nextParams = new URLSearchParams(router.state.location.search);
	expect(nextParams.get("search")).toBe("人工智能教学");
	expect(nextParams.get("status")).toBe("approved");
});

it("个人视图标注参与成果的第一申报人，并使去修改按钮无障碍名称包含可见文案", async () => {
	mockHttp((url) => {
		if (url.endsWith("/auth/me")) return json(identity);
		if (url.startsWith("/api/v1/achievements?")) {
			return json({
				items: [
					{
						id: 11,
						teacher_id: 2,
						teacher_name: "李四",
						title: "协作教学课题",
						type_code: "research_project",
						type_name: "教科研项目",
						subject_code: "math",
						academic_term_id: 1,
						occurred_at: "2026-10-01",
						level_code: "city",
						award_grade_code: "first",
						status: "approved",
						version: 1,
						created_at: "2026-10-01T00:00:00Z",
						updated_at: "2026-10-02T00:00:00Z",
					},
					{
						id: 12,
						teacher_id: 1,
						teacher_name: "原管理员",
						title: "退回修改成果",
						type_code: "research_project",
						type_name: "教科研项目",
						subject_code: "math",
						academic_term_id: 1,
						occurred_at: "2026-10-01",
						level_code: "city",
						award_grade_code: "first",
						status: "returned",
						version: 1,
						created_at: "2026-10-01T00:00:00Z",
						updated_at: "2026-10-02T00:00:00Z",
					},
				],
				page: 1,
				page_size: 20,
				total: 2,
			});
		}
		return json([]);
	});
	showPage(<Page />, "/achievements?scope=mine");
	expect(await screen.findByText("参与 · 李四")).toBeTruthy();
	expect(
		screen.getByRole("link", { name: "去修改：退回修改成果" }),
	).toBeTruthy();
});
