import { type FormEvent, useRef, useState } from "react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import {
	type LoginPageSettings,
	loginPageApi,
} from "@/features/auth/loginPageApi";
import { HttpError } from "@/services/httpClient";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import { AnnouncementManager } from "./AnnouncementManager";
import { LoginImageField } from "./LoginImageField";

export function LoginPageSettingsForm({
	initial,
	csrf,
	onReload,
}: {
	initial: LoginPageSettings;
	csrf: string;
	onReload: () => void;
}) {
	const { refresh } = useAuth();
	const [data, setData] = useState(initial);
	const [saved, setSaved] = useState(initial);
	const [saving, setSaving] = useState(false);
	const [uploading, setUploading] = useState(false);
	const [noticeDirty, setNoticeDirty] = useState(false);
	const [error, setError] = useState("");
	const [message, setMessage] = useState("");
	const [conflict, setConflict] = useState(false);
	const pending = useRef(false);
	const busy = saving || uploading;
	const guard = useUnsavedChanges(
		JSON.stringify(data) !== JSON.stringify(saved) || noticeDirty,
		busy,
	);
	const persist = async (next: LoginPageSettings) => {
		if (pending.current || uploading) return false;
		pending.current = true;
		setSaving(true);
		setError("");
		setMessage("");
		setConflict(false);
		try {
			const result = await loginPageApi.save(next, csrf);
			setData(result);
			setSaved(result);
			setMessage("登录页设置已保存");
			return true;
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			setConflict(error instanceof HttpError && error.status === 409);
			setError(errorMessage(error));
			return false;
		} finally {
			pending.current = false;
			setSaving(false);
		}
	};
	const submit = (event: FormEvent) => {
		event.preventDefault();
		void persist(data);
	};
	return (
		<div className="login-page-settings">
			{error && (
				<p role="alert" className="inline-error">
					{error}
				</p>
			)}
			{conflict && (
				<ActionButton
					disabled={busy}
					onClick={() => {
						if (guard.confirmLeave()) onReload();
					}}
				>
					加载最新设置
				</ActionButton>
			)}
			{message && (
				<p role="status" className="success-text">
					{message}
				</p>
			)}
			<form className="panel settings-form" onSubmit={submit}>
				<div className="section-heading">
					<h2>登录页外观</h2>
					<ActionButton type="submit" variant="primary" disabled={busy}>
						{saving ? "正在保存" : "保存设置"}
					</ActionButton>
				</div>
				<FormField id="site-title" label="网站名称" required>
					<input
						id="site-title"
						value={data.title}
						maxLength={80}
						required
						disabled={busy}
						onChange={(event) =>
							setData({ ...data, title: event.target.value })
						}
					/>
				</FormField>
				<div className="form-grid">
					{(
						[
							["background_url", "背景图片"],
							["logo_url", "LOGO"],
						] as const
					).map(([key, label]) => (
						<LoginImageField
							key={key}
							label={label}
							value={data[key]}
							csrf={csrf}
							disabled={busy}
							onBusy={setUploading}
							onError={setError}
							onChange={(url) =>
								setData((previous) => ({ ...previous, [key]: url }))
							}
						/>
					))}
				</div>
				<FormField id="site-footer" label="页脚内容">
					<textarea
						id="site-footer"
						rows={2}
						value={data.footer}
						maxLength={300}
						disabled={busy}
						onChange={(event) =>
							setData({ ...data, footer: event.target.value })
						}
					/>
				</FormField>
			</form>
			<AnnouncementManager
				items={data.announcements}
				disabled={busy}
				onDirty={setNoticeDirty}
				onSave={(announcements) => persist({ ...data, announcements })}
			/>
		</div>
	);
}
