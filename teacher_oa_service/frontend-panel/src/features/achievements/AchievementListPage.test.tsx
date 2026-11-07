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
