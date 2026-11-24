// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
} from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { ConfirmProvider } from "@/components/common/ConfirmDialog";
import { AchievementEditorPage } from "./AchievementEditorPage";
import { achievementApi, type Catalog } from "./api";

const auth = vi.hoisted(() => ({
	session: {
		csrf_token: "csrf",
		user: {
			id: 7,
			display_name: "张老师",
			teacher: { id: 7, department_name: "物理组" },
			grants: [{ permission: "achievements:write", scope: "SELF" }],
		},
	},
	refresh: vi.fn(),
}));
vi.mock("@/features/auth/AuthProvider", () => ({
	useAuth: () => auth,
	useOptionalAuth: () => auth,
}));

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	window.localStorage.clear();
});

function mountNewEditor() {
	vi.spyOn(achievementApi, "catalog").mockResolvedValue({
		types: [{ code: "open_class", name: "公开课", category: "teaching" }],
		terms: [],
		dictionaries: [],
		teachers: [],
	} as unknown as Catalog);
	vi.spyOn(achievementApi, "schema").mockResolvedValue({
		code: "open_class",
		name: "公开课",
		category: "teaching",
		version: 1,
		fields: [
			{
				key: "title",
				label: "成果名称",
				type: "text",
				required: true,
				visible: true,
				order: 1,
			},
		],
		json_schema: {},
	});
	const router = createMemoryRouter(
		[{ path: "/achievements/new", element: <AchievementEditorPage /> }],
		{ initialEntries: ["/achievements/new"] },
	);
	render(
		<ConfirmProvider>
			<RouterProvider router={router} />
		</ConfirmProvider>,
	);
	return router;
}

it("自动将脏草稿暂存至本地并在重新进入时支持一键恢复", async () => {
	mountNewEditor();
	const titleInput = await screen.findByLabelText(/^成果名称/);
	fireEvent.change(titleInput, { target: { value: "跨会话自动暂存的公开课" } });

	await waitFor(
		() =>
			expect(
				window.localStorage.getItem(
					"teacher-oa:achievement-draft:7:new:open_class",
				),
			).toContain("跨会话自动暂存的公开课"),
		{ timeout: 1500 },
	);
	expect(await screen.findByText("本地已暂存")).toBeTruthy();

	// Simulate crash / session expiry + re-entry
	cleanup();
	mountNewEditor();
	expect(
		await screen.findByText(/检测到上次未提交的本地暂存草稿/),
	).toBeTruthy();
	fireEvent.click(screen.getByRole("button", { name: "恢复暂存草稿" }));
	expect((screen.getByLabelText(/^成果名称/) as HTMLInputElement).value).toBe(
		"跨会话自动暂存的公开课",
	);
});
