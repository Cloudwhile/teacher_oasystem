// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProtectedRoute } from "@/components/layout/ProtectedRoute";
import { AuthProvider, useAuth } from "./AuthProvider";
import type { AuthResponse } from "./api";

function Identity() {
	const { session } = useAuth();
	return <p>{session?.user.teacher.name}</p>;
}
function TestApp() {
	return (
		<AuthProvider>
			<MemoryRouter initialEntries={["/dashboard"]}>
				<Routes>
					<Route path="/login" element={<p>登录入口</p>} />
					<Route element={<ProtectedRoute />}>
						<Route path="/dashboard" element={<Identity />} />
					</Route>
				</Routes>
			</MemoryRouter>
		</AuthProvider>
	);
}

const identity: AuthResponse = {
	csrf_token: "test-csrf",
	user: {
		id: 1,
		display_name: "测试教师",
		roles: ["teacher"],
		grants: [{ permission: "teachers:read", scope: "SELF" }],
		teacher: {
			id: 1,
			user_id: 1,
			name: "测试教师",
			employee_no: "T001",
			department_id: 1,
			department_name: "数学教研组",
			subject_code: "math",
			employment_status: "active",
		},
	},
};

describe("身份门禁", () => {
	afterEach(() => {
		cleanup();
		vi.unstubAllGlobals();
	});
	it("未登录时转向登录入口", async () => {
		vi.stubGlobal(
			"fetch",
			vi.fn(
				async () =>
					new Response(JSON.stringify({ code: "unauthorized" }), {
						status: 401,
					}),
			),
		);
		render(<TestApp />);
		expect(await screen.findByText("登录入口")).toBeTruthy();
	});
	it("使用接口返回的教师身份", async () => {
		vi.stubGlobal(
			"fetch",
			vi.fn(
				async () =>
					new Response(JSON.stringify({ code: "success", data: identity })),
			),
		);
		render(<TestApp />);
		expect(await screen.findByText("测试教师")).toBeTruthy();
	});
	it.each([
		[403, "当前账户无法访问，请联系学校管理员。"],
		[503, "暂时无法连接，请稍后重试。"],
	])("正确展示 %s 状态", async (status, message) => {
		vi.stubGlobal(
			"fetch",
			vi.fn(async () => new Response("{}", { status })),
		);
		render(<TestApp />);
		expect(await screen.findByText(message)).toBeTruthy();
		expect(screen.queryByText("测试教师")).toBeNull();
	});
});
