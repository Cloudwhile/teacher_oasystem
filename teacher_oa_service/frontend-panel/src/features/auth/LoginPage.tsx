import {
	type FormEvent,
	useCallback,
	useEffect,
	useRef,
	useState,
} from "react";
import { Navigate, useLocation, useSearchParams } from "react-router-dom";
import { Qr, User } from "reicon-react";
import { LoginMethods } from "@/components/auth/LoginMethods";
import { LoginPageLayout } from "@/components/auth/LoginPageLayout";
import { ActionButton } from "@/components/common/ActionButton";
import { Feedback } from "@/components/common/Feedback";
import { PasswordField } from "@/components/common/PasswordField";
import { TextField } from "@/components/common/TextField";
import { errorMessage } from "@/features/achievements/api";
import { safeReturnPath } from "@/services/returnPath";
import { useResource } from "@/services/useResource";
import { useAuth } from "./AuthProvider";
import { authApi, type LoginOptions } from "./api";
import { defaultLoginPage, loginPageApi } from "./loginPageApi";
export function LoginPage() {
	const { status, session, login } = useAuth();
	const location = useLocation();
	const [params] = useSearchParams();
	const next = safeReturnPath(params.get("next") ?? location.state?.from);
	const [options, setOptions] = useState<LoginOptions | null>(null);
	const [optionError, setOptionError] = useState("");
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const pending = useRef(false);
	const page = useResource(loginPageApi.public);
	const loadOptions = useCallback(async () => {
		setLoading(true);
		setOptionError("");
		try {
			setOptions(await authApi.options());
		} catch {
			setOptionError("登录入口暂时无法加载");
		} finally {
			setLoading(false);
		}
	}, []);
	useEffect(() => {
		void loadOptions();
	}, [loadOptions]);
	const preview = params.get("preview") === "1";
	const settings = page.data ?? defaultLoginPage;
	useEffect(() => {
		document.title = `${settings.title || "TeacherOA"} · 登录`;
	}, [settings.title]);
	if (status === "ready" && !preview)
		return (
			<Navigate
				to={
					session?.must_change_password
						? `/account?next=${encodeURIComponent(next)}`
						: next
				}
				replace
			/>
		);
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (preview || pending.current) return;
		pending.current = true;
		setBusy(true);
		setError("");
		try {
			await login(username, password);
		} catch (error) {
			setError(errorMessage(error, "登录失败，请重试"));
		} finally {
			pending.current = false;
			setBusy(false);
		}
	};
	const mode = /wxwork/i.test(navigator.userAgent) ? "h5" : "web";
	return (
		<LoginPageLayout
			settings={settings}
			loading={page.loading}
			error={Boolean(page.error)}
			onRetry={page.reload}
		>
			<section className="login-panel" aria-labelledby="login-title">
				<h1 id="login-title" className="visually-hidden">
					登录
				</h1>
				{preview && (
					<p className="login-preview-note" role="status">
						当前为登录页预览模式，登录与授权跳转已停用。
					</p>
				)}
				{(error || params.has("error")) && (
					<p className="inline-error" role="alert">
						{error || "企业微信登录未完成，请确认已开通账号并绑定企业微信。"}
					</p>
				)}
				<LoginMethods
					wecomEnabled={Boolean(options?.wecom_enabled)}
					busy={busy}
					account={
						<form
							className="login-fields"
							onSubmit={(event) => void submit(event)}
						>
							<TextField
								id="username"
								label="账号"
								icon={<User />}
								autoComplete="username"
								value={username}
								onChange={(e) => setUsername(e.target.value)}
								maxLength={64}
								required
								disabled={busy || preview}
								autoCapitalize="none"
								spellCheck={false}
							/>
							<PasswordField
								id="password"
								label="密码"
								value={password}
								onChange={setPassword}
								autoComplete="current-password"
								disabled={busy || preview}
							/>
							<ActionButton
								type="submit"
								variant="primary"
								className="login-submit"
								aria-busy={busy}
								disabled={
									preview || busy || status === "loading" || status === "ready"
								}
							>
								{busy ? "正在登录" : "登录"}
							</ActionButton>
						</form>
					}
					wecom={
						<div className="wecom-login">
							<Qr aria-hidden="true" />
							{preview ? (
								<ActionButton
									variant="primary"
									className="login-submit"
									disabled
								>
									{mode === "h5" ? "企业微信登录" : "企业微信扫码登录"}
								</ActionButton>
							) : (
								<a
									className="action-button action-button-primary login-submit"
									href={`/api/v1/auth/wecom/login?mode=${mode}&next=${encodeURIComponent(next)}`}
								>
									{mode === "h5" ? "企业微信登录" : "企业微信扫码登录"}
								</a>
							)}
						</div>
					}
				/>
				{(loading || optionError) && (
					<div className="login-options-status">
						{loading ? (
							<Feedback loading message="正在读取登录方式…" />
						) : optionError ? (
							<div className="row-actions">
								<p role="alert">{optionError}</p>
								<ActionButton onClick={() => void loadOptions()}>
									重试登录入口
								</ActionButton>
							</div>
						) : null}
					</div>
				)}
			</section>
		</LoginPageLayout>
	);
}
