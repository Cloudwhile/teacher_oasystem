import { LoginPageSettingsForm } from "@/components/admin/LoginPageSettingsForm";
import { Feedback } from "@/components/common/Feedback";
import { PageHeading } from "@/components/common/PageHeading";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { loginPageApi } from "@/features/auth/loginPageApi";
import { useResource } from "@/services/useResource";

export function LoginAppearancePage() {
	const { session } = useAuth();
	const result = useResource(loginPageApi.read);
	if (
		!session?.user.grants.some((grant) => grant.permission === "admin:manage")
	)
		return <Feedback message="无权访问登录页设置" />;
	return (
		<>
			<PageHeading title="登录页与公告" eyebrow="系统管理">
				<a
					className="action-button action-button-secondary"
					href="/login?preview=1"
					target="_blank"
					rel="noreferrer"
				>
					查看登录页
				</a>
			</PageHeading>
			{result.loading ? (
				<Feedback loading message="正在读取登录页设置…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<LoginPageSettingsForm
					initial={result.data}
					csrf={session.csrf_token}
				/>
			)}
		</>
	);
}
