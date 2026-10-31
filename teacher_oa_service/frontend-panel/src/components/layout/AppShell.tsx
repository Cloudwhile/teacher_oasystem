import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link, Outlet, useLocation } from "react-router-dom";
import { GraduationCap, Logout, Menu, X } from "reicon-react";
import { IconButton } from "@/components/common/IconButton";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";
import { workspaceContext } from "@/routes/workspaceContext";
import { LeaveGuardContext } from "@/services/useUnsavedChanges";
import { Navigation } from "./Navigation";

export function AppShell() {
	const { session, logout } = useAuth();
	const [menuOpen, setMenuOpen] = useState(false);
	const [error, setError] = useState("");
	const [leaving, setLeaving] = useState(false);
	const location = useLocation();
	const context = workspaceContext(
		location.pathname,
		location.search,
		location.state?.from,
	);
	const leaveGuard = useRef<(() => boolean) | null>(null);
	const navigation = useRef<HTMLElement>(null);
	const closeMenu = useCallback(() => {
		setMenuOpen(false);
		document.getElementById("navigation-toggle")?.focus();
	}, []);
	useEffect(() => {
		if (!menuOpen) return;
		navigation.current?.querySelector<HTMLElement>("button")?.focus();
		const handleEscape = (event: KeyboardEvent) => {
			if (event.key === "Escape") closeMenu();
		};
		window.addEventListener("keydown", handleEscape);
		return () => window.removeEventListener("keydown", handleEscape);
	}, [menuOpen, closeMenu]);
	const guards = useMemo(
		() => ({
			register: (guard: () => boolean) => {
				leaveGuard.current = guard;
				return () => {
					if (leaveGuard.current === guard) leaveGuard.current = null;
				};
			},
		}),
		[],
	);
	const leave = async () => {
		if (leaving || (leaveGuard.current && !leaveGuard.current())) return;
		setLeaving(true);
		setError("");
		try {
			await logout();
		} catch {
			setError("退出未完成，请重试。");
		} finally {
			setLeaving(false);
		}
	};
	return (
		<LeaveGuardContext.Provider value={guards}>
			<div className="app-shell">
				<aside
					ref={navigation}
					id="main-navigation"
					className={`sidebar ${menuOpen ? "mobile-open" : ""}`}
					aria-label="主导航"
				>
					<IconButton
						className="mobile-nav-close"
						label="关闭导航"
						onClick={closeMenu}
					>
						<X />
					</IconButton>
					<Link to={appPaths.dashboard} className="brand">
						<GraduationCap aria-hidden="true" />
						<span>
							TeacherOA<small>教师综合 OA</small>
						</span>
					</Link>
					<Navigation onNavigate={closeMenu} activeSection={context.section} />
					<div className="sidebar-account">
						<span className="avatar">
							{session?.user.display_name.slice(0, 1)}
						</span>
						<div className="account-label">
							<strong>{session?.user.display_name}</strong>
							<small title={session?.user.teacher.department_name}>
								{session?.user.teacher.department_name}
							</small>
						</div>
						<IconButton
							label="退出登录"
							disabled={leaving}
							onClick={() => {
								void leave();
							}}
						>
							<Logout />
						</IconButton>
					</div>
					{error && (
						<p className="inline-error" role="alert">
							{error}
						</p>
					)}
				</aside>
				<div className="workspace">
					<header className="topbar">
						<div className="breadcrumb">
							<IconButton
								className="menu-toggle"
								id="navigation-toggle"
								label={menuOpen ? "收起导航" : "展开导航"}
								aria-controls="main-navigation"
								aria-expanded={menuOpen}
								onClick={() => setMenuOpen(!menuOpen)}
							>
								<Menu />
							</IconButton>
							<span>工作空间</span>
							<span aria-hidden="true">/</span>
							<strong>{context.title}</strong>
						</div>
						<time dateTime={new Date().toISOString().slice(0, 10)}>
							{new Intl.DateTimeFormat("zh-CN", {
								year: "numeric",
								month: "long",
								day: "numeric",
								timeZone: "Asia/Singapore",
							}).format(new Date())}
						</time>
					</header>
					<main id="main-content" className="page-content">
						<Outlet />
					</main>
					<footer className="workspace-footer">
						TeacherOA <span>教师工作空间</span>
					</footer>
				</div>
			</div>
		</LeaveGuardContext.Provider>
	);
}
