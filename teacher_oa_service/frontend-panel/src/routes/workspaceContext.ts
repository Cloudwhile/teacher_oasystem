import { appPaths } from "./routePaths";

export function workspaceContext(
	pathname: string,
	search: string,
	from: unknown,
) {
	if (pathname.startsWith("/achievements")) {
		if (pathname === "/achievements/new")
			return { title: "成果填报", section: "create" };
		if (pathname === "/achievements")
			return {
				title:
					new URLSearchParams(search).get("scope") === "all"
						? "成果查询"
						: "我的成果",
				section: "achievements",
			};
		if (
			typeof from === "string" &&
			(from === "/reviews" || from.startsWith("/reviews?"))
		)
			return { title: "审核中心", section: "reviews" };
		return {
			title: pathname.endsWith("/edit") ? "编辑成果" : "成果详情",
			section: "achievements",
		};
	}
	const contexts: Record<string, { title: string; section: string }> = {
		[appPaths.dashboard]: { title: "工作台", section: "dashboard" },
		[appPaths.profile]: { title: "个人档案", section: "profile" },
		"/reviews": { title: "审核中心", section: "reviews" },
		"/files": { title: "文件资料", section: "files" },
		"/account": { title: "账号安全", section: "account" },
		"/admin/accounts": { title: "成员与账号", section: "accounts" },
		"/admin/login": { title: "登录设置", section: "login-settings" },
		"/admin/login-page": { title: "登录页与公告", section: "login-page" },
	};
	return contexts[pathname] ?? { title: "页面未找到", section: "" };
}
