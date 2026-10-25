import { Navigate, Outlet, useLocation } from "react-router-dom";
import { Feedback } from "@/components/common/Feedback";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export function ProtectedRoute() {
	const { status, refresh } = useAuth();
	const location = useLocation();
	if (status === "loading") return <Feedback message="正在读取身份…" loading />;
	if (status === "anonymous")
		return (
			<Navigate
				to={appPaths.login}
				replace
				state={{ from: location.pathname }}
			/>
		);
	if (status === "forbidden")
		return (
			<Feedback
				message="当前账户无法访问，请联系学校管理员。"
				onRetry={() => {
					void refresh();
				}}
			/>
		);
	if (status === "error")
		return (
			<Feedback
				message="暂时无法连接，请稍后重试。"
				onRetry={() => {
					void refresh();
				}}
			/>
		);
	return <Outlet />;
}
