import { type FormEvent, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { ActionButton } from "@/components/common/ActionButton";
import { PageHeading } from "@/components/common/PageHeading";
import { PasswordField } from "@/components/common/PasswordField";
import { errorMessage } from "@/features/achievements/api";
import { safeReturnPath } from "@/services/returnPath";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import { useAuth } from "./AuthProvider";
export function AccountPage() {
	const { session, changePassword } = useAuth();
	const [current, setCurrent] = useState("");
	const [password, setPassword] = useState("");
	const [confirm, setConfirm] = useState("");
	const [busy, setBusy] = useState(false);
	const [message, setMessage] = useState("");
	const [error, setError] = useState("");
	const [params] = useSearchParams();
	const navigate = useNavigate();
	const guard = useUnsavedChanges(
		Boolean(current || password || confirm),
		busy,
	);
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (busy) return;
		if (password !== confirm) {
			setError("两次新密码不一致");
			return;
		}
		setBusy(true);
		setError("");
		setMessage("");
		try {
			await changePassword(current, password);
			setCurrent("");
			setPassword("");
			setConfirm("");
			setMessage("密码已更新");
			if (session?.must_change_password) {
				guard.allowLeave();
				navigate(safeReturnPath(params.get("next")), { replace: true });
			}
		} catch (error) {
			setError(errorMessage(error));
		} finally {
			setBusy(false);
		}
	};
	return (
		<>
			<PageHeading title="账号安全" eyebrow={session?.user.display_name} />
			<form
				className="panel settings-form"
				onSubmit={(event) => void submit(event)}
			>
				{session?.must_change_password && (
					<p className="form-note">请先设置自己的登录密码。</p>
				)}
				<PasswordField
					id="current-password"
					label="当前密码"
					value={current}
					onChange={setCurrent}
					autoComplete="current-password"
					disabled={busy}
				/>
				<PasswordField
					id="new-password"
					label="新密码（至少12位）"
					value={password}
					onChange={setPassword}
					minLength={12}
					disabled={busy}
				/>
				<PasswordField
					id="confirm-password"
					label="确认新密码"
					value={confirm}
					onChange={setConfirm}
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
		</>
	);
}
