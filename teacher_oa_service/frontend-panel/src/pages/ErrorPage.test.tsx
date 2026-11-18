// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { createMemoryRouter, Outlet, RouterProvider } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ErrorPage, RouteErrorPage } from "./ErrorPage";

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

function CrashComponent({ chunk = false }: { chunk?: boolean }): never {
	throw new Error(
		chunk
			? "Failed to fetch dynamically imported module: /assets/Dashboard-hash.js"
			: "Unexpected render failure",
	);
}

describe("ErrorPage & RouteErrorPage", () => {
	it("404 路由展示明确的 404 标题与返回工作台入口", () => {
		const router = createMemoryRouter(
			[{ path: "*", element: <ErrorPage inShell /> }],
			{ initialEntries: ["/unknown-page"] },
		);
		render(<RouterProvider router={router} />);
		expect(screen.getByText("404 · 页面未找到")).toBeTruthy();
		expect(
			screen.getByRole("heading", { name: "未找到您访问的页面" }),
		).toBeTruthy();
		expect(screen.getByRole("link", { name: "返回工作台" })).toBeTruthy();
	});

	it("子路由渲染异常在壳层内被捕获，保留导航并提供刷新与返回首页操作", () => {
		vi.spyOn(console, "error").mockImplementation(() => {});
		const router = createMemoryRouter(
			[
				{
					element: (
						<div>
							<nav aria-label="壳层导航">工作台侧栏</nav>
							<Outlet />
						</div>
					),
					children: [
						{
							errorElement: <RouteErrorPage inShell />,
							children: [
								{ path: "/broken", element: <CrashComponent /> },
								{ path: "/stale", element: <CrashComponent chunk /> },
							],
						},
					],
				},
			],
			{ initialEntries: ["/broken"] },
		);
		render(<RouterProvider router={router} />);
		expect(screen.getByLabelText("壳层导航")).toBeTruthy();
		expect(
			screen.getByRole("heading", { name: "当前页面加载出现异常" }),
		).toBeTruthy();
		expect(screen.getByRole("button", { name: "刷新页面" })).toBeTruthy();
		expect(screen.getByRole("link", { name: "返回工作台" })).toBeTruthy();
	});

	it("动态分块加载过期时识别为版本更新提示", () => {
		vi.spyOn(console, "error").mockImplementation(() => {});
		const router = createMemoryRouter(
			[
				{
					path: "/stale",
					element: <CrashComponent chunk />,
					errorElement: <RouteErrorPage />,
				},
			],
			{ initialEntries: ["/stale"] },
		);
		render(<RouterProvider router={router} />);
		expect(
			screen.getByRole("heading", { name: "检测到系统版本已更新" }),
		).toBeTruthy();
		expect(screen.getByRole("button", { name: "刷新页面" })).toBeTruthy();
	});
});
