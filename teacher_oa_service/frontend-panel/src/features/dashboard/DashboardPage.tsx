import {
	LuArrowUpRight,
	LuBadgeCheck,
	LuBookOpen,
	LuBuilding2,
	LuPlus,
	LuRefreshCw,
	LuUserRound,
} from "react-icons/lu";
import { Link } from "react-router-dom";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { subjectLabels } from "@/components/common/ProfileFields";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export const roleLabels: Record<string, string> = {
	teacher: "教师",
	group_leader: "教研组长",
	reviewer: "审核人员",
	admin: "管理员",
};
export const scopeLabels: Record<string, string> = {
	SELF: "本人",
	DEPARTMENT: "所在部门",
	ALL: "全校",
};
const permissionLabels: Record<string, string> = {
	"teachers:read": "教师档案",
	"achievements:write": "成果填报",
	"reviews:write": "成果审核",
	"admin:manage": "系统管理",
	"achievements:read": "成果查看",
	"files:read": "材料查看",
	"files:write": "材料上传",
};

export function DashboardPage() {
	const { session, refresh } = useAuth();
	if (!session) return null;
	const { user } = session;
	return (
		<>
			<PageHeading title={`你好，${user.display_name}`} eyebrow="我的工作台">
				<Link
					to="/achievements/new"
					className="icon-button primary-action"
					aria-label="新建成果"
					title="新建成果"
				>
					<LuPlus />
				</Link>
				<IconButton
					label="刷新工作台"
					onClick={() => {
						void refresh();
					}}
				>
					<LuRefreshCw />
				</IconButton>
			</PageHeading>
			<div className="identity-summary">
				<div className="summary-item">
					<LuUserRound />
					<span>教师工号</span>
					<strong>{user.teacher.employee_no}</strong>
				</div>
				<div className="summary-item">
					<LuBuilding2 />
					<span>所属部门</span>
					<strong>{user.teacher.department_name}</strong>
				</div>
				<div className="summary-item">
					<LuBookOpen />
					<span>任教学科</span>
					<strong>
						{subjectLabels[user.teacher.subject_code] ??
							user.teacher.subject_code}
					</strong>
				</div>
				<div className="summary-item">
					<LuBadgeCheck />
					<span>在职状态</span>
					<strong className="success-text">在职</strong>
				</div>
			</div>
			<div className="workspace-grid">
				<section className="panel" aria-labelledby="access-title">
					<div className="section-heading">
						<h2 id="access-title">角色与访问范围</h2>
						<span className="section-meta">
							{user.roles.map((role) => roleLabels[role] ?? role).join(" / ")}
						</span>
					</div>
					<div className="table-scroll">
						<table>
							<thead>
								<tr>
									<th scope="col">事项</th>
									<th scope="col">可访问范围</th>
								</tr>
							</thead>
							<tbody>
								{user.grants.map((grant) => (
									<tr key={`${grant.permission}-${grant.scope}`}>
										<td>
											{permissionLabels[grant.permission] ?? grant.permission}
										</td>
										<td>
											<span
												className={`scope-label scope-${grant.scope.toLowerCase()}`}
											>
												{scopeLabels[grant.scope] ?? grant.scope}
											</span>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				</section>
				<section className="panel" aria-labelledby="profile-title">
					<div className="section-heading">
						<h2 id="profile-title">个人档案</h2>
						<LuUserRound aria-hidden="true" />
					</div>
					<div className="profile-overview">
						<span className="profile-avatar">
							{user.display_name.slice(0, 1)}
						</span>
						<h3>{user.teacher.name}</h3>
						<p>{user.teacher.department_name}</p>
						<p className="muted">{user.teacher.employee_no}</p>
					</div>
					<Link className="action-row" to={appPaths.profile}>
						<span>查看档案</span>
						<LuArrowUpRight aria-hidden="true" />
					</Link>
				</section>
			</div>
		</>
	);
}
