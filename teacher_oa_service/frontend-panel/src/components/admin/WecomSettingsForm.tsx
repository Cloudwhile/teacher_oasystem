import { type FormEvent, useState } from "react";
import { CheckCircle, Copy } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import { errorMessage } from "@/features/achievements/api";
import { adminApi, type WecomSettings } from "@/features/admin/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { authApi } from "@/features/auth/api";
import { HttpError } from "@/services/httpClient";
import { notifyInfo, notifySuccess } from "@/services/notify";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";

export function WecomSettingsForm({
	initial,
	csrf,
	onReload,
}: {
	initial: WecomSettings;
	csrf: string;
	onReload: () => void;
}) {
	const { refresh } = useAuth();
	const [saved, setSaved] = useState(initial);
	const [corp, setCorp] = useState(initial.corp_id);
	const [agent, setAgent] = useState(initial.agent_id);
	const [secret, setSecret] = useState("");
	const [enabled, setEnabled] = useState(initial.enabled);
	const [busy, setBusy] = useState(false);
	const [verifying, setVerifying] = useState(false);
	const [error, setError] = useState("");
	const [message, setMessage] = useState("");
	const [conflict, setConflict] = useState(false);
	const isDirty =
		corp !== saved.corp_id ||
		agent !== saved.agent_id ||
		enabled !== saved.enabled ||
		Boolean(secret);
	const guard = useUnsavedChanges(isDirty, busy);

	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (busy) return;
		setBusy(true);
		setError("");
		setMessage("");
		setConflict(false);
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
			notifySuccess("企业微信设置已保存");
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			setConflict(error instanceof HttpError && error.status === 409);
			setError(errorMessage(error));
		} finally {
			setBusy(false);
		}
	};

	const verifyConnection = async () => {
		if (verifying || busy) return;
		setVerifying(true);
		setError("");
		try {
			const liveOptions = await authApi.options();
			if (liveOptions.wecom_enabled) {
				const note = isDirty
					? "当前线上已开启企业微信登录（您有未保存的本地修改，请先保存后再扫码实测）。"
					: "企业微信登录入口已就绪，可前往登录页预览进行扫码验证。";
				setMessage(note);
				notifyInfo("企业微信入口检查通过");
			} else {
				const note = enabled
					? "请先点击右上角「保存企业微信设置」，使开关与配置生效后再验证。"
					: "当前企业微信登录处于关闭状态；开启开关并保存后即可在登录页启用扫码登录。";
				setMessage(note);
			}
		} catch (err) {
			setError(errorMessage(err, "登录入口状态检查失败，请稍后重试。"));
		} finally {
			setVerifying(false);
		}
	};

	const copyCallbackUrl = async () => {
		try {
			await navigator.clipboard?.writeText(saved.callback_url);
			notifySuccess("授权回调地址已复制");
		} catch {
			notifyInfo("请手动选中回调地址进行复制");
		}
	};

	const corpChanged = corp.trim() !== saved.corp_id;
	const secretRequired = enabled && (corpChanged || !saved.secret_configured);
	const secretHint = corpChanged
		? enabled
			? "您修改了企业 ID，保存时必须重新填写对应企业应用的 Secret。"
			: "更换企业 ID 将清空之前保存的原密钥。"
		: saved.secret_configured
			? "系统已安全加密保存密钥；若无需更换密钥，请留空即可保留原配置。"
			: "请在企业微信管理后台「应用管理」中查看并复制应用 Secret。";

	return (
		<form
			className="panel settings-form settings-form-wide"
			onSubmit={(event) => void submit(event)}
		>
			<div className="section-heading">
				<h2>企业微信应用</h2>
				<div className="row-actions">
					<ActionButton
						variant="secondary"
						disabled={busy || verifying}
						onClick={() => void verifyConnection()}
					>
						<CheckCircle aria-hidden="true" />
						{verifying ? "正在检查" : "检查登录入口"}
					</ActionButton>
					<ActionButton type="submit" variant="primary" disabled={busy}>
						{busy ? "正在保存" : "保存企业微信设置"}
					</ActionButton>
				</div>
			</div>
			<div className="wecom-guide">
				<strong>三步完成企业微信扫码登录配置</strong>
				<ol>
					<li>
						登录{" "}
						<a
							href="https://work.weixin.qq.com/"
							target="_blank"
							rel="noreferrer"
						>
							企业微信管理后台
						</a>
						，在「我的企业」底部复制<strong>企业 ID</strong>
						，并在「应用管理」自建应用中获取 <strong>AgentId</strong> 与{" "}
						<strong>Secret</strong>。
					</li>
					<li>
						进入该应用的「企业微信授权登录 → Web 网页」，将下方
						<strong>授权回调地址</strong>对应的域名填入授权回调域。
					</li>
					<li>
						在「成员与账号」中为教师绑定企业微信成员账号（UserID），勾选下方启用开关并保存。
					</li>
				</ol>
			</div>
			<label className="confirmation-row">
				<input
					type="checkbox"
					checked={enabled}
					disabled={busy}
					onChange={(e) => setEnabled(e.target.checked)}
				/>
				<span>启用企业微信登录</span>
			</label>
			<FormField
				id="corp"
				label="企业ID"
				hint="位于企业微信管理后台「我的企业 → 企业信息 → 企业ID」（例如 ww1234567890abcdef）。"
			>
				<input
					id="corp"
					value={corp}
					disabled={busy}
					maxLength={128}
					required={enabled}
					onChange={(e) => setCorp(e.target.value)}
				/>
			</FormField>
			<FormField
				id="agent"
				label="应用 AgentId"
				hint="自建应用详情页中的数字 AgentId（例如 1000002）。"
			>
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
				label="应用 Secret"
				required={secretRequired}
				hint={secretHint}
			>
				<input
					id="wecom-secret"
					type="password"
					autoComplete="new-password"
					value={secret}
					disabled={busy}
					maxLength={2048}
					required={secretRequired}
					onChange={(e) => setSecret(e.target.value)}
				/>
			</FormField>
			<FormField
				id="callback"
				label="授权回调地址"
				hint="只读地址，请将该地址所在域名填写至企业微信应用「企业微信授权登录」回调域。"
			>
				<div className="readonly-copy-row">
					<input id="callback" value={saved.callback_url} readOnly />
					<ActionButton
						variant="secondary"
						size="sm"
						onClick={() => void copyCallbackUrl()}
					>
						<Copy aria-hidden="true" />
						复制地址
					</ActionButton>
				</div>
			</FormField>
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			{conflict && (
				<ActionButton
					disabled={busy}
					onClick={() => {
						void guard.confirmLeave().then((allowed) => {
							if (allowed) onReload();
						});
					}}
				>
					加载最新设置
				</ActionButton>
			)}
			{message && (
				<p className="success-text" role="status">
					{message}
				</p>
			)}
		</form>
	);
}
