import { lazy, Suspense } from "react";
import { Link } from "react-router-dom";
import {
	AlertTriangle,
	ArrowUpRight,
	Clock,
	FileText,
	Refresh,
	X,
} from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { DashboardSkeleton } from "@/components/common/Skeleton";
import {
	achievementApi,
	errorMessage,
	statusLabels,
} from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { readLastAchievementListPath } from "@/services/returnPath";
import { useResource } from "@/services/useResource";
import { useUiPreferencesStore } from "@/stores/uiPreferencesStore";

const DashboardStatusChart = lazy(() => import("./DashboardStatusChart"));

export const roleLabels: Record<string, string> = {
	teacher: "教师",
	group_leader: "教研组长",
	reviewer: "审核人员",
	school_leader: "校级行政",
	admin: "管理员",
};
export const scopeLabels: Record<string, string> = {
	SELF: "本人",
	DEPARTMENT: "所在部门",
	ALL: "全校",
};

const statusOrder = [
	"draft",
	"pending",
	"returned",
	"approved",
	"archived",
] as const;

export function DashboardPage() {
	const { session } = useAuth();
	const result = useResource(achievementApi.dashboard);
	const onboardingDismissed = useUiPreferencesStore(
		(state) => state.onboardingDismissed,
	);
	const dismissOnboarding = useUiPreferencesStore(
		(state) => state.dismissOnboarding,
	);
	const canWrite = hasPermission(session?.user, "achievements:write");
	const canReview = Boolean(
		session?.user.grants.some((g) => g.permission === "reviews:write"),
	);
	const canManage = Boolean(
		session?.user.grants.some((g) => g.permission === "admin:manage"),
	);
	const readOnlyLeader = !canWrite && !canReview && !canManage;
	const hasRecent = Boolean(result.data?.recent.length);
	const composeState = { from: readLastAchievementListPath("/achievements") };

	const counts = result.data?.counts ?? {};
	const totalCount = statusOrder.reduce(
		(sum, key) => sum + (counts[key] ?? 0),
		0,
	);
	const returnedCount = counts.returned ?? 0;
	const draftCount = counts.draft ?? 0;
	const reviewPending = result.data?.review_pending ?? 0;
	const hasTodos =
		(canWrite && (returnedCount > 0 || draftCount > 0)) ||
		(canReview && reviewPending > 0);
	const chartData = statusOrder.map((status) => ({
		status,
		name: statusLabels[status] ?? status,
		count: counts[status] ?? 0,
	}));
	const canRenderChart =
		totalCount > 0 &&
		typeof window !== "undefined" &&
		"ResizeObserver" in window;

	return (
		<>
			<PageHeading
				title="工作台"
				eyebrow={`你好，${session?.user.display_name ?? ""}`}
			>
				{canWrite && !result.loading && hasRecent && (
					<Link
						className="action-button action-button-primary"
						to="/achievements/new"
						state={composeState}
					>
						新建成果
					</Link>
				)}
				<IconButton
					label="刷新工作台"
					disabled={result.loading || result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
			</PageHeading>
			{result.loading && !result.data ? (
				<DashboardSkeleton message="正在读取工作台…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<>
					{readOnlyLeader && (
						<p className="role-notice" role="status">
							当前账号为成果查阅权限（可前往「成果查询」浏览本部门或授权范围内的教师成果）。如需填报个人成果，请联系学校管理员配置权限。
						</p>
					)}
					{canWrite && !onboardingDismissed && totalCount === 0 && (
						<section
							className="panel onboarding-card"
							aria-labelledby="onboarding-title"
						>
							<div className="section-heading">
								<h2 id="onboarding-title">第一次使用？三步完成成果申报</h2>
								<IconButton label="关闭新手引导" onClick={dismissOnboarding}>
									<X />
								</IconButton>
							</div>
							{/* biome-ignore lint/a11y/noRedundantRoles: Safari removes list semantics when list-style: none is applied. */}
							<ol className="onboarding-steps" role="list">
								<li>
									<strong>1. 新建并填写信息</strong>
									<span>
										选择公开课、课题或获奖等类型，填写成果基本资料与参与人。
									</span>
								</li>
								<li>
									<strong>2. 上传证明材料</strong>
									<span>支持拖拽或手机拍照上传 PDF、JPG、PNG 证明文件。</span>
								</li>
								<li>
									<strong>3. 提交审核并跟进</strong>
									<span>
										提交后在工作台查看进度；若需补正可一键修改并重新提交。
									</span>
								</li>
							</ol>
						</section>
					)}
					{hasTodos && (
						<section
							className="panel todo-panel"
							aria-labelledby="todo-heading"
						>
							<div className="section-heading">
								<h2 id="todo-heading">需要您处理</h2>
							</div>
							{/* biome-ignore lint/a11y/noRedundantRoles: Safari removes list semantics when list-style: none is applied. */}
							<ul className="todo-list" role="list">
								{canWrite && returnedCount > 0 && (
									<li>
										<Link
											className="todo-item todo-item-urgent"
											to="/achievements?scope=mine&status=returned"
										>
											<span className="todo-item-label">
												<AlertTriangle aria-hidden="true" />有 {returnedCount}{" "}
												项成果被退回待补正，请修改后重新提交
											</span>
											<ArrowUpRight aria-hidden="true" />
										</Link>
									</li>
								)}
								{canReview && reviewPending > 0 && (
									<li>
										<Link
											className="todo-item todo-item-warning"
											to="/reviews?status=pending"
										>
											<span className="todo-item-label">
												<Clock aria-hidden="true" />
												待审核成果 {reviewPending} 项，点击进入审核中心办理
											</span>
											<ArrowUpRight aria-hidden="true" />
										</Link>
									</li>
								)}
								{canWrite && draftCount > 0 && (
									<li>
										<Link
											className="todo-item"
											to="/achievements?scope=mine&status=draft"
										>
											<span className="todo-item-label">
												<FileText aria-hidden="true" />有 {draftCount}{" "}
												项草稿尚未提交审核
											</span>
											<ArrowUpRight aria-hidden="true" />
										</Link>
									</li>
								)}
							</ul>
						</section>
					)}
					<div className="dashboard-stats">
						<Link className="summary-item" to="/achievements?scope=mine">
							<span>我的成果</span>
							<strong>{totalCount}</strong>
						</Link>
						{statusOrder.map((status) => (
							<Link
								key={status}
								className={`summary-item metric-${status}`}
								to={`/achievements?scope=mine&status=${status}`}
							>
								<span>{statusLabels[status]}</span>
								<strong>{counts[status] ?? 0}</strong>
							</Link>
						))}
					</div>
					{canReview && reviewPending === 0 && (
						<Link className="review-inbox action-row" to="/reviews">
							<span>待审核成果</span>
							<strong>0</strong>
							<ArrowUpRight />
						</Link>
					)}
					<div className={canRenderChart ? "dashboard-main-grid" : undefined}>
						{canRenderChart && (
							<Suspense fallback={null}>
								<DashboardStatusChart
									data={chartData}
									totalCount={totalCount}
								/>
							</Suspense>
						)}
						<section className="panel">
							<div className="section-heading">
								<h2>最近更新</h2>
								{hasRecent && (
									<Link
										className="text-link"
										to="/achievements?scope=mine"
										aria-label="查看全部成果"
									>
										查看全部
									</Link>
								)}
							</div>
							{hasRecent ? (
								<AchievementTable
									items={result.data.recent}
									hideTeacher
									contextTeacherId={session?.user.teacher?.id}
								/>
							) : (
								<EmptyState
									icon={<FileText />}
									title="暂无成果记录"
									description="填报您的教学或科研成果并关联证明材料，即可提交审核。"
									action={
										canWrite ? (
											<Link
												className="action-button action-button-primary"
												to="/achievements/new"
												state={composeState}
											>
												新建成果
											</Link>
										) : undefined
									}
								/>
							)}
						</section>
					</div>
				</>
			)}
		</>
	);
}
