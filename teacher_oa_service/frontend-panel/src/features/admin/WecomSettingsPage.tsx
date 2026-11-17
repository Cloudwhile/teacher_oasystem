import { WecomSettingsForm } from "@/components/admin/WecomSettingsForm";
import { Feedback } from "@/components/common/Feedback";
import { PageHeading } from "@/components/common/PageHeading";
import { PermissionDenied } from "@/components/common/PermissionDenied";
import { FormSkeleton } from "@/components/common/Skeleton";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { useResource } from "@/services/useResource";
import { adminApi } from "./api";

export function WecomSettingsPage() {
	const { session } = useAuth();
	const result = useResource(adminApi.settings);
	if (!session?.user.grants.some((g) => g.permission === "admin:manage")) {
		return <PermissionDenied title="登录设置" message="无权访问登录设置" />;
	}
	return (
		<>
			<PageHeading title="登录设置">
				<a
					className="action-button action-button-secondary"
					href="/login?preview=1"
					target="_blank"
					rel="noreferrer"
				>
					预览登录页
				</a>
			</PageHeading>
			{result.loading && !result.data ? (
				<FormSkeleton message="正在读取设置…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<WecomSettingsForm
					key={result.data.version}
					initial={result.data}
					csrf={session.csrf_token}
					onReload={result.reload}
				/>
			)}
		</>
	);
}
