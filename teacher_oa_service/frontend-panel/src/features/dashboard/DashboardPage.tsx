import { Link } from "react-router-dom";
import { ArrowUpRight, Refresh } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import {
	achievementApi,
	errorMessage,
	statusLabels,
} from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { useResource } from "@/services/useResource";
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
export function DashboardPage() {
	const { session } = useAuth();
	const result = useResource(achievementApi.dashboard);
	return (
		<>
			<PageHeading
				title="工作台"
				eyebrow={`你好，${session?.user.display_name ?? ""}`}
			>
				<Link
					className="action-button action-button-primary"
					to="/achievements/new"
				>
					新建成果
				</Link>
				<IconButton label="刷新工作台" onClick={result.reload}>
					<Refresh />
				</IconButton>
			</PageHeading>
			{result.loading ? (
				<Feedback loading message="正在读取工作台…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<>
					<div className="dashboard-stats">
						<Link className="summary-item" to="/achievements?scope=mine">
							<span>我的成果</span>
							<strong>
								{Object.values(result.data.counts).reduce((a, b) => a + b, 0)}
							</strong>
						</Link>
						{Object.entries(result.data.counts).map(([status, count]) => (
							<Link
								key={status}
								className={`summary-item metric-${status}`}
								to={`/achievements?scope=mine&status=${status}`}
							>
								<span>{statusLabels[status]}</span>
								<strong>{count}</strong>
							</Link>
						))}
					</div>
					{session?.user.grants.some(
						(g) => g.permission === "reviews:write",
					) && (
						<Link className="review-inbox action-row" to="/reviews">
							<span>待审核成果</span>
							<strong>{result.data.review_pending}</strong>
							<ArrowUpRight />
						</Link>
					)}
					<section className="panel">
						<div className="section-heading">
							<h2>最近更新</h2>
							<Link
								className="icon-button"
								to="/achievements?scope=mine"
								aria-label="查看全部成果"
							>
								<ArrowUpRight />
							</Link>
						</div>
						{result.data.recent.length ? (
							<AchievementTable items={result.data.recent} />
						) : (
							<p className="empty-message">暂无成果记录</p>
						)}
					</section>
				</>
			)}
		</>
	);
}
