import { Link, NavLink } from "react-router-dom";
import {
	CalendarDays,
	Category,
	ChartBar,
	Check,
	FileText,
	Filter,
	Folder,
	Key,
	Layout,
	Plus,
	Settings,
	User,
	Users,
} from "reicon-react";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export function Navigation({
	onNavigate,
	activeSection,
}: {
	onNavigate: () => void;
	activeSection: string;
}) {
	const { session } = useAuth();
	const items = [
		{
			label: "工作台",
			icon: Category,
			to: appPaths.dashboard,
			section: "dashboard",
		},
		{
			label: "我的成果",
			icon: FileText,
			to: "/achievements",
			section: "achievements",
		},
		{
			label: "成果填报",
			icon: Plus,
			to: "/achievements/new",
			section: "create",
		},
		...(session?.user.grants.some(
			(grant) => grant.permission === "reviews:write",
		)
			? [
					{
						label: "审核中心",
						icon: Check,
						to: "/reviews",
						section: "reviews",
					},
				]
			: []),
		{
			label: "个人档案",
			icon: User,
			to: appPaths.profile,
			section: "profile",
		},
		{ label: "账号安全", icon: Key, to: "/account", section: "account" },
		{ label: "统计报表", icon: ChartBar },
		{ label: "文件资料", icon: Folder, to: "/files", section: "files" },
	];
	return (
		<nav className="navigation">
			<p className="nav-group-label">个人工作空间</p>
			{items.map((item) =>
				item.to ? (
					<Link
						key={item.label}
						to={item.to}
						onClick={onNavigate}
						aria-current={activeSection === item.section ? "page" : undefined}
						className={`nav-link ${activeSection === item.section ? "active" : ""}`}
					>
						<item.icon aria-hidden="true" />
						<span>{item.label}</span>
					</Link>
				) : (
					<span key={item.label} className="nav-link disabled" title="尚未开放">
						<item.icon aria-hidden="true" />
						<span>{item.label}</span>
					</span>
				),
			)}
			{session?.user.grants.some(
				(grant) => grant.permission === "admin:manage",
			) && (
				<>
					<p className="nav-group-label">系统管理</p>
					<NavLink
						className={({ isActive }) => `nav-link ${isActive ? "active" : ""}`}
						to="/admin/accounts"
						onClick={onNavigate}
					>
						<Users />
						<span>成员与账号</span>
					</NavLink>
					<NavLink
						className={({ isActive }) => `nav-link ${isActive ? "active" : ""}`}
						to="/admin/login"
						onClick={onNavigate}
					>
						<Settings />
						<span>登录设置</span>
					</NavLink>
					<NavLink
						className={({ isActive }) => `nav-link ${isActive ? "active" : ""}`}
						to="/admin/login-page"
						onClick={onNavigate}
					>
						<Layout aria-hidden="true" />
						<span>登录页与公告</span>
					</NavLink>
					{[
						{ label: "成果类型", icon: Filter },
						{ label: "学年学期", icon: CalendarDays },
					].map((item) => (
						<span
							className="nav-link disabled"
							key={item.label}
							title="尚未开放"
						>
							<item.icon aria-hidden="true" />
							<span>{item.label}</span>
						</span>
					))}
				</>
			)}
		</nav>
	);
}
