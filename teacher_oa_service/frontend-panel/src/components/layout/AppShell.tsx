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
import { ConfirmProvider } from "@/components/common/ConfirmDialog";
import { IconButton } from "@/components/common/IconButton";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { resolveNavigation, resolveTrailLinks } from "@/routes/navigationState";
import { appPaths } from "@/routes/routePaths";
import { formatAppDate, formatAppLongDate } from "@/services/dateFormat";
import { notifyInfo } from "@/services/notify";
import {
	readLastAchievementListPath,
	rememberListPath,
} from "@/services/returnPath";
import { useMediaQuery } from "@/services/useMediaQuery";
import { useOpenUntilNavigation } from "@/services/useOpenUntilNavigation";
import { LeaveGuardContext } from "@/services/useUnsavedChanges";
import { Navigation } from "./Navigation";
import { navPaths } from "./navigation.config";
import { SidebarAccount } from "./SidebarAccount";

export const mobileDrawerQuery = "(max-width: 720px)";

function AppShellInner() {
	const { session, logout } = useAuth();
	const [error, setError] = useState("");
	const [leaving, setLeaving] = useState(false);
	const location = useLocation();
	const currentLocationPath = location.pathname + location.search;
	const nav = resolveNavigation(
		location.pathname,
		location.search,
		location.state?.from,
	);
	const trailLinks = resolveTrailLinks(
		location.pathname,
		location.search,
		location.state?.from,
	);
	useEffect(() => {
		document.title = `${nav.title} · TeacherOA`;
	}, [nav.title]);
	useEffect(() => {
		rememberListPath(location.pathname, location.search);
	}, [location.pathname, location.search]);
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
					'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]',
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
	const leaveGuard = useRef<(() => Promise<boolean>) | null>(null);
	const guards = useMemo(
		() => ({
			register: (guard: () => Promise<boolean>) => {
				leaveGuard.current = guard;
				return () => {
					if (leaveGuard.current === guard) leaveGuard.current = null;
				};
			},
		}),
		[],
	);
	const leave = async () => {
		if (leaving) return;
		if (leaveGuard.current && !(await leaveGuard.current())) return;
		setLeaving(true);
		setError("");
		try {
			await logout();
			notifyInfo("已退出登录");
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
	const composeFromState = {
		from:
			location.pathname === "/achievements" || location.pathname === "/reviews"
				? currentLocationPath
				: readLastAchievementListPath("/achievements"),
	};
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
							to={mustChangePassword ? currentLocationPath : appPaths.dashboard}
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
										state={composeFromState}
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
							{trailLinks.map((part, index) =>
								index === trailLinks.length - 1 ? (
									<strong key={part.label}>{part.label}</strong>
								) : (
									<Fragment key={part.label}>
										{part.to && !mustChangePassword ? (
											<Link to={part.to} state={{ from: part.to }}>
												{part.label}
											</Link>
										) : (
											<span>{part.label}</span>
										)}
										<span aria-hidden="true">/</span>
									</Fragment>
								),
							)}
						</div>
						<time dateTime={formatAppDate(now)}>{formatAppLongDate(now)}</time>
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

export function AppShell() {
	return (
		<ConfirmProvider>
			<AppShellInner />
		</ConfirmProvider>
	);
}
