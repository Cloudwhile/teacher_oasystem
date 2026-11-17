import { useCallback, useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Check, Refresh, Search, X } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { ActionButton } from "@/components/common/ActionButton";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { FormField } from "@/components/common/FormField";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { PermissionDenied } from "@/components/common/PermissionDenied";
import { TableSkeleton } from "@/components/common/Skeleton";
import { achievementApi, errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";
import { useResource } from "@/services/useResource";

type BatchActionKind = "approve" | "return" | "archive" | null;

export function ReviewListPage() {
	const { session, refresh } = useAuth();
	const [params, setParams] = useSearchParams();
	const urlSearch = params.get("search") ?? "";
	const urlTeacher = params.get("teacher") ?? "";
	const [searchInput, setSearchInput] = useState(urlSearch);
	const [teacherInput, setTeacherInput] = useState(urlTeacher);
	const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
	const [batchAction, setBatchAction] = useState<BatchActionKind>(null);
	const [batchOpinion, setBatchOpinion] = useState("");
	const [batchError, setBatchError] = useState("");
	const [batchBusy, setBatchBusy] = useState(false);

	useEffect(() => {
		setSearchInput(urlSearch);
	}, [urlSearch]);

	useEffect(() => {
		setTeacherInput(urlTeacher);
	}, [urlTeacher]);

	useEffect(
		() => () => {
			if (debounceRef.current) clearTimeout(debounceRef.current);
		},
		[],
	);

	const query = params.toString();
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.reviews(query, signal),
		[query],
	);
	const result = useResource(load);
	const catalog = useResource(achievementApi.catalog);

	const currentStatus = params.get("status") ?? "pending";
	const batchCapable =
		currentStatus === "pending" || currentStatus === "approved";

	const [lastQuery, setLastQuery] = useState(query);
	if (lastQuery !== query) {
		setLastQuery(query);
		if (selectedIds.size > 0) {
			setSelectedIds(new Set());
		}
	}

	const commitParams = (
		updater: (next: URLSearchParams) => void,
		replace = false,
	) => {
		if (debounceRef.current) {
			clearTimeout(debounceRef.current);
			debounceRef.current = null;
		}
		const next = new URLSearchParams(params);
		updater(next);
		setParams(next, { replace });
	};

	const scheduleTextSync = (nextSearch: string, nextTeacher: string) => {
		if (debounceRef.current) clearTimeout(debounceRef.current);
		debounceRef.current = setTimeout(() => {
			debounceRef.current = null;
			const s = nextSearch.trim();
			const t = nextTeacher.trim();
			if (
				s === (params.get("search") ?? "") &&
				t === (params.get("teacher") ?? "")
			) {
				return;
			}
			const next = new URLSearchParams(params);
			if (s) next.set("search", s);
			else next.delete("search");
			if (t) next.set("teacher", t);
			else next.delete("teacher");
			next.delete("page");
			setParams(next);
		}, 350);
	};

	const filter = (key: string, value: string) => {
		const s = searchInput.trim();
		const t = teacherInput.trim();
		commitParams((next) => {
			if (s) next.set("search", s);
			else next.delete("search");
			if (t) next.set("teacher", t);
			else next.delete("teacher");
			if (value) next.set(key, value);
			else next.delete(key);
			next.delete("page");
		});
	};

	const clearFilters = () => {
		setSearchInput("");
		setTeacherInput("");
		commitParams((next) => {
			for (const key of Array.from(next.keys())) next.delete(key);
		});
	};

	const hasFilters = Boolean(
		params.get("search") ||
			params.get("teacher") ||
			params.get("type_code") ||
			params.get("academic_term_id") ||
			(params.get("status") && params.get("status") !== "pending"),
	);

	const runBatch = async () => {
		if (!batchAction || !session || !result.data || batchBusy) return;
		if (batchAction === "return" && !batchOpinion.trim()) {
			setBatchError("请填写退回补正意见");
			return;
		}
		const targets = result.data.items.filter((item) =>
			selectedIds.has(item.id),
		);
		if (!targets.length) {
			setBatchAction(null);
			return;
		}
		setBatchBusy(true);
		setBatchError("");
		let succeeded = 0;
		const failedTitles: string[] = [];
		try {
			for (const item of targets) {
				try {
					await achievementApi.review(
						item.id,
						batchAction,
						item.version,
						batchOpinion,
						session.csrf_token,
					);
					succeeded += 1;
				} catch (err) {
					if (err instanceof HttpError && err.status === 401) {
						void refresh();
						return;
					}
					failedTitles.push(`${item.title}（${errorMessage(err)}）`);
				}
			}
			setSelectedIds(new Set());
			setBatchAction(null);
			setBatchOpinion("");
			result.reload();
			if (succeeded > 0) {
				const actionName =
					batchAction === "approve"
						? "批量审核通过"
						: batchAction === "return"
							? "批量退回补正"
							: "批量归档";
				notifySuccess(
					`${actionName}已完成 ${succeeded} 项`,
					failedTitles.length
						? `另有 ${failedTitles.length} 项未能完成`
						: undefined,
				);
			}
			if (failedTitles.length > 0) {
				setBatchError(`部分记录未完成：${failedTitles.join("；")}`);
			}
		} finally {
			setBatchBusy(false);
		}
	};

	if (result.error instanceof HttpError && result.error.status === 403) {
		return (
			<PermissionDenied
				title="审核中心"
				message="当前账号没有审核权限，无法查看审核待办。"
			/>
		);
	}

	const items = result.data?.items ?? [];

	return (
		<>
			<PageHeading title="审核中心">
				<IconButton
					label="刷新待办"
					disabled={result.loading || result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
			</PageHeading>
			<form
				className="achievement-filters"
				onSubmit={(e) => {
					e.preventDefault();
					commitParams((next) => {
						const s = searchInput.trim();
						const t = teacherInput.trim();
						if (s) next.set("search", s);
						else next.delete("search");
						if (t) next.set("teacher", t);
						else next.delete("teacher");
						next.delete("page");
					});
				}}
			>
				<div className="search-control">
					<IconButton type="submit" label="搜索审核记录">
						<Search />
					</IconButton>
					<input
						name="search"
						aria-label="成果名称"
						placeholder="成果名称"
						value={searchInput}
						maxLength={100}
						onChange={(event) => {
							setSearchInput(event.target.value);
							scheduleTextSync(event.target.value, teacherInput);
						}}
					/>
					{searchInput && (
						<IconButton
							label="清空成果名称搜索"
							onClick={() => {
								setSearchInput("");
								commitParams((next) => {
									next.delete("search");
									next.delete("page");
								});
							}}
						>
							<X />
						</IconButton>
					)}
				</div>
				<input
					name="teacher"
					aria-label="填报教师"
					placeholder="填报教师"
					value={teacherInput}
					maxLength={100}
					onChange={(event) => {
						setTeacherInput(event.target.value);
						scheduleTextSync(searchInput, event.target.value);
					}}
				/>
				<select
					aria-label="办理状态"
					value={currentStatus}
					onChange={(e) => filter("status", e.target.value)}
				>
					<option value="pending">待审核</option>
					<option value="approved">待归档</option>
					<option value="returned">已退回</option>
					<option value="archived">已归档</option>
					<option value="history">我的办理历史</option>
				</select>
				<select
					aria-label="筛选成果类型"
					value={params.get("type_code") ?? ""}
					onChange={(e) => filter("type_code", e.target.value)}
				>
					<option value="">全部类型</option>
					{catalog.data?.types.map((t) => (
						<option key={t.code} value={t.code}>
							{t.name}
						</option>
					))}
				</select>
				<select
					aria-label="筛选学期"
					value={params.get("academic_term_id") ?? ""}
					onChange={(e) => filter("academic_term_id", e.target.value)}
				>
					<option value="">全部学期</option>
					{catalog.data?.terms.map((t) => (
						<option key={t.id} value={t.id}>
							{t.name}
						</option>
					))}
				</select>
			</form>
			{catalog.error && (
				<Feedback message="筛选项暂时无法读取" onRetry={catalog.reload} />
			)}
			{batchError && !batchAction && (
				<p className="inline-error" role="alert">
					{batchError}
				</p>
			)}
			{result.loading && !result.data ? (
				<TableSkeleton message="正在读取审核记录…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<section className="panel">
					{batchCapable && selectedIds.size > 0 && (
						<section className="batch-bar" aria-label="批量审核操作">
							<span>已选择 {selectedIds.size} 项记录</span>
							<div className="row-actions">
								{currentStatus === "pending" && (
									<>
										<ActionButton
											variant="primary"
											size="sm"
											onClick={() => {
												setBatchError("");
												setBatchAction("approve");
											}}
										>
											批量通过
										</ActionButton>
										<ActionButton
											variant="secondary"
											size="sm"
											onClick={() => {
												setBatchError("");
												setBatchAction("return");
											}}
										>
											批量退回
										</ActionButton>
									</>
								)}
								{currentStatus === "approved" && (
									<ActionButton
										variant="primary"
										size="sm"
										onClick={() => {
											setBatchError("");
											setBatchAction("archive");
										}}
									>
										批量归档
									</ActionButton>
								)}
								<ActionButton
									variant="tertiary"
									size="sm"
									onClick={() => setSelectedIds(new Set())}
								>
									取消选择
								</ActionButton>
							</div>
						</section>
					)}
					{items.length ? (
						<AchievementTable
							items={items}
							selectable={batchCapable}
							selectedIds={selectedIds}
							onToggleSelect={(id) =>
								setSelectedIds((prev) => {
									const next = new Set(prev);
									if (next.has(id)) next.delete(id);
									else next.add(id);
									return next;
								})
							}
							onToggleSelectAll={() =>
								setSelectedIds((prev) => {
									const allSelected = items.every((item) => prev.has(item.id));
									if (allSelected) return new Set();
									return new Set(items.map((item) => item.id));
								})
							}
						/>
					) : (
						<EmptyState
							icon={hasFilters ? <Search /> : <Check />}
							title="暂无符合条件的记录"
							description={
								hasFilters
									? "请尝试调整搜索或筛选条件。"
									: "当前待办已全部处理完毕。"
							}
							action={
								hasFilters ? (
									<ActionButton variant="secondary" onClick={clearFilters}>
										清除筛选
									</ActionButton>
								) : undefined
							}
						/>
					)}
					<Pagination
						page={result.data.page}
						pageSize={result.data.page_size}
						total={result.data.total}
						onChange={(page) => {
							commitParams((next) => {
								next.set("page", String(page));
							});
						}}
						onPageSizeChange={(size) => {
							commitParams((next) => {
								if (size === 20) next.delete("page_size");
								else next.set("page_size", String(size));
								next.delete("page");
							});
						}}
					/>
				</section>
			)}
			<ConfirmDialog
				open={batchAction !== null}
				busy={batchBusy}
				title={
					batchAction === "approve"
						? `确认批量通过 ${selectedIds.size} 项成果？`
						: batchAction === "return"
							? `确认批量退回 ${selectedIds.size} 项成果？`
							: `确认批量归档 ${selectedIds.size} 项成果？`
				}
				description={
					batchAction === "archive"
						? "归档后成果将锁定为最终档案，无法撤销或再次编辑。"
						: batchAction === "return"
							? "所选成果将统一退回填报教师补正，请填写明确的退回意见。"
							: "通过后所选成果将进入待归档状态。"
				}
				confirmLabel={
					batchAction === "approve"
						? "确认通过"
						: batchAction === "return"
							? "确认退回"
							: "确认归档"
				}
				tone={
					batchAction === "return" || batchAction === "archive"
						? "danger"
						: "primary"
				}
				onCancel={() => {
					if (batchBusy) return;
					setBatchAction(null);
					setBatchError("");
				}}
				onConfirm={() => {
					void runBatch();
				}}
			>
				{batchAction !== null && (
					<FormField
						id="batch-review-opinion"
						label={batchAction === "return" ? "退回意见" : "办理意见（选填）"}
						required={batchAction === "return"}
						error={batchError}
					>
						<textarea
							id="batch-review-opinion"
							rows={3}
							maxLength={2000}
							disabled={batchBusy}
							value={batchOpinion}
							onChange={(event) => {
								if (batchError) setBatchError("");
								setBatchOpinion(event.target.value);
							}}
						/>
					</FormField>
				)}
			</ConfirmDialog>
		</>
	);
}
