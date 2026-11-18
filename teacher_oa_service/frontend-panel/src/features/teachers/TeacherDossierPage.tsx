import { useCallback } from "react";
import { Link, useLocation, useParams } from "react-router-dom";
import { ArrowLeft, FileText, Folder, Refresh } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { MaterialList } from "@/components/achievements/MaterialList";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { PermissionDenied } from "@/components/common/PermissionDenied";
import { ProfileFields } from "@/components/common/ProfileFields";
import { DetailSkeleton } from "@/components/common/Skeleton";
import { errorMessage, statusLabels } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { roleLabels } from "@/features/dashboard/DashboardPage";
import { formatAppDate } from "@/services/dateFormat";
import { HttpError } from "@/services/httpClient";
import { safeReturnPath } from "@/services/returnPath";
import { useResource } from "@/services/useResource";
import { teacherDirectoryApi } from "./api";

const dossierStatuses = [
	"pending",
	"returned",
	"approved",
	"archived",
] as const;

export function TeacherDossierPage() {
	const { id } = useParams();
	const location = useLocation();
	const back = safeReturnPath(location.state?.from, "/teachers");
	const { session } = useAuth();
	const canAccess = hasPermission(session?.user, "teachers:directory");
	const teacherId = Number(id);

	const load = useCallback(
		(signal: AbortSignal) =>
			canAccess && Number.isFinite(teacherId) && teacherId > 0
				? teacherDirectoryApi.dossier(teacherId, signal)
				: Promise.reject(new Error("无效的教师编号")),
		[canAccess, teacherId],
	);
	const result = useResource(load);

	if (!canAccess) {
		return (
			<PermissionDenied
				title="教师档案详情"
				message="仅系统管理员与校级行政可查阅全校教师资料"
			/>
		);
	}

	if (result.loading && !result.data) {
		return <DetailSkeleton message="正在读取教师资料…" />;
	}

	if (result.error || !result.data) {
		const notFound =
			result.error instanceof HttpError && result.error.status === 404;
		return (
			<>
				<PageHeading title="教师档案详情">
					<Link
						to={back}
						className="icon-button"
						aria-label="返回教师名录"
						title="返回教师名录"
					>
						<ArrowLeft />
					</Link>
				</PageHeading>
				<Feedback
					message={
						notFound
							? "未找到该教师档案记录。"
							: errorMessage(result.error, "教师资料暂时无法读取。")
					}
					onRetry={notFound ? undefined : result.reload}
				/>
			</>
		);
	}

	const {
		profile,
		account_status,
		roles,
		created_at,
		counts,
		achievements,
		achievement_total,
		files,
	} = result.data;
	const displayRoles = roles.filter((r) => r !== "teacher");
	const roleText = (displayRoles.length ? displayRoles : ["teacher"])
		.map((role) => roleLabels[role] ?? role)
		.join(" / ");

	return (
		<>
			<PageHeading
				title={profile.name}
				eyebrow={`${profile.department_name} · ${profile.employee_no}`}
			>
				<Link
					to={back}
					className="icon-button"
					aria-label="返回教师名录"
					title="返回教师名录"
				>
					<ArrowLeft />
				</Link>
				<IconButton
					label="刷新教师资料"
					disabled={result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
				<Link
					to={`/achievements?scope=all&teacher_id=${profile.id}`}
					className="action-button action-button-secondary"
				>
					在成果查询中筛选
				</Link>
			</PageHeading>

			<div className="dashboard-stats">
				<Link
					className="summary-item"
					to={`/achievements?scope=all&teacher_id=${profile.id}`}
				>
					<span>已报成果</span>
					<strong>{achievement_total}</strong>
				</Link>
				{dossierStatuses.map((status) => (
					<Link
						key={status}
						className={`summary-item metric-${status}`}
						to={`/achievements?scope=all&teacher_id=${profile.id}&status=${status}`}
					>
						<span>{statusLabels[status]}</span>
						<strong>{counts[status] ?? 0}</strong>
					</Link>
				))}
				<div className="summary-item">
					<span>证明材料</span>
					<strong>{files.length}</strong>
				</div>
			</div>

			<div className="editor-sections">
				<section className="panel">
					<div className="section-heading">
						<h2>基础资料</h2>
						<span className="section-meta">
							{roleText} ·{" "}
							{account_status === "active" ? "账号启用" : "账号已停用"} · 建档于{" "}
							{formatAppDate(created_at)}
						</span>
					</div>
					<ProfileFields profile={profile} />
				</section>

				<section className="panel">
					<div className="section-heading">
						<h2>已报成果记录</h2>
						<span className="section-meta">
							{achievement_total > achievements.length
								? `显示最近 ${achievements.length} / 共 ${achievement_total} 条`
								: `共 ${achievement_total} 条`}
						</span>
					</div>
					{achievements.length ? (
						<AchievementTable items={achievements} hideTeacher />
					) : (
						<EmptyState
							icon={<FileText />}
							title="暂无已提交的成果记录。"
							description="该教师尚未提交公开课、科研或荣誉等成果记录（未提交的个人草稿不对外展示）。"
						/>
					)}
				</section>

				<section className="panel">
					<div className="section-heading">
						<h2>关联证明材料</h2>
						<span className="section-meta">共 {files.length} 份</span>
					</div>
					{files.length ? (
						<MaterialList files={files} />
					) : (
						<EmptyState
							icon={<Folder />}
							title="暂无关联证明材料。"
							description="该教师已提交成果所附带的 PDF 或图片证明材料将汇总展示于此。"
						/>
					)}
				</section>
			</div>
		</>
	);
}
