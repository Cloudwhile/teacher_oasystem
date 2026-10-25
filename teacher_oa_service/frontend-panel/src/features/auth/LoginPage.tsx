import { type FormEvent, useEffect, useState } from "react";
import {
	LuArrowRight,
	LuArrowUpRight,
	LuGraduationCap,
	LuQrCode,
} from "react-icons/lu";
import { Navigate, useSearchParams } from "react-router-dom";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { appPaths } from "@/routes/routePaths";
import { useAuth } from "./AuthProvider";
import { authApi, type LoginOptions } from "./api";

export function LoginPage() {
	const { status, login } = useAuth();
	const [options, setOptions] = useState<LoginOptions | null>(null);
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	const [identity, setIdentity] = useState("teacher");
	const [params] = useSearchParams();
	useEffect(() => {
		let active = true;
		authApi
			.options()
			.then((value) => {
				if (active) setOptions(value);
			})
			.catch(() => {
				if (active) setError("登录入口暂时无法加载，请稍后重试。");
			});
		return () => {
			active = false;
		};
	}, []);
	if (status === "ready") return <Navigate to={appPaths.dashboard} replace />;
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		setBusy(true);
		setError("");
		try {
			await login(identity);
		} catch {
			setError("登录未完成，请确认账户状态后重试。");
		} finally {
			setBusy(false);
		}
	};
	return (
		<main className="login-page">
			<section className="login-panel" aria-labelledby="login-title">
				<div className="brand">
					<LuGraduationCap aria-hidden="true" />
					<span>TeacherOA</span>
				</div>
				<p className="eyebrow">教师工作空间</p>
				<h1 id="login-title">登录</h1>
				{status === "loading" && <Feedback message="正在读取身份…" loading />}
				{!options && !error && <Feedback message="正在加载登录入口…" loading />}
				{(error || params.has("error")) && (
					<p className="inline-error" role="alert">
						{error || "企业微信授权未完成，请重新登录。"}
					</p>
				)}
				{options?.wecom_enabled && (
					<div className="login-methods">
						<a href="/api/v1/auth/wecom/login" className="action-row">
							<LuQrCode />
							<span>企业微信扫码</span>
							<LuArrowUpRight />
						</a>
						<a className="text-link" href="/api/v1/auth/wecom/login?mode=h5">
							企业微信内登录
						</a>
					</div>
				)}
				{options?.mock_enabled && (
					<form
						onSubmit={(event) => {
							void submit(event);
						}}
						className="identity-form"
					>
						<label htmlFor="identity">测试身份</label>
						<div className="identity-input">
							<select
								id="identity"
								value={identity}
								onChange={(event) => setIdentity(event.target.value)}
								disabled={busy}
							>
								<option value="teacher">教师</option>
								<option value="leader">教研组长</option>
								<option value="reviewer">审核人员</option>
								<option value="admin">管理员</option>
							</select>
							<IconButton
								type="submit"
								label="登录"
								disabled={busy || status === "loading"}
							>
								<LuArrowRight />
							</IconButton>
						</div>
					</form>
				)}
				{options && !options.mock_enabled && !options.wecom_enabled && (
					<p className="empty-message">登录入口暂未开放，请联系学校管理员。</p>
				)}
			</section>
			<footer>TeacherOA · 教师综合 OA</footer>
		</main>
	);
}
