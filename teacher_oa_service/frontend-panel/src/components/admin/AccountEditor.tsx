import { type FormEvent, useState } from "react";
import { X } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import { IconButton } from "@/components/common/IconButton";
import { PasswordField } from "@/components/common/PasswordField";
import { errorMessage } from "@/features/achievements/api";
import {
	type AccountInput,
	type AccountItem,
	adminApi,
} from "@/features/admin/api";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
export function AccountEditor({
	account,
	self = false,
	csrf,
	onDone,
	onCancel,
}: {
	account: AccountItem | null;
	self?: boolean;
	csrf: string;
	onDone: () => void;
	onCancel: () => void;
}) {
	const initial: AccountInput = {
		id: account?.id,
		username: account?.username ?? "",
		name: account?.name ?? "",
		department: account?.department ?? "",
		role: account?.role ?? "teacher",
		active: account?.active ?? true,
		password: "",
		wecom_userid: account?.wecom_userid ?? "",
	};
	const [data, setData] = useState(initial);
	const [busy, setBusy] = useState(false);
	const [error, setError] = useState("");
	const guard = useUnsavedChanges(
		JSON.stringify(initial) !== JSON.stringify(data),
		busy,
	);
	const submit = async (event: FormEvent) => {
		event.preventDefault();
		if (busy) return;
		setBusy(true);
		setError("");
		try {
			await adminApi.saveAccount(data, csrf);
			guard.allowLeave();
			onDone();
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
				<h2>{account ? "编辑账号" : "新增账号"}</h2>
				<div className="row-actions">
					<IconButton
						label="取消编辑账号"
						disabled={busy}
						onClick={() => {
							if (guard.confirmLeave()) onCancel();
						}}
					>
						<X />
					</IconButton>
					<ActionButton type="submit" variant="primary" disabled={busy}>
						{busy ? "正在保存" : "保存账号"}
					</ActionButton>
				</div>
			</div>
			<div className="form-grid">
				{(["username", "name", "department", "wecom_userid"] as const).map(
					(key) => (
						<FormField
							key={key}
							id={key}
							label={
								{
									username: "登录账号",
									name: "姓名",
									department: "部门",
									wecom_userid: "企业微信成员账号",
								}[key]
							}
							required={key !== "wecom_userid"}
						>
							<input
								id={key}
								value={data[key] ?? ""}
								required={key !== "wecom_userid"}
								maxLength={key === "username" ? 64 : 100}
								disabled={busy || (key === "username" && Boolean(account))}
								autoComplete="off"
								onChange={(e) => setData({ ...data, [key]: e.target.value })}
							/>
						</FormField>
					),
				)}
				<FormField id="role" label="角色">
					<select
						id="role"
						value={data.role}
						disabled={busy || self}
						onChange={(e) => setData({ ...data, role: e.target.value })}
					>
						<option value="teacher">教师</option>
						<option value="group_leader">教研组长</option>
						<option value="reviewer">审核人员</option>
						<option value="admin">管理员</option>
					</select>
				</FormField>
				{!self && (
					<PasswordField
						id="initial-password"
						label={account ? "重置密码（留空保留）" : "初始密码（至少12位）"}
						value={data.password ?? ""}
						required={!account}
						minLength={12}
						disabled={busy}
						onChange={(password) => setData({ ...data, password })}
					/>
				)}
			</div>
			<label className="confirmation-row">
				<input
					type="checkbox"
					checked={data.active}
					disabled={busy || self}
					onChange={(e) => setData({ ...data, active: e.target.checked })}
				/>
				启用账号
			</label>
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
		</form>
	);
}
