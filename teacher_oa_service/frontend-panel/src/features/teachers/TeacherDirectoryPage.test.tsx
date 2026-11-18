// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { json, mockHttp } from "@/features/admin/adminTestHarness";
import { AuthProvider } from "@/features/auth/AuthProvider";
import { TeacherDirectoryPage } from "./TeacherDirectoryPage";
import { TeacherDossierPage } from "./TeacherDossierPage";

const leaderSession = {
	csrf_token: "csrf",
	must_change_password: false,
	user: {
		id: 10,
		display_name: "张校长",
		roles: ["teacher", "school_leader"],
		grants: [
			{ permission: "teachers:directory", scope: "ALL" },
			{ permission: "teachers:read", scope: "ALL" },
			{ permission: "achievements:read", scope: "ALL" },
			{ permission: "files:read", scope: "ALL" },
		],
		teacher: {
			id: 10,
			user_id: 10,
			employee_no: "ADM-01",
			name: "张校长",
			department_id: 1,
			department_name: "校长室",
			subject_code: "education_management",
			employment_status: "active",
		},
	},
};

const plainTeacherSession = {
	...leaderSession,
	user: {
		...leaderSession.user,
		roles: ["teacher"],
		grants: [{ permission: "achievements:write", scope: "SELF" }],
	},
};

function renderTeachersApp(initialEntry: string) {
	const router = createMemoryRouter(
		[
			{ path: "/teachers", element: <TeacherDirectoryPage /> },
			{ path: "/teachers/:id", element: <TeacherDossierPage /> },
			{ path: "/dashboard", element: <p>工作台</p> },
		],
		{ initialEntries: [initialEntry] },
	);
	render(
		<AuthProvider>
			<RouterProvider router={router} />
		</AuthProvider>,
	);
	return router;
}

afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
});

it("普通教师访问全校教师资料页面时显示无权限提示", async () => {
	mockHttp((url) => {
		if (url.endsWith("/auth/me")) return json(plainTeacherSession);
		throw new Error(`Unexpected request: ${url}`);
	});
	renderTeachersApp("/teachers");
	expect(
		await screen.findByText("仅系统管理员与校级行政可查阅全校教师资料"),
	).toBeTruthy();
});

it("校级行政可按部门筛选全校教师并点入查看教师档案、成果与材料", async () => {
	mockHttp((url) => {
		if (url.endsWith("/auth/me")) return json(leaderSession);
		if (url.includes("/teachers/directory")) {
			return json({
				departments: [
					{ id: 91001, name: "数学教研组" },
					{ id: 91002, name: "语文教研组" },
				],
				items: [
					{
						id: 91001,
						user_id: 91001,
						employee_no: "DEMO-91001",
						name: "李老师",
						department_id: 91001,
						department_name: "数学教研组",
						subject_code: "math",
						employment_status: "active",
						account_status: "active",
						roles: ["teacher"],
						achievement_count: 2,
						approved_count: 1,
						material_count: 1,
					},
				],
				page: 1,
				page_size: 20,
				total: 1,
			});
		}
		if (url.endsWith("/teachers/91001/dossier")) {
			return json({
				profile: {
					id: 91001,
					user_id: 91001,
					employee_no: "DEMO-91001",
					name: "李老师",
					department_id: 91001,
					department_name: "数学教研组",
					subject_code: "math",
					employment_status: "active",
				},
				account_status: "active",
				roles: ["teacher"],
				created_at: "2026-10-01T08:00:00Z",
				counts: { pending: 1, returned: 0, approved: 1, archived: 0 },
				achievements: [
					{
						id: 501,
						teacher_id: 91001,
						teacher_name: "李老师",
						title: "数学建模市级公开课",
						type_code: "open_class",
						type_name: "公开课",
						subject_code: "math",
						academic_term_id: 1,
						occurred_at: "2026-10-18",
						level_code: null,
						award_grade_code: null,
						status: "approved",
						version: 2,
						created_at: "2026-10-18T08:00:00Z",
						updated_at: "2026-10-19T08:00:00Z",
					},
				],
				achievement_total: 2,
				files: [
					{
						id: 88,
						filename: "公开课获奖证书.pdf",
						size_bytes: 20480,
						mime_type: "application/pdf",
						sha256: "abc",
						uploaded_by: 91001,
						upload_status: "ready",
						created_at: "2026-10-18T08:00:00Z",
						association: "linked",
					},
				],
			});
		}
		throw new Error(`Unexpected request: ${url}`);
	});

	const router = renderTeachersApp("/teachers");
	expect(await screen.findByText("李老师")).toBeTruthy();
	expect(screen.getByText("2 项（已认定 1）")).toBeTruthy();

	fireEvent.change(screen.getByLabelText("筛选所属部门"), {
		target: { value: "91001" },
	});
	expect(
		new URLSearchParams(router.state.location.search).get("department_id"),
	).toBe("91001");

	fireEvent.click(await screen.findByRole("link", { name: "李老师" }));
	expect(
		await screen.findByRole("heading", { name: "李老师", level: 1 }),
	).toBeTruthy();
	expect(screen.getByText("数学建模市级公开课")).toBeTruthy();
	expect(screen.getByText("公开课获奖证书.pdf")).toBeTruthy();
});
