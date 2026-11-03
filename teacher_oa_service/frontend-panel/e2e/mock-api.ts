import type { Page, Route } from "playwright/test";

// Self-contained API stand-in for layout and navigation checks, so they run
// without a database. Business flows stay covered by the backend-backed specs.
export type Role = "teacher" | "reviewer" | "admin";

const permissions: Record<Role, string[]> = {
	teacher: ["achievements:write"],
	reviewer: ["achievements:write", "reviews:write"],
	admin: ["achievements:write", "reviews:write", "admin:manage"],
};
const now = "2026-10-20T08:00:00Z";
function record(id: number, title: string) {
	return {
		id,
		title,
		type_code: "open_class",
		type_name: "公开课",
		status: "draft",
		teacher_id: 1,
		teacher_name: "王老师",
		version: 1,
		created_at: now,
		updated_at: now,
	};
}
const titleField = {
	key: "title",
	label: "成果名称",
	type: "text",
	required: true,
	visible: true,
	order: 1,
};
const schema = {
	code: "open_class",
	name: "公开课",
	category: "teaching",
	version: 1,
	fields: [titleField],
	json_schema: {},
};

export async function mockApi(
	page: Page,
	role: Role,
	options: { departmentName?: string; logoutFails?: boolean } = {},
) {
	let signedIn = true;
	const user = {
		id: 1,
		display_name: "王老师",
		roles: [role],
		grants: permissions[role].map((permission) => ({
			permission,
			scope: "SELF",
		})),
		teacher: {
			id: 1,
			user_id: 1,
			name: "王老师",
			department_id: 1,
			department_name: options.departmentName ?? "数学教研组",
			employee_no: "T-001",
			employment_status: "active",
			subject_code: "math",
		},
	};
	const ok = (route: Route, data: unknown) =>
		route.fulfill({ json: { code: "success", msg: "ok", data } });
	await page.route("**/api/v1/**", async (route) => {
		const url = new URL(route.request().url());
		const path = url.pathname.replace("/api/v1", "");
		const params = url.searchParams;
		if (path === "/auth/me")
			return signedIn
				? ok(route, { user, csrf_token: "csrf", must_change_password: false })
				: route.fulfill({ status: 401, json: { msg: "unauthorized" } });
		if (path === "/auth/logout") {
			if (options.logoutFails)
				return route.fulfill({ status: 500, json: { msg: "failed" } });
			signedIn = false;
			return ok(route, null);
		}
		if (path === "/auth/options")
			return ok(route, {
				password_enabled: true,
				mock_enabled: false,
				wecom_enabled: false,
			});
		if (path === "/login-page")
			return ok(route, {
				title: "TeacherOA",
				background_url: "",
				logo_url: "",
				footer: "",
				announcements: [],
				version: 0,
			});
		if (path === "/dashboard")
			return ok(route, {
				counts: { draft: 2 },
				recent: [record(7, "最近的公开课")],
				review_pending: 1,
			});
		if (path === "/achievements") {
			const scope = params.get("scope") === "all" ? "全部" : "我的";
			const pageNo = Number(params.get("page") ?? 1);
			const search = params.get("search");
			const label = `${scope}成果${search ? `-${search}` : ""}-第${pageNo}页`;
			return ok(route, {
				items: [record(10 + pageNo, label)],
				page: pageNo,
				page_size: 1,
				total: 3,
			});
		}
		if (path === "/reviews/pending")
			return ok(route, {
				items: [record(31, "待审核的公开课")],
				page: 1,
				page_size: 20,
				total: 1,
			});
		const detail = /^\/achievements\/(\d+)(\/revisions)?$/.exec(path);
		if (detail?.[2]) return ok(route, []);
		if (detail)
			return ok(route, {
				record: record(Number(detail[1]), `成果${detail[1]}`),
				schema,
				data: {
					type_code: "open_class",
					schema_version: 1,
					title: `成果${detail[1]}`,
					participants: [],
				},
				files: [],
				events: [],
				can_edit: true,
				can_review: false,
				can_archive: false,
			});
		if (path === "/achievement-types")
			return ok(route, [
				{
					code: "open_class",
					name: "公开课",
					category: "teaching",
					current_version: 1,
				},
			]);
		if (path === "/achievement-types/open_class/schema")
			return ok(route, schema);
		if (
			path === "/academic-terms" ||
			path === "/dictionaries" ||
			path === "/teachers"
		)
			return ok(route, []);
		if (path === "/files")
			return ok(route, { items: [], page: 1, page_size: 20, total: 0 });
		if (path === "/teachers/me") return ok(route, user.teacher);
		return route.fulfill({ status: 404, json: { msg: "not mocked" } });
	});
}
