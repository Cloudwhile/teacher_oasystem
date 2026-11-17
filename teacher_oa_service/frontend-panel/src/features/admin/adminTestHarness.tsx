import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { vi } from "vitest";
import { ConfirmProvider } from "@/components/common/ConfirmDialog";
import { ProtectedRoute } from "@/components/layout/ProtectedRoute";
import { AuthProvider, useAuth } from "@/features/auth/AuthProvider";
import type { AuthResponse } from "@/features/auth/api";

export const identity: AuthResponse = {
	must_change_password: false,
	csrf_token: "csrf",
	user: {
		id: 1,
		display_name: "原管理员",
		roles: ["admin"],
		grants: [
			{ permission: "admin:manage", scope: "ALL" },
			{ permission: "achievements:write", scope: "SELF" },
		],
		teacher: {
			id: 1,
			user_id: 1,
			name: "原管理员",
			employee_no: "T1",
			department_id: 1,
			department_name: "学校",
			subject_code: "other",
			employment_status: "active",
		},
	},
};
export function json(data: unknown, status = 200) {
	return new Response(
		JSON.stringify({ data, msg: status === 200 ? "" : "保存失败" }),
		{ status },
	);
}
export function mockHttp(
	handle: (url: string, init?: RequestInit) => Response,
) {
	vi.stubGlobal(
		"fetch",
		vi.fn(async (input: RequestInfo | URL, init?: RequestInit) =>
			handle(String(input), init),
		),
	);
}
function CurrentName() {
	const { session } = useAuth();
	return (
		<output aria-label="当前账户姓名">{session?.user.display_name}</output>
	);
}
export function showPage(page: ReactNode, path: string) {
	const router = createMemoryRouter(
		[
			{ path: "/login", element: <p>登录目的页</p> },
			{
				element: <ProtectedRoute />,
				children: [
					{
						path: "*",
						element: (
							<>
								<CurrentName />
								{page}
							</>
						),
					},
				],
			},
		],
		{ initialEntries: [path] },
	);
	render(
		<AuthProvider>
			<ConfirmProvider>
				<RouterProvider router={router} />
			</ConfirmProvider>
		</AuthProvider>,
	);
	return router;
}
