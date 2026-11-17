import { useMemo, useState } from "react";
import {
	Pen,
	Refresh,
	Search,
	SortAsc,
	SortDesc,
	Users,
	X,
} from "reicon-react";
import { AccountEditor } from "@/components/admin/AccountEditor";
import { ActionButton } from "@/components/common/ActionButton";
import { useConfirm } from "@/components/common/ConfirmDialog";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { PermissionDenied } from "@/components/common/PermissionDenied";
import { TableSkeleton } from "@/components/common/Skeleton";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";
import { useResource } from "@/services/useResource";
import { type AccountItem, adminApi } from "./api";

const roles: Record<string, string> = {
	teacher: "教师",
	group_leader: "教研组长",
	reviewer: "审核人员",
	admin: "管理员",
};

type SortColumn = "username" | "name" | "department" | "role" | "active";
const collator = new Intl.Collator("zh-CN", {
	numeric: true,
	sensitivity: "base",
});

export function AccountsPage() {
	const { session, refresh } = useAuth();
	const confirm = useConfirm();
	const result = useResource(adminApi.accounts);
	const [editing, setEditing] = useState<AccountItem | null | undefined>(
		undefined,
	);
	const [search, setSearch] = useState("");
	const [sortCol, setSortCol] = useState<SortColumn | null>(null);
	const [sortDir, setSortDir] = useState<"ascending" | "descending">(
		"ascending",
	);
	const [page, setPage] = useState(1);
	const [pageSize, setPageSize] = useState(20);
	const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
	const [batchBusy, setBatchBusy] = useState(false);
	const [batchError, setBatchError] = useState("");
	const [message, setMessage] = useState("");

	const allItems = result.data?.items ?? [];
	const keyword = search.trim().toLowerCase();

	const filteredItems = useMemo(() => {
		const matched = allItems.filter((account) => {
			if (!keyword) return true;
			return [
				account.username,
				account.name,
				account.department,
				account.wecom_userid ?? "",
				roles[account.role] ?? account.role,
			].some((field) => field.toLowerCase().includes(keyword));
		});
		if (!sortCol) return matched;
		const factor = sortDir === "ascending" ? 1 : -1;
		return [...matched].sort((a, b) => {
			if (sortCol === "active") {
				return factor * (Number(a.active) - Number(b.active));
			}
			const left = sortCol === "role" ? (roles[a.role] ?? a.role) : a[sortCol];
			const right = sortCol === "role" ? (roles[b.role] ?? b.role) : b[sortCol];
			return factor * collator.compare(left, right);
		});
	}, [allItems, keyword, sortCol, sortDir]);

	if (!session?.user.grants.some((g) => g.permission === "admin:manage")) {
		return <PermissionDenied title="成员与账号" message="无权访问账号管理" />;
	}

	const totalPages = Math.max(1, Math.ceil(filteredItems.length / pageSize));
	const safePage = Math.min(page, totalPages);
	const pageItems = filteredItems.slice(
		(safePage - 1) * pageSize,
		safePage * pageSize,
	);
	const selectablePageItems = pageItems.filter(
		(account) => account.id !== session.user.id,
	);
	const allPageSelected =
		selectablePageItems.length > 0 &&
		selectablePageItems.every((item) => selectedIds.has(item.id));

	const toggleSort = (col: SortColumn) => {
		if (sortCol === col) {
			setSortDir((prev) => (prev === "ascending" ? "descending" : "ascending"));
		} else {
			setSortCol(col);
			setSortDir("ascending");
		}
	};

	const runBatchActive = async (nextActive: boolean) => {
		if (batchBusy) return;
		const targets = allItems.filter(
			(item) => selectedIds.has(item.id) && item.id !== session.user.id,
		);
		if (!targets.length) return;
		const ok = await confirm({
			title: `确认批量${nextActive ? "启用" : "停用"} ${targets.length} 个账号？`,
			description: nextActive
				? "启用后，所选成员可正常登录系统。"
				: "停用后，所选成员将无法登录系统，其历史成果记录仍会保留。",
			confirmLabel: nextActive ? "确认启用" : "确认停用",
			tone: nextActive ? "primary" : "danger",
		});
		if (!ok) return;
		setBatchBusy(true);
		setBatchError("");
		setMessage("");
		let done = 0;
		const failures: string[] = [];
		try {
			for (const target of targets) {
				try {
					await adminApi.saveAccount(
						{
							id: target.id,
							username: target.username,
							name: target.name,
							department: target.department,
							role: target.role,
							active: nextActive,
							password: "",
							wecom_userid: target.wecom_userid ?? "",
						},
						session.csrf_token,
					);
					done += 1;
				} catch (err) {
					if (err instanceof HttpError && err.status === 401) {
						void refresh();
						return;
					}
					failures.push(`${target.username}（${errorMessage(err)}）`);
				}
			}
			setSelectedIds(new Set());
			result.reload();
			if (done > 0) {
				const summary = `已批量${nextActive ? "启用" : "停用"} ${done} 个账号`;
				setMessage(summary);
				notifySuccess(summary);
			}
			if (failures.length > 0) {
				setBatchError(`部分账号未完成：${failures.join("；")}`);
			}
		} finally {
			setBatchBusy(false);
		}
	};

	const showHeaderCreate =
		!result.loading && editing === undefined && allItems.length > 0;

	return (
		<>
			<PageHeading title="成员与账号">
				<IconButton
					label="刷新账号"
					disabled={
						editing !== undefined || result.loading || result.refreshing
					}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
				{showHeaderCreate && (
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
				)}
			</PageHeading>
			{message && (
				<p role="status" className="success-text">
					{message}
				</p>
			)}
			{batchError && (
				<p role="alert" className="inline-error">
					{batchError}
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
						notifySuccess("账号已保存");
						result.reload();
					}}
				/>
			)}
			{result.loading && !result.data ? (
				<TableSkeleton message="正在读取账号…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : allItems.length === 0 ? (
				<section className="panel">
					<EmptyState
						icon={<Users />}
						title="暂无成员账号"
						description="创建成员账号并分配部门与角色后，教师即可登录填报成果。"
						action={
							editing === undefined ? (
								<ActionButton
									variant="primary"
									onClick={() => {
										setEditing(null);
										setMessage("");
									}}
								>
									新增账号
								</ActionButton>
							) : undefined
						}
					/>
				</section>
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
								onChange={(event) => {
									setSearch(event.target.value);
									setPage(1);
								}}
							/>
							{search && (
								<IconButton
									label="清空账号搜索"
									onClick={() => {
										setSearch("");
										setPage(1);
									}}
								>
									<X />
								</IconButton>
							)}
						</div>
						<span className="section-meta">
							{keyword
								? `共 ${allItems.length} 人 · 筛选出 ${filteredItems.length} 人`
								: `共 ${allItems.length} 人`}
						</span>
					</div>
					{selectedIds.size > 0 && (
						<section className="batch-bar" aria-label="批量账号操作">
							<span>已选择 {selectedIds.size} 个成员账号</span>
							<div className="row-actions">
								<ActionButton
									variant="secondary"
									size="sm"
									disabled={batchBusy}
									onClick={() => void runBatchActive(true)}
								>
									批量启用
								</ActionButton>
								<ActionButton
									variant="danger-tertiary"
									size="sm"
									disabled={batchBusy}
									onClick={() => void runBatchActive(false)}
								>
									批量停用
								</ActionButton>
								<ActionButton
									variant="tertiary"
									size="sm"
									disabled={batchBusy}
									onClick={() => setSelectedIds(new Set())}
								>
									取消选择
								</ActionButton>
							</div>
						</section>
					)}
					{filteredItems.length ? (
						<>
							<section
								className="table-scroll"
								// biome-ignore lint/a11y/noNoninteractiveTabindex: horizontal scrolling must be keyboard accessible.
								tabIndex={0}
								aria-label="成员与账号列表"
							>
								<table className="achievement-table">
									<thead>
										<tr>
											<th scope="col" className="col-select">
												<input
													type="checkbox"
													className="table-check"
													aria-label="全选当前页可操作账号"
													disabled={selectablePageItems.length === 0}
													checked={allPageSelected}
													onChange={() =>
														setSelectedIds((prev) => {
															const next = new Set(prev);
															if (allPageSelected) {
																for (const row of selectablePageItems)
																	next.delete(row.id);
															} else {
																for (const row of selectablePageItems)
																	next.add(row.id);
															}
															return next;
														})
													}
												/>
											</th>
											{(
												[
													["username", "账号"],
													["name", "姓名"],
													["department", "部门"],
													["role", "角色"],
													["active", "状态"],
												] as const
											).map(([col, label]) => (
												<th
													key={col}
													scope="col"
													aria-sort={sortCol === col ? sortDir : undefined}
												>
													<button
														type="button"
														className="sortable-th-button"
														onClick={() => toggleSort(col)}
													>
														<span>{label}</span>
														{sortCol === col &&
															(sortDir === "ascending" ? (
																<SortAsc aria-hidden="true" />
															) : (
																<SortDesc aria-hidden="true" />
															))}
													</button>
												</th>
											))}
											<th scope="col">企业微信</th>
											<th scope="col">操作</th>
										</tr>
									</thead>
									<tbody>
										{pageItems.map((account) => {
											const isSelf = account.id === session.user.id;
											return (
												<tr key={account.id}>
													<td data-label="选择" className="col-select">
														<input
															type="checkbox"
															className="table-check"
															aria-label={`选择账号${account.username}`}
															disabled={isSelf}
															checked={!isSelf && selectedIds.has(account.id)}
															onChange={() =>
																setSelectedIds((prev) => {
																	const next = new Set(prev);
																	if (next.has(account.id))
																		next.delete(account.id);
																	else next.add(account.id);
																	return next;
																})
															}
														/>
													</td>
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
											);
										})}
									</tbody>
								</table>
							</section>
							{filteredItems.length > 10 && (
								<Pagination
									page={safePage}
									pageSize={pageSize}
									total={filteredItems.length}
									onChange={setPage}
									onPageSizeChange={(size) => {
										setPageSize(size);
										setPage(1);
									}}
								/>
							)}
						</>
					) : (
						<EmptyState
							icon={<Search />}
							title="没有符合条件的成员账号。"
							description="请尝试更换账号、姓名、部门或企业微信关键词。"
							action={
								<ActionButton variant="secondary" onClick={() => setSearch("")}>
									清空搜索
								</ActionButton>
							}
						/>
					)}
				</section>
			)}
		</>
	);
}
