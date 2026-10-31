// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
} from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { authApi } from "./api";
import { LoginPage } from "./LoginPage";
import { defaultLoginPage, loginPageApi } from "./loginPageApi";

const auth = vi.hoisted(() => ({
	status: "anonymous",
	session: null as null | { must_change_password: boolean },
	login: vi.fn(),
}));
vi.mock("./AuthProvider", () => ({ useAuth: () => auth }));
beforeEach(() => {
	vi.spyOn(loginPageApi, "public").mockResolvedValue(defaultLoginPage);
});
function show(path = "/login") {
	return render(
		<MemoryRouter initialEntries={[path]}>
			<Routes>
				<Route path="/login" element={<LoginPage />} />
				<Route path="/reviews" element={<p>审核目的页</p>} />
				<Route path="/dashboard" element={<p>工作台目的页</p>} />
				<Route path="/account" element={<p>修改密码目的页</p>} />
			</Routes>
		</MemoryRouter>,
	);
}
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	auth.status = "anonymous";
	auth.session = null;
	auth.login.mockReset();
});
it("提交真实账号密码，入口加载失败可以重试", async () => {
	const options = vi
		.spyOn(authApi, "options")
		.mockRejectedValueOnce(new Error("offline"))
		.mockResolvedValue({
			password_enabled: true,
			mock_enabled: false,
			wecom_enabled: false,
		});
	show();
	fireEvent.click(await screen.findByRole("button", { name: "重试登录入口" }));
	await waitFor(() => expect(options).toHaveBeenCalledTimes(2));
	const wecom = screen.getByRole("tab", { name: "企业微信登录" });
	expect(wecom.getAttribute("aria-disabled")).toBe("true");
	fireEvent.click(wecom);
	expect(
		screen.getByRole("tab", { name: "账号登录" }).getAttribute("aria-selected"),
	).toBe("true");
	expect(options).toHaveBeenCalledTimes(2);
	fireEvent.change(screen.getByRole("textbox", { name: "账号" }), {
		target: { value: "admin" },
	});
	fireEvent.change(screen.getByLabelText(/^密码/), {
		target: { value: "private-password" },
	});
	fireEvent.click(screen.getByRole("button", { name: "登录" }));
	await waitFor(() =>
		expect(auth.login).toHaveBeenCalledWith("admin", "private-password"),
	);
	expect(screen.queryByLabelText("测试身份")).toBeNull();
});
it("登录后恢复目标页面并拒绝外站跳转", () => {
	vi.spyOn(authApi, "options").mockResolvedValue({
		password_enabled: true,
		mock_enabled: false,
		wecom_enabled: false,
	});
	auth.status = "ready";
	auth.session = { must_change_password: false };
	const view = show("/login?next=%2Freviews%3Fstatus%3Dpending");
	expect(screen.getByText("审核目的页")).toBeTruthy();
	view.unmount();
	show("/login?next=https://evil.example");
	expect(screen.getByText("工作台目的页")).toBeTruthy();
});
it("初始密码账号必须进入改密页", () => {
	vi.spyOn(authApi, "options").mockResolvedValue({
		password_enabled: true,
		mock_enabled: false,
		wecom_enabled: false,
	});
	auth.status = "ready";
	auth.session = { must_change_password: true };
	show("/login?next=%2Freviews");
	expect(screen.getByText("修改密码目的页")).toBeTruthy();
});
it("企业微信链接携带原目标且不泄露密钥", async () => {
	vi.spyOn(authApi, "options").mockResolvedValue({
		password_enabled: true,
		mock_enabled: false,
		wecom_enabled: true,
	});
	show("/login?next=%2Freviews%3Fstatus%3Dpending");
	const wecom = screen.getByRole("tab", { name: "企业微信登录" });
	await waitFor(() =>
		expect(wecom.getAttribute("aria-disabled")).not.toBe("true"),
	);
	fireEvent.change(screen.getByRole("textbox", { name: "账号" }), {
		target: { value: "teacher" },
	});
	expect(screen.queryByRole("link", { name: "企业微信扫码登录" })).toBeNull();
	fireEvent.click(wecom);
	const link = await screen.findByRole("link", { name: "企业微信扫码登录" });
	expect(link.getAttribute("href")).toBe(
		"/api/v1/auth/wecom/login?mode=web&next=%2Freviews%3Fstatus%3Dpending",
	);
	expect(screen.queryByRole("textbox", { name: "账号" })).toBeNull();
	fireEvent.click(screen.getByRole("tab", { name: "账号登录" }));
	expect(
		(screen.getByRole("textbox", { name: "账号" }) as HTMLInputElement).value,
	).toBe("teacher");
});
