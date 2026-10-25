import {
	LuCalendarDays,
	LuChartColumn,
	LuCheck,
	LuFileText,
	LuFolder,
	LuLayoutDashboard,
	LuListFilter,
	LuPlus,
	LuUserRound,
	LuUsers,
} from "react-icons/lu";
import { NavLink } from "react-router-dom";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export function Navigation({ onNavigate }: { onNavigate: () => void }) {
	const { session } = useAuth();
	const items = [
		{ label: "工作台", icon: LuLayoutDashboard, to: appPaths.dashboard },
		{ label: "我的成果", icon: LuFileText, to: "/achievements" },
		{ label: "成果填报", icon: LuPlus, to: "/achievements/new" },
		...(session?.user.grants.some(
			(grant) => grant.permission === "reviews:write",
		)
			? [{ label: "审核中心", icon: LuCheck }]
			: []),
		{ label: "个人档案", icon: LuUserRound, to: appPaths.profile },
		{ label: "统计报表", icon: LuChartColumn },
		{ label: "文件资料", icon: LuFolder, to: "/files" },
	];
	return (
		<nav className="navigation">
			<p className="nav-group-label">个人工作空间</p>
			{items.map((item) =>
				item.to ? (
					<NavLink
						key={item.label}
						to={item.to}
						end={item.to === "/achievements"}
						onClick={onNavigate}
						className={({ isActive }) => `nav-link ${isActive ? "active" : ""}`}
					>
						<item.icon aria-hidden="true" />
						<span>{item.label}</span>
					</NavLink>
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
					{[
						{ label: "成员与角色", icon: LuUsers },
						{ label: "成果类型", icon: LuListFilter },
						{ label: "学年学期", icon: LuCalendarDays },
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
