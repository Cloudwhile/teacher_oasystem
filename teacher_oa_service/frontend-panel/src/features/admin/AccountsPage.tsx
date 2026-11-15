import { useState } from "react";
import { Pen, Refresh, Search } from "reicon-react";
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
	const [search, setSearch] = useState("");
	const [message, setMessage] = useState("");
	if (!session?.user.grants.some((g) => g.permission === "admin:manage"))
		return <Feedback message="无权访问账号管理" />;
	const keyword = search.trim().toLowerCase();
	const items =
		result.data?.items.filter((account) => {
			if (!keyword) return true;
			return [
				account.username,
				account.name,
				account.department,
				account.wecom_userid ?? "",
				roles[account.role] ?? account.role,
			].some((field) => field.toLowerCase().includes(keyword));
		}) ?? [];
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
					<div className="section-heading">
						<div className="search-control account-search">
							<Search aria-hidden="true" />
							<input
								type="search"
								value={search}
								maxLength={100}
								aria-label="搜索成员与账号"
								placeholder="搜索账号、姓名、部门或企业微信"
								onChange={(event) => setSearch(event.target.value)}
							/>
						</div>
						<span className="section-meta">共 {items.length} 人</span>
					</div>
					{items.length ? (
						<section
							className="table-scroll"
							// biome-ignore lint/a11y/noNoninteractiveTabindex: horizontal scrolling must be keyboard accessible.
							tabIndex={0}
							aria-label="成员与账号列表"
						>
							<table className="achievement-table">
								<thead>
									<tr>
										<th scope="col">账号</th>
										<th scope="col">姓名</th>
										<th scope="col">部门</th>
										<th scope="col">角色</th>
										<th scope="col">状态</th>
										<th scope="col">企业微信</th>
										<th scope="col">操作</th>
									</tr>
								</thead>
								<tbody>
									{items.map((account) => (
										<tr key={account.id}>
											<td data-label="账号">{account.username}</td>
											<td data-label="姓名">{account.name}</td>
											<td data-label="部门">{account.department}</td>
											<td data-label="角色">{roles[account.role]}</td>
											<td data-label="状态">
												{account.active ? "启用" : "停用"}
												{account.must_change ? " · 待改密" : ""}
											</td>
											<td data-label="企业微信">
												{account.wecom_userid || "未绑定"}
											</td>
											<td data-label="操作">
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
						</section>
					) : (
						<p className="empty-message">没有符合条件的成员账号。</p>
					)}
				</section>
			)}
		</>
	);
}
