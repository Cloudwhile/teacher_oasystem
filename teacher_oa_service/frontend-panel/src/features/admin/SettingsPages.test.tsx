// @vitest-environment jsdom
import { cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AccountPage } from "@/features/auth/AccountPage";
import { defaultLoginPage } from "@/features/auth/loginPageApi";
import { AccountsPage } from "./AccountsPage";
import { identity, json, mockHttp, showPage } from "./adminTestHarness";
import { LoginAppearancePage } from "./LoginAppearancePage";
import { WecomSettingsPage } from "./WecomSettingsPage";

const wecom = {
	version: 1,
	enabled: false,
	corp_id: "oldcorp",
	agent_id: "1",
	secret_configured: true,
	callback_url: "https://oa.test/callback",
};
const account = {
	id: 1,
	username: "admin",
	name: "原管理员",
	department: "学校",
	role: "admin",
	active: true,
	must_change: false,
};
afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
});

it.each(["password", "account", "wecom", "appearance", "image"] as const)(
	"%s 写入会话失效后回到登录",
	async (kind) => {
		let expired = false;
		mockHttp((url, init) => {
			if (url.endsWith("/auth/me")) return json(identity, expired ? 401 : 200);
			if (init?.method === "POST" || init?.method === "PUT") {
				expired = true;
				return json(null, 401);
			}
			if (url.endsWith("/admin/accounts")) return json({ items: [account] });
			if (url.endsWith("/admin/wecom")) return json(wecom);
			if (url.endsWith("/admin/login-page")) return json(defaultLoginPage);
			throw new Error(`Unexpected request ${url}`);
		});
		if (kind === "password") {
			showPage(<AccountPage />, "/account");
			fireEvent.change(await screen.findByLabelText(/^当前密码/), {
				target: { value: "old-password-123" },
			});
			fireEvent.change(screen.getByLabelText(/^新密码/), {
				target: { value: "new-password-123" },
			});
			fireEvent.change(screen.getByLabelText(/^确认新密码/), {
				target: { value: "new-password-123" },
			});
			fireEvent.click(screen.getByRole("button", { name: "更新密码" }));
		} else if (kind === "account") {
			showPage(<AccountsPage />, "/admin/accounts");
			fireEvent.click(await screen.findByRole("button", { name: "编辑admin" }));
			fireEvent.click(screen.getByRole("button", { name: "保存账号" }));
		} else if (kind === "wecom") {
			showPage(<WecomSettingsPage />, "/admin/login");
			fireEvent.click(
				await screen.findByRole("button", { name: "保存企业微信设置" }),
			);
		} else {
			showPage(<LoginAppearancePage />, "/admin/login-page");
			if (kind === "image") {
				fireEvent.change(await screen.findByLabelText("选择背景图片"), {
					target: {
						files: [new File(["image"], "test.png", { type: "image/png" })],
					},
				});
			} else
				fireEvent.click(
					await screen.findByRole("button", { name: "保存设置" }),
				);
		}
		expect(await screen.findByText("登录目的页")).toBeTruthy();
	},
);

it.each([400, 409, 500])("企业微信写入 %s 保留表单与身份", async (status) => {
	mockHttp((url, init) =>
		url.endsWith("/auth/me")
			? json(identity)
			: init?.method === "PUT"
				? json(null, status)
				: json(wecom),
	);
	showPage(<WecomSettingsPage />, "/admin/login");
	fireEvent.change(await screen.findByLabelText("企业ID"), {
		target: { value: "localcorp" },
	});
	fireEvent.click(screen.getByRole("button", { name: "保存企业微信设置" }));
	await screen.findByText("保存失败");
	expect(screen.getByLabelText("企业ID")).toHaveProperty("value", "localcorp");
	expect(screen.getByLabelText("当前账户姓名").textContent).toBe("原管理员");
});

it.each(["wecom", "appearance"] as const)(
	"%s 冲突取消保留草稿，确认重载使用新版本",
	async (kind) => {
		let version = 1;
		let saved = false;
		mockHttp((url, init) => {
			if (url.endsWith("/auth/me")) return json(identity);
			if (init?.method === "PUT") {
				const body: unknown = JSON.parse(String(init.body));
				if (typeof body !== "object" || body === null || !("version" in body))
					throw new Error("Missing version");
				if (body.version !== 2) {
					version = 2;
					return json(null, 409);
				}
				saved = true;
				return json(
					kind === "wecom"
						? { ...wecom, version: 3, corp_id: "latestcorp" }
						: { ...defaultLoginPage, version: 3, title: "服务端新标题" },
				);
			}
			return json(
				kind === "wecom"
					? {
							...wecom,
							version,
							corp_id: version === 1 ? "oldcorp" : "latestcorp",
						}
					: {
							...defaultLoginPage,
							version,
							title: version === 1 ? "旧标题" : "服务端新标题",
						},
			);
		});
		showPage(
			kind === "wecom" ? <WecomSettingsPage /> : <LoginAppearancePage />,
			kind === "wecom" ? "/admin/login" : "/admin/login-page",
		);
		const label = kind === "wecom" ? "企业ID" : "网站名称";
		const save = kind === "wecom" ? "保存企业微信设置" : "保存设置";
		fireEvent.change(await screen.findByLabelText(new RegExp(`^${label}`)), {
			target: { value: "localdraft" },
		});
		fireEvent.click(screen.getByRole("button", { name: save }));
		fireEvent.click(
			await screen.findByRole("button", { name: "加载最新设置" }),
		);
		fireEvent.click(await screen.findByRole("button", { name: "继续编辑" }));
		expect(screen.getByLabelText(new RegExp(`^${label}`))).toHaveProperty(
			"value",
			"localdraft",
		);
		fireEvent.click(screen.getByRole("button", { name: "加载最新设置" }));
		fireEvent.click(await screen.findByRole("button", { name: "放弃并离开" }));
		const field = await screen.findByDisplayValue(
			kind === "wecom" ? "latestcorp" : "服务端新标题",
		);
		expect(field).toBeTruthy();
		fireEvent.click(screen.getByRole("button", { name: save }));
		await screen.findByText(
			kind === "wecom" ? "企业微信设置已保存" : "登录页设置已保存",
		);
		expect(saved).toBe(true);
	},
);

it.each([true, false])(
	"编辑成员时仅自身更新当前身份：self=%s",
	async (self) => {
		let name = "原管理员";
		mockHttp((url, init) => {
			if (url.endsWith("/auth/me"))
				return json({
					...identity,
					user: { ...identity.user, display_name: name },
				});
			if (init?.method === "POST") {
				name = "修改后的管理员";
				return json({ items: [] });
			}
			return json({ items: [{ ...account, id: self ? 1 : 2 }] });
		});
		showPage(<AccountsPage />, "/admin/accounts");
		fireEvent.click(await screen.findByRole("button", { name: "编辑admin" }));
		fireEvent.change(screen.getByLabelText(/^姓名/), {
			target: { value: "修改后的管理员" },
		});
		fireEvent.click(screen.getByRole("button", { name: "保存账号" }));
		await screen.findByText("账号已保存");
		expect(screen.getByLabelText("当前账户姓名").textContent).toBe(
			self ? "修改后的管理员" : "原管理员",
		);
	},
);

it("删除公告不会误提交尚未保存的外观草稿", async () => {
	let putBody: Record<string, unknown> | null = null;
	const pageWithNotice = {
		...defaultLoginPage,
		title: "已保存标题",
		announcements: [
			{
				id: "notice-1",
				title: "系统维护通知",
				content: "今晚升级",
				published: true,
				published_at: "2026-11-15T18:30:00Z",
			},
		],
	};
	mockHttp((url, init) => {
		if (url.endsWith("/auth/me")) return json(identity);
		if (init?.method === "PUT") {
			putBody = JSON.parse(String(init.body)) as Record<string, unknown>;
			return json({
				...pageWithNotice,
				version: 2,
				announcements: [],
			});
		}
		return json(pageWithNotice);
	});
	showPage(<LoginAppearancePage />, "/admin/login-page");
	expect(await screen.findByText(/2026-11-16/)).toBeTruthy();
	fireEvent.change(screen.getByLabelText(/^网站名称/), {
		target: { value: "未保存新标题" },
	});
	fireEvent.click(screen.getByRole("button", { name: "删除" }));
	fireEvent.click(await screen.findByRole("button", { name: "确认删除" }));
	await screen.findByText("暂无公告");
	expect(putBody).toMatchObject({
		title: "已保存标题",
		announcements: [],
	});
	expect(screen.getByLabelText(/^网站名称/)).toHaveProperty(
		"value",
		"未保存新标题",
	);
});
