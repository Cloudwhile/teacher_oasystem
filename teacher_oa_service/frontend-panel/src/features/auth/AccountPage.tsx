import { type FormEvent, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Monitor, Moon, Sun } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { PageHeading } from "@/components/common/PageHeading";
import { PasswordField } from "@/components/common/PasswordField";
import { errorMessage } from "@/features/achievements/api";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";
import { safeReturnPath } from "@/services/returnPath";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import {
	type ThemePreference,
	useUiPreferencesStore,
} from "@/stores/uiPreferencesStore";
import { useAuth } from "./AuthProvider";

const themeOptions: {
	value: ThemePreference;
	label: string;
	icon: typeof Monitor;
}[] = [
	{ value: "system", label: "跟随系统", icon: Monitor },
	{ value: "light", label: "浅色模式", icon: Sun },
	{ value: "dark", label: "深色模式", icon: Moon },
];

export function AccountPage() {
	const { session, changePassword, refresh } = useAuth();
	const theme = useUiPreferencesStore((state) => state.theme);
	const setTheme = useUiPreferencesStore((state) => state.setTheme);
	const [current, setCurrent] = useState("");
	const [password, setPassword] = useState("");
	const [confirm, setConfirm] = useState("");
	const [fieldErrors, setFieldErrors] = useState<{
		current?: string;
		password?: string;
		confirm?: string;
	}>({});
	const [busy, setBusy] = useState(false);
	const [message, setMessage] = useState("");
	const [error, setError] = useState("");
	const formRef = useRef<HTMLFormElement>(null);
	const [params] = useSearchParams();
	const navigate = useNavigate();
	const guard = useUnsavedChanges(
		Boolean(current || password || confirm),
		busy,
	);
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (busy) return;
		const nextErrors: {
			current?: string;
			password?: string;
			confirm?: string;
		} = {};
		if (!current) nextErrors.current = "请输入当前密码";
		if (!password || password.length < 12)
			nextErrors.password = "新密码长度不能少于 12 位";
		if (!confirm) nextErrors.confirm = "请再次输入新密码";
		else if (password !== confirm)
			nextErrors.confirm = "两次输入的新密码不一致";
		if (nextErrors.current || nextErrors.password || nextErrors.confirm) {
			setFieldErrors(nextErrors);
			const targetId = nextErrors.current
				? "#current-password"
				: nextErrors.password
					? "#new-password"
					: "#confirm-password";
			formRef.current?.querySelector<HTMLElement>(targetId)?.focus();
			return;
		}
		setBusy(true);
		setError("");
		setFieldErrors({});
		setMessage("");
		try {
			await changePassword(current, password);
			setCurrent("");
			setPassword("");
			setConfirm("");
			setMessage("密码已更新");
			notifySuccess("密码已更新");
			if (session?.must_change_password) {
				guard.allowLeave();
				navigate(safeReturnPath(params.get("next")), { replace: true });
			}
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			setError(errorMessage(error));
		} finally {
			setBusy(false);
		}
	};
	return (
		<>
			<PageHeading title="账号安全" eyebrow={session?.user.display_name} />
			<div className="settings-split">
				<form
					ref={formRef}
					noValidate
					className="panel settings-form"
					onSubmit={(event) => void submit(event)}
				>
					<div className="section-heading">
						<h2>修改登录密码</h2>
					</div>
					{session?.must_change_password && (
						<p className="form-note">请先设置自己的登录密码。</p>
					)}
					<PasswordField
						id="current-password"
						label="当前密码"
						value={current}
						error={fieldErrors.current}
						onChange={(val) => {
							setCurrent(val);
							if (fieldErrors.current)
								setFieldErrors((prev) => ({ ...prev, current: undefined }));
						}}
						autoComplete="current-password"
						disabled={busy}
					/>
					<PasswordField
						id="new-password"
						label="新密码（至少12位）"
						value={password}
						error={fieldErrors.password}
						hint="建议使用大小写字母、数字与符号组合。"
						onChange={(val) => {
							setPassword(val);
							if (fieldErrors.password)
								setFieldErrors((prev) => ({ ...prev, password: undefined }));
						}}
						minLength={12}
						disabled={busy}
					/>
					<PasswordField
						id="confirm-password"
						label="确认新密码"
						value={confirm}
						error={fieldErrors.confirm}
						onChange={(val) => {
							setConfirm(val);
							if (fieldErrors.confirm)
								setFieldErrors((prev) => ({ ...prev, confirm: undefined }));
						}}
						minLength={12}
						disabled={busy}
					/>
					{error && (
						<p role="alert" className="inline-error">
							{error}
						</p>
					)}
					{message && (
						<p role="status" className="success-text">
							{message}
						</p>
					)}
					<div className="row-actions">
						<ActionButton type="submit" variant="primary" disabled={busy}>
							{busy ? "正在更新" : "更新密码"}
						</ActionButton>
					</div>
				</form>
				<div className="editor-sections">
					<section
						className="panel settings-form"
						aria-labelledby="appearance-heading"
					>
						<div className="section-heading">
							<h2 id="appearance-heading">界面外观</h2>
						</div>
						<p className="field-hint">
							选择工作台的颜色主题。默认跟随操作系统深浅色切换，也可在此固定当前浏览器的显示偏好。
						</p>
						<fieldset className="theme-segmented">
							<legend className="visually-hidden">颜色主题</legend>
							{themeOptions.map((option) => {
								const Icon = option.icon;
								const active = theme === option.value;
								return (
									<label key={option.value} className="theme-option">
										<input
											type="radio"
											className="visually-hidden"
											name="color-theme"
											value={option.value}
											checked={active}
											onChange={() => setTheme(option.value)}
										/>
										<Icon aria-hidden="true" />
										<span>{option.label}</span>
									</label>
								);
							})}
						</fieldset>
					</section>
					<section
						className="panel settings-form"
						aria-labelledby="security-tips-heading"
					>
						<div className="section-heading">
							<h2 id="security-tips-heading">安全说明</h2>
						</div>
						<ul className="wecom-aside-notes">
							<li>
								修改密码成功后，其他设备上的历史会话将自动失效并要求重新验证。
							</li>
							<li>
								密码长度须为 12 至 128 位，请勿使用与工号相同的纯数字弱口令。
							</li>
							<li>
								在公共办公室电脑完成填报或审批后，建议点击左下角退出按钮主动结束会话。
							</li>
						</ul>
					</section>
				</div>
			</div>
		</>
	);
}
