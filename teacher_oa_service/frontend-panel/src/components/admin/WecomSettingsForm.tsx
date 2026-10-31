import { type FormEvent, useState } from "react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import { errorMessage } from "@/features/achievements/api";
import { adminApi, type WecomSettings } from "@/features/admin/api";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
export function WecomSettingsForm({
	initial,
	csrf,
}: {
	initial: WecomSettings;
	csrf: string;
}) {
	const [saved, setSaved] = useState(initial);
	const [corp, setCorp] = useState(initial.corp_id);
	const [agent, setAgent] = useState(initial.agent_id);
	const [secret, setSecret] = useState("");
	const [enabled, setEnabled] = useState(initial.enabled);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState("");
	const [message, setMessage] = useState("");
	useUnsavedChanges(
		corp !== saved.corp_id ||
			agent !== saved.agent_id ||
			enabled !== saved.enabled ||
			Boolean(secret),
		busy,
	);
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (busy) return;
		setBusy(true);
		setError("");
		setMessage("");
		try {
			const result = await adminApi.saveSettings(
				{
					corp_id: corp,
					agent_id: agent,
					secret: secret || undefined,
					enabled,
					version: saved.version,
				},
				csrf,
			);
			setSaved(result);
			setCorp(result.corp_id);
			setAgent(result.agent_id);
			setSecret("");
			setMessage("企业微信设置已保存");
		} catch (error) {
			setError(errorMessage(error));
		} finally {
			setBusy(false);
		}
	};
	return (
		<form
			className="panel settings-form"
			onSubmit={(event) => void submit(event)}
		>
			<div className="section-heading">
				<h2>企业微信应用</h2>
				<ActionButton type="submit" variant="primary" disabled={busy}>
					{busy ? "正在保存" : "保存企业微信设置"}
				</ActionButton>
			</div>
			<label className="confirmation-row">
				<input
					type="checkbox"
					checked={enabled}
					disabled={busy}
					onChange={(e) => setEnabled(e.target.checked)}
				/>
				启用企业微信登录
			</label>
			<FormField id="corp" label="企业ID">
				<input
					id="corp"
					value={corp}
					disabled={busy}
					maxLength={128}
					required={enabled}
					onChange={(e) => setCorp(e.target.value)}
				/>
			</FormField>
			<FormField id="agent" label="应用 AgentId">
				<input
					id="agent"
					value={agent}
					disabled={busy}
					maxLength={64}
					required={enabled}
					onChange={(e) => setAgent(e.target.value)}
				/>
			</FormField>
			<FormField
				id="wecom-secret"
				label={
					saved.secret_configured ? "应用 Secret（留空保留）" : "应用 Secret"
				}
			>
				<input
					id="wecom-secret"
					type="password"
					autoComplete="new-password"
					value={secret}
					disabled={busy}
					maxLength={2048}
					required={enabled && !saved.secret_configured}
					onChange={(e) => setSecret(e.target.value)}
				/>
			</FormField>
			<FormField id="callback" label="授权回调地址">
				<input id="callback" value={saved.callback_url} readOnly />
			</FormField>
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			{message && (
				<p className="success-text" role="status">
					{message}
				</p>
			)}
		</form>
	);
}
