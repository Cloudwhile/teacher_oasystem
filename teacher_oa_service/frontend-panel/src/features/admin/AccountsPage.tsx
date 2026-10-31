import { useState } from "react";
import { Pen, Refresh } from "reicon-react";
import { AccountEditor } from "@/components/admin/AccountEditor";
import { ActionButton } from "@/components/common/ActionButton";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { useResource } from "@/services/useResource";
import { type AccountItem, adminApi } from "./api";

const roles: Record<string, string> = {
	teacher: "教师",
	group_leader: "教研组长",
	reviewer: "审核人员",
	admin: "管理员",
};
export function AccountsPage() {
	const { session } = useAuth();
	const result = useResource(adminApi.accounts);
	const [editing, setEditing] = useState<AccountItem | null | undefined>(
		undefined,
	);
	const [message, setMessage] = useState("");
	if (!session?.user.grants.some((g) => g.permission === "admin:manage"))
		return <Feedback message="无权访问账号管理" />;
	return (
		<>
			<PageHeading title="成员与账号" eyebrow="系统管理">
				<IconButton
					label="刷新账号"
					disabled={editing !== undefined}
					onClick={result.reload}
				>
					<Refresh />
				</IconButton>
				<ActionButton
					variant="primary"
					disabled={editing !== undefined}
					onClick={() => {
						setEditing(null);
						setMessage("");
					}}
				>
					新增账号
				</ActionButton>
			</PageHeading>
			{message && (
				<p role="status" className="success-text">
					{message}
				</p>
			)}
			{editing !== undefined && (
				<AccountEditor
					key={editing?.id ?? "new"}
					account={editing}
					self={editing?.id === session.user.id}
					csrf={session.csrf_token}
					onCancel={() => setEditing(undefined)}
					onDone={() => {
						setEditing(undefined);
						setMessage("账号已保存");
						result.reload();
					}}
				/>
			)}
			{result.loading ? (
				<Feedback loading message="正在读取账号…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<section className="panel">
					<div className="table-scroll">
						<table className="achievement-table">
							<thead>
								<tr>
									<th>账号</th>
									<th>姓名</th>
									<th>部门</th>
									<th>角色</th>
									<th>状态</th>
									<th>企业微信</th>
									<th>操作</th>
								</tr>
							</thead>
							<tbody>
								{result.data.items.map((account) => (
									<tr key={account.id}>
										<td>{account.username}</td>
										<td>{account.name}</td>
										<td>{account.department}</td>
										<td>{roles[account.role]}</td>
										<td>
											{account.active ? "启用" : "停用"}
											{account.must_change ? " · 待改密" : ""}
										</td>
										<td>{account.wecom_userid || "未绑定"}</td>
										<td>
											<IconButton
												label={`编辑${account.username}`}
												disabled={editing !== undefined}
												onClick={() => setEditing(account)}
											>
												<Pen />
											</IconButton>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				</section>
			)}
		</>
	);
}
