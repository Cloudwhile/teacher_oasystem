import { type KeyboardEvent, useId, useRef } from "react";
import { Link } from "react-router-dom";
import { ChevronDown, Key, Logout, User } from "reicon-react";
import { IconButton } from "@/components/common/IconButton";
import type { CurrentUser } from "@/features/auth/api";
import { type NavKey, navLabels } from "@/routes/navigationState";
import { appPaths } from "@/routes/routePaths";
import { useOpenUntilNavigation } from "@/services/useOpenUntilNavigation";

const accountLinks = [
	{ key: "profile", to: appPaths.profile, icon: User },
	{ key: "account", to: "/account", icon: Key },
] as const;

export function SidebarAccount({
	user,
	activeKey,
	leaving,
	error,
	mustChangePassword = false,
	onLogout,
}: {
	user: CurrentUser | undefined;
	activeKey: NavKey | null;
	leaving: boolean;
	error: string;
	mustChangePassword?: boolean;
	onLogout: () => void;
}) {
	const [open, setOpen] = useOpenUntilNavigation();
	const trigger = useRef<HTMLButtonElement>(null);
	const menuId = useId();
	const name = user?.display_name ?? "";
	const department = user?.teacher.department_name ?? "";
	const links = mustChangePassword
		? accountLinks.filter((link) => link.key === "account")
		: accountLinks;
	const current = links.some((link) => link.key === activeKey);
	const closeOnEscape = (event: KeyboardEvent) => {
		if (event.key !== "Escape" || !open) return;
		event.stopPropagation();
		setOpen(false);
		trigger.current?.focus();
	};
	return (
		<section className="sidebar-account" aria-label="个人账户">
			<div className="account-row">
				<button
					ref={trigger}
					type="button"
					className={`account-trigger${current ? " contains-current" : ""}`}
					aria-label={`账户菜单：${name}${department ? `，${department}` : ""}`}
					aria-expanded={open}
					aria-controls={menuId}
					onClick={() => setOpen(!open)}
					onKeyDown={closeOnEscape}
				>
					<span className="avatar" aria-hidden="true">
						{name.slice(0, 1)}
					</span>
					<span className="account-label">
						<strong title={name}>{name}</strong>
						<small title={department}>{department}</small>
					</span>
					<ChevronDown className="nav-chevron" aria-hidden="true" />
				</button>
				<IconButton
					className="account-logout"
					label="退出登录"
					disabled={leaving}
					onClick={onLogout}
				>
					<Logout />
				</IconButton>
			</div>
			<div id={menuId} className="account-menu" hidden={!open}>
				<ul>
					{links.map((link) => (
						<li key={link.key}>
							<Link
								to={link.to}
								className={`nav-link${activeKey === link.key ? " active" : ""}`}
								aria-current={activeKey === link.key ? "page" : undefined}
								onKeyDown={closeOnEscape}
							>
								<link.icon aria-hidden="true" />
								<span className="nav-text">{navLabels[link.key]}</span>
							</Link>
						</li>
					))}
				</ul>
			</div>
			{error && (
				<p className="inline-error account-error" role="alert">
					{error}
				</p>
			)}
		</section>
	);
}
