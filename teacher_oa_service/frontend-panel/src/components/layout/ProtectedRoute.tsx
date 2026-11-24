import { useEffect } from "react";
import { Navigate, Outlet, useLocation, useNavigate } from "react-router-dom";
import { AlertCircle, Refresh, ShieldAlert } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import {
	StandaloneStatusShell,
	StatusStateCard,
} from "@/components/common/StatusState";
import { useAuth } from "@/features/auth/AuthProvider";
import { appPaths } from "@/routes/routePaths";

export function ProtectedRoute() {
	const { status, session, sessionExpired, refresh, logout } = useAuth();
	const location = useLocation();
	const navigate = useNavigate();

	useEffect(() => {
		if (status === "forbidden") {
			document.title = "访问受限 · 教师发展与成果管理系统";
		} else if (status === "error") {
			document.title = "连接异常 · 教师发展与成果管理系统";
		}
	}, [status]);

	if (status === "loading") {
		return <StandaloneStatusShell loading loadingLabel="正在读取身份…" />;
	}
	if (status === "anonymous") {
		return (
			<Navigate
				to={appPaths.login}
				replace
				state={{
					from: location.pathname + location.search,
					expired: Boolean(sessionExpired),
				}}
			/>
		);
	}
	if (status === "forbidden") {
		return (
			<StandaloneStatusShell>
				<StatusStateCard
					framed
					role="alert"
					tone="warning"
					icon={<ShieldAlert />}
					code="403 · 访问受限"
					title="账户暂无访问权限"
					description="当前账户无法访问，请联系学校管理员。"
					actions={
						<>
							<ActionButton
								variant="secondary"
								onClick={() => {
									void refresh();
								}}
							>
								<Refresh aria-hidden="true" />
								重新加载
							</ActionButton>
							<ActionButton
								variant="primary"
								onClick={() => {
									void logout().finally(() => {
										navigate(appPaths.login, { replace: true });
									});
								}}
							>
								切换账号登录
							</ActionButton>
						</>
					}
				/>
			</StandaloneStatusShell>
		);
	}
	if (status === "error") {
		return (
			<StandaloneStatusShell>
				<StatusStateCard
					framed
					role="alert"
					tone="danger"
					icon={<AlertCircle />}
					code="连接异常"
					title="服务连接中断"
					description="暂时无法连接，请稍后重试。"
					actions={
						<>
							<ActionButton
								variant="primary"
								onClick={() => {
									void refresh();
								}}
							>
								<Refresh aria-hidden="true" />
								重新加载
							</ActionButton>
							<ActionButton
								variant="secondary"
								onClick={() => {
									navigate(appPaths.login, { replace: true });
								}}
							>
								返回登录页
							</ActionButton>
						</>
					}
				/>
			</StandaloneStatusShell>
		);
	}
	if (session?.must_change_password && location.pathname !== "/account") {
		return (
			<Navigate
				to={`/account?next=${encodeURIComponent(location.pathname + location.search)}`}
				replace
			/>
		);
	}
	return <Outlet />;
}
