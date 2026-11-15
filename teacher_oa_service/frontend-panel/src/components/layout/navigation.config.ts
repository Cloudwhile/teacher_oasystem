import {
	Category,
	Check,
	FileText,
	Folder,
	type IconComponent,
	Layout,
	Settings,
	Users,
} from "reicon-react";
import type { Permission } from "@/features/auth/permissions";
import type { NavGroupKey, NavKey } from "@/routes/navigationState";
import { appPaths } from "@/routes/routePaths";

export type NavLinkItem = {
	kind: "link";
	key: NavKey;
	to: string;
	icon?: IconComponent;
	permission?: Permission;
};
export type NavGroupItem = {
	kind: "group";
	key: NavGroupKey;
	icon: IconComponent;
	children: NavLinkItem[];
	permission?: Permission;
};
export type NavSection = {
	id: string;
	label: string;
	permission?: Permission;
	items: (NavLinkItem | NavGroupItem)[];
};

export const navPaths = {
	achievementsMine: "/achievements?scope=mine",
	achievementsAll: "/achievements?scope=all",
	achievementsNew: "/achievements/new",
} as const;

export const navigationSections: NavSection[] = [
	{
		id: "workspace",
		label: "工作空间",
		items: [
			{
				kind: "link",
				key: "dashboard",
				to: appPaths.dashboard,
				icon: Category,
			},
			{
				kind: "group",
				key: "achievements",
				icon: FileText,
				children: [
					{
						kind: "link",
						key: "achievements-mine",
						to: navPaths.achievementsMine,
					},
					{
						kind: "link",
						key: "achievements-all",
						to: navPaths.achievementsAll,
					},
					{
						kind: "link",
						key: "achievements-new",
						to: navPaths.achievementsNew,
						permission: "achievements:write",
					},
				],
			},
			{
				kind: "link",
				key: "reviews",
				to: "/reviews",
				icon: Check,
				permission: "reviews:write",
			},
			{ kind: "link", key: "files", to: "/files", icon: Folder },
		],
	},
	{
		id: "admin",
		label: "系统管理",
		permission: "admin:manage",
		items: [
			{
				kind: "link",
				key: "admin-accounts",
				to: "/admin/accounts",
				icon: Users,
			},
			{ kind: "link", key: "admin-login", to: "/admin/login", icon: Settings },
			{
				kind: "link",
				key: "admin-login-page",
				to: "/admin/login-page",
				icon: Layout,
			},
		],
	},
];

export function visibleSections(
	sections: NavSection[],
	can: (permission: Permission) => boolean,
): NavSection[] {
	const allowed = (item: { permission?: Permission }) =>
		!item.permission || can(item.permission);
	return sections.filter(allowed).flatMap((section): NavSection[] => {
		const items = section.items
			.filter(allowed)
			.flatMap((item): NavSection["items"] => {
				if (item.kind === "link") return [item];
				const children = item.children.filter(allowed);
				return children.length ? [{ ...item, children }] : [];
			});
		return items.length ? [{ ...section, items }] : [];
	});
}
