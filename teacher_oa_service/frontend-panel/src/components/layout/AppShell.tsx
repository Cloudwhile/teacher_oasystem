import {
	Fragment,
	useCallback,
	useEffect,
	useMemo,
	useRef,
	useState,
} from "react";
import { Link, Outlet, useLocation } from "react-router-dom";
import { GraduationCap, Menu, Plus, X } from "reicon-react";
import { IconButton } from "@/components/common/IconButton";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { resolveNavigation } from "@/routes/navigationState";
import { appPaths } from "@/routes/routePaths";
import { formatSingaporeDate } from "@/services/dateFormat";
import { useMediaQuery } from "@/services/useMediaQuery";
import { useOpenUntilNavigation } from "@/services/useOpenUntilNavigation";
import { LeaveGuardContext } from "@/services/useUnsavedChanges";
import { Navigation } from "./Navigation";
import { navPaths } from "./navigation.config";
import { SidebarAccount } from "./SidebarAccount";

export const mobileDrawerQuery = "(max-width: 720px)";

export function AppShell() {
	const { session, logout } = useAuth();
	const [error, setError] = useState("");
	const [leaving, setLeaving] = useState(false);
	const location = useLocation();
	const nav = resolveNavigation(
		location.pathname,
		location.search,
		location.state?.from,
	);
	useEffect(() => {
		document.title = `${nav.title} · TeacherOA`;
	}, [nav.title]);
	const mobile = useMediaQuery(mobileDrawerQuery);
	const [menuOpen, setMenuOpen] = useOpenUntilNavigation();
	if (menuOpen && !mobile) setMenuOpen(false);
	const drawerOpen = mobile && menuOpen;
	const toggleButton = useRef<HTMLButtonElement>(null);
	const closeButton = useRef<HTMLButtonElement>(null);
	const drawer = useRef<HTMLDivElement>(null);
	const main = useRef<HTMLElement>(null);
	const restoreToggleFocus = useRef(false);
	const wasOpen = useRef(false);
	const closeDrawer = useCallback(() => {
		restoreToggleFocus.current = true;
		setMenuOpen(false);
	}, [setMenuOpen]);
	useEffect(() => {
		if (drawerOpen) {
			wasOpen.current = true;
			closeButton.current?.focus();
			return;
		}
		if (!wasOpen.current) return;
		wasOpen.current = false;
		const toToggle = restoreToggleFocus.current;
		restoreToggleFocus.current = false;
		// Closed by the user: back to the trigger. Closed because a navigation
		// committed: into the new page instead of the now-hidden menu.
		if (toToggle) toggleButton.current?.focus();
		else if (mobile) main.current?.focus({ preventScroll: true });
	}, [drawerOpen, mobile]);
	useEffect(() => {
		if (!drawerOpen) return;
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === "Escape") {
				closeDrawer();
				return;
			}
			if (event.key !== "Tab" || !drawer.current) return;
			const controls = Array.from(
				drawer.current.querySelectorAll<HTMLElement>(
					'a[href], button:not(:disabled), [tabindex="0"]',
				),
			).filter((element) => element.getClientRects().length > 0);
			const first = controls.at(0);
			const last = controls.at(-1);
			if (event.shiftKey && document.activeElement === first) {
				event.preventDefault();
				last?.focus();
			} else if (!event.shiftKey && document.activeElement === last) {
				event.preventDefault();
				first?.focus();
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [drawerOpen, closeDrawer]);
	const leaveGuard = useRef<(() => boolean) | null>(null);
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
	const mustChangePassword = Boolean(session?.must_change_password);
	const canWriteAchievement = hasPermission(
		session?.user,
		"achievements:write",
	);
	const now = new Date();
	return (
		<LeaveGuardContext.Provider value={guards}>
			<div className={`app-shell${drawerOpen ? " drawer-open" : ""}`}>
				<a className="skip-link" href="#main-content">
					跳至主要内容
				</a>
				{/* Pointer shortcut only; the close button and Escape serve keyboard users. */}
				<div
					className="drawer-backdrop"
					aria-hidden="true"
					onClick={closeDrawer}
				/>
				{/* Modal only while it covers the page as a drawer; a plain sidebar otherwise. */}
				<div
					ref={drawer}
					id="main-navigation"
					className="sidebar"
					{...(drawerOpen && {
						role: "dialog",
						"aria-modal": true,
						"aria-label": "导航菜单",
					})}
				>
					<div className="sidebar-header">
						<Link
							to={
								mustChangePassword
									? location.pathname + location.search
									: appPaths.dashboard
							}
							className="brand"
						>
							<GraduationCap aria-hidden="true" />
							<span>
								TeacherOA<small>教师综合 OA</small>
							</span>
						</Link>
						<IconButton
							ref={closeButton}
							className="mobile-nav-close"
							label="关闭导航"
							onClick={closeDrawer}
						>
							<X />
						</IconButton>
					</div>
					<nav className="sidebar-nav" aria-label="主导航">
						{mustChangePassword ? (
							<p className="nav-group-label">请先修改初始密码后使用其他功能</p>
						) : (
							<>
								{canWriteAchievement && (
									<Link
										to={navPaths.achievementsNew}
										className="sidebar-create"
									>
										<Plus aria-hidden="true" />
										<span>新建成果</span>
									</Link>
								)}
								<Navigation state={nav} />
							</>
						)}
					</nav>
					<SidebarAccount
						user={session?.user}
						activeKey={nav.activeKey}
						leaving={leaving}
						error={error}
						mustChangePassword={mustChangePassword}
						onLogout={() => {
							void leave();
						}}
					/>
				</div>
				<div className="workspace" inert={drawerOpen}>
					<header className="topbar">
						<div className="breadcrumb">
							<IconButton
								ref={toggleButton}
								className="menu-toggle"
								label="展开导航"
								aria-controls="main-navigation"
								aria-expanded={drawerOpen}
								onClick={() => setMenuOpen(true)}
							>
								<Menu />
							</IconButton>
							{nav.trail.map((part, index) =>
								index === nav.trail.length - 1 ? (
									<strong key={part}>{part}</strong>
								) : (
									<Fragment key={part}>
										<span>{part}</span>
										<span aria-hidden="true">/</span>
									</Fragment>
								),
							)}
						</div>
						<time dateTime={formatSingaporeDate(now)}>
							{new Intl.DateTimeFormat("zh-CN", {
								year: "numeric",
								month: "long",
								day: "numeric",
								timeZone: "Asia/Singapore",
							}).format(now)}
						</time>
					</header>
					<main
						ref={main}
						id="main-content"
						className="page-content"
						tabIndex={-1}
					>
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
