import { useCallback, useEffect, useState } from "react";
import { Refresh } from "reicon-react";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { ProfileFields } from "@/components/common/ProfileFields";
import { useAuth } from "@/features/auth/AuthProvider";
import { authApi, type TeacherProfile } from "@/features/auth/api";
import { roleLabels } from "@/features/dashboard/DashboardPage";
import { HttpError } from "@/services/httpClient";

export function MyProfilePage() {
	const { session, refresh } = useAuth();
	const [profile, setProfile] = useState<TeacherProfile | null>(null);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState("");
	const load = useCallback(
		async (signal?: AbortSignal) => {
			setLoading(true);
			setError("");
			try {
				const value = await authApi.profile(signal);
				if (!signal?.aborted) setProfile(value);
			} catch (error) {
				if (!signal?.aborted) {
					if (error instanceof HttpError && [401, 403].includes(error.status)) {
						await refresh();
					} else setError("档案暂时无法读取，请稍后重试。");
				}
			} finally {
				if (!signal?.aborted) setLoading(false);
			}
		},
		[refresh],
	);
	useEffect(() => {
		const controller = new AbortController();
		void load(controller.signal);
		return () => controller.abort();
	}, [load]);
	return (
		<>
			<PageHeading title="个人档案" eyebrow="我的资料">
				<IconButton
					label="刷新档案"
					disabled={loading}
					onClick={() => {
						void load();
					}}
				>
					<Refresh />
				</IconButton>
			</PageHeading>
			{loading ? (
				<Feedback message="正在读取档案…" loading />
			) : error ? (
				<Feedback
					message={error}
					onRetry={() => {
						void load();
					}}
				/>
			) : profile ? (
				<section className="panel">
					<div className="section-heading">
						<h2>基本资料</h2>
						<span className="section-meta">
							{session?.user.roles
								.map((role) => roleLabels[role] ?? role)
								.join(" / ")}
						</span>
					</div>
					<ProfileFields profile={profile} />
				</section>
			) : (
				<p className="empty-message">未找到教师档案。</p>
			)}
		</>
	);
}
