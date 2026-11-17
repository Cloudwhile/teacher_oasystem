import { safeReturnPath } from "@/services/returnPath";
import { appPaths } from "./routePaths";

export type NavKey =
	| "dashboard"
	| "achievements-mine"
	| "achievements-all"
	| "achievements-new"
	| "reviews"
	| "files"
	| "profile"
	| "account"
	| "admin-accounts"
	| "admin-login"
	| "admin-login-page";

export type NavGroupKey = "achievements";

export type NavigationState = {
	/** Navigation item that represents the current page, if any. */
	activeKey: NavKey | null;
	/** `page` when the URL is the item itself; `location` for its detail pages. */
	activeMatch: "page" | "location";
	/** Group that owns the current page, used to expand it automatically. */
	activeGroup: NavGroupKey | null;
	title: string;
	trail: string[];
};

export type TrailPart = {
	label: string;
	to?: string;
};

export const navLabels: Record<NavKey, string> = {
	dashboard: "工作台",
	"achievements-mine": "我的成果",
	"achievements-all": "成果查询",
	"achievements-new": "成果填报",
	reviews: "审核中心",
	files: "文件资料",
	profile: "个人档案",
	account: "账号安全",
	"admin-accounts": "成员与账号",
	"admin-login": "登录设置",
	"admin-login-page": "登录页与公告",
};

export const navGroupLabels: Record<NavGroupKey, string> = {
	achievements: "成果管理",
};

const pages: Record<
	string,
	{ key: NavKey; parent?: string; parentTo?: string }
> = {
	[appPaths.dashboard]: { key: "dashboard" },
	"/reviews": { key: "reviews" },
	"/files": { key: "files" },
	[appPaths.profile]: {
		key: "profile",
		parent: "个人账户",
		parentTo: appPaths.profile,
	},
	"/account": {
		key: "account",
		parent: "个人账户",
		parentTo: appPaths.profile,
	},
	"/admin/accounts": {
		key: "admin-accounts",
		parent: "系统管理",
		parentTo: "/admin/accounts",
	},
	"/admin/login": {
		key: "admin-login",
		parent: "系统管理",
		parentTo: "/admin/accounts",
	},
	"/admin/login-page": {
		key: "admin-login-page",
		parent: "系统管理",
		parentTo: "/admin/accounts",
	},
};

const groupLabel = navGroupLabels.achievements;

function trimPath(pathname: string) {
	return pathname.length > 1 ? pathname.replace(/\/+$/, "") : pathname;
}

function listKey(search: string): NavKey {
	return new URLSearchParams(search).get("scope") === "all"
		? "achievements-all"
		: "achievements-mine";
}

/**
 * Maps `location.state.from` (written by lists and kept by detail/edit pages)
 * back to the list the user came from. Unknown or missing sources return
 * `null` so a deep link is never attributed to a list it was not opened from.
 */
function sourceKey(from: unknown): NavKey | null {
	if (typeof from !== "string" || !from.startsWith("/")) return null;
	const index = from.indexOf("?");
	const pathname = trimPath(index === -1 ? from : from.slice(0, index));
	const search = index === -1 ? "" : from.slice(index);
	if (pathname === "/reviews") return "reviews";
	if (pathname === "/achievements") return listKey(search);
	return null;
}

export function resolveNavigation(
	pathname: string,
	search: string,
	from: unknown,
): NavigationState {
	const path = trimPath(pathname);
	if (path === "/achievements" || path === "/achievements/new") {
		const key = path === "/achievements" ? listKey(search) : "achievements-new";
		return {
			activeKey: key,
			activeMatch: "page",
			activeGroup: "achievements",
			title: navLabels[key],
			trail: [groupLabel, navLabels[key]],
		};
	}
	const record = /^\/achievements\/[^/]+(\/edit)?$/.exec(path);
	if (record) {
		const title = record[1] ? "编辑成果" : "成果详情";
		const source = sourceKey(from);
		if (source === "reviews")
			return {
				activeKey: "reviews",
				activeMatch: "location",
				activeGroup: null,
				title,
				trail: [navLabels.reviews, title],
			};
		return {
			activeKey: source,
			activeMatch: "location",
			activeGroup: "achievements",
			title,
			trail: source
				? [groupLabel, navLabels[source], title]
				: [groupLabel, title],
		};
	}
	const page = pages[path];
	if (page)
		return {
			activeKey: page.key,
			activeMatch: "page",
			activeGroup: null,
			title: navLabels[page.key],
			trail: page.parent
				? [page.parent, navLabels[page.key]]
				: [navLabels[page.key]],
		};
	return {
		activeKey: null,
		activeMatch: "page",
		activeGroup: null,
		title: "页面未找到",
		trail: ["页面未找到"],
	};
}

export function resolveTrailLinks(
	pathname: string,
	search: string,
	from: unknown,
): TrailPart[] {
	const path = trimPath(pathname);
	if (path === "/achievements") {
		const key = listKey(search);
		return [{ label: groupLabel }, { label: navLabels[key] }];
	}
	if (path === "/achievements/new") {
		return [
			{ label: groupLabel, to: "/achievements?scope=mine" },
			{ label: navLabels["achievements-new"] },
		];
	}
	const record = /^\/achievements\/([^/]+)(\/edit)?$/.exec(path);
	if (record) {
		const title = record[2] ? "编辑成果" : "成果详情";
		const source = sourceKey(from);
		const safeFrom = safeReturnPath(
			from,
			source === "reviews" ? "/reviews" : "/achievements?scope=mine",
		);
		if (source === "reviews") {
			return [{ label: navLabels.reviews, to: safeFrom }, { label: title }];
		}
		if (source) {
			return [
				{ label: groupLabel, to: "/achievements?scope=mine" },
				{ label: navLabels[source], to: safeFrom },
				{ label: title },
			];
		}
		return [
			{ label: groupLabel, to: "/achievements?scope=mine" },
			{ label: title },
		];
	}
	const page = pages[path];
	if (page) {
		return page.parent
			? [
					{
						label: page.parent,
						to: page.parentTo !== path ? page.parentTo : undefined,
					},
					{ label: navLabels[page.key] },
				]
			: [{ label: navLabels[page.key] }];
	}
	return [{ label: "页面未找到" }];
}
