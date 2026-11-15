import { Navigate, Outlet, useLocation, useNavigate } from "react-router-dom";
import { ActionButton } from "@/components/common/ActionButton";
import { Feedback } from "@/components/common/Feedback";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export function ProtectedRoute() {
	const { status, session, refresh, logout } = useAuth();
	const location = useLocation();
	const navigate = useNavigate();
	if (status === "loading") return <Feedback message="正在读取身份…" loading />;
	if (status === "anonymous")
		return (
			<Navigate
				to={appPaths.login}
				replace
				state={{ from: location.pathname + location.search }}
			/>
		);
	if (status === "forbidden")
		return (
			<div className="stack">
				<Feedback
					message="当前账户无法访问，请联系学校管理员。"
					onRetry={() => {
						void refresh();
					}}
				/>
				<div className="row-actions">
					<ActionButton
						onClick={() => {
							void logout().finally(() => {
								navigate(appPaths.login, { replace: true });
							});
						}}
					>
						切换账号登录
					</ActionButton>
				</div>
			</div>
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
	if (session?.must_change_password && location.pathname !== "/account")
		return (
			<Navigate
				to={`/account?next=${encodeURIComponent(location.pathname + location.search)}`}
				replace
			/>
		);
	return <Outlet />;
}
