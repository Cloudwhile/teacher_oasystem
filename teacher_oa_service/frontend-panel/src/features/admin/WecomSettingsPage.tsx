import { WecomSettingsForm } from "@/components/admin/WecomSettingsForm";
import { Feedback } from "@/components/common/Feedback";
import { PageHeading } from "@/components/common/PageHeading";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { useResource } from "@/services/useResource";
import { adminApi } from "./api";
export function WecomSettingsPage() {
	const { session } = useAuth();
	const result = useResource(adminApi.settings);
	if (!session?.user.grants.some((g) => g.permission === "admin:manage"))
		return <Feedback message="无权访问登录设置" />;
	return (
		<>
			<PageHeading title="登录设置" eyebrow="系统管理" />
			{result.loading ? (
				<Feedback loading message="正在读取设置…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<WecomSettingsForm initial={result.data} csrf={session.csrf_token} />
			)}
		</>
	);
}
