import { useCallback } from "react";
import { useSearchParams } from "react-router-dom";
import { Refresh, Search } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { achievementApi, errorMessage } from "@/features/achievements/api";
import { useResource } from "@/services/useResource";
export function ReviewListPage() {
	const [params, setParams] = useSearchParams();
	const query = params.toString();
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.reviews(query, signal),
		[query],
	);
	const result = useResource(load);
	const catalog = useResource(achievementApi.catalog);
	const filter = (key: string, value: string) => {
		const next = new URLSearchParams(params);
		if (value) next.set(key, value);
		else next.delete(key);
		next.delete("page");
		setParams(next);
	};
	return (
		<>
			<PageHeading title="审核中心" eyebrow="成果管理">
				<IconButton label="刷新待办" onClick={result.reload}>
					<Refresh />
				</IconButton>
			</PageHeading>
			<form
				className="achievement-filters"
				onSubmit={(e) => {
					e.preventDefault();
					const data = new FormData(e.currentTarget);
					const next = new URLSearchParams(params);
					for (const key of ["search", "teacher"]) {
						const value = String(data.get(key) ?? "").trim();
						if (value) next.set(key, value);
						else next.delete(key);
					}
					next.delete("page");
					setParams(next);
				}}
			>
				<div className="search-control">
					<IconButton type="submit" label="搜索审核记录">
						<Search />
					</IconButton>
					<input
						key={params.get("search")}
						name="search"
						aria-label="成果名称"
						placeholder="成果名称"
						defaultValue={params.get("search") ?? ""}
						maxLength={100}
					/>
				</div>
				<input
					key={params.get("teacher")}
					name="teacher"
					aria-label="填报教师"
					placeholder="填报教师"
					defaultValue={params.get("teacher") ?? ""}
					maxLength={100}
				/>
				<select
					aria-label="办理状态"
					value={params.get("status") ?? "pending"}
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
			{result.loading ? (
				<Feedback loading message="正在读取审核记录…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<section className="panel">
					{result.data.items.length ? (
						<AchievementTable items={result.data.items} />
					) : (
						<p className="empty-message">暂无符合条件的记录</p>
					)}
					<Pagination
						page={result.data.page}
						pageSize={result.data.page_size}
						total={result.data.total}
						onChange={(page) => {
							const next = new URLSearchParams(params);
							next.set("page", String(page));
							setParams(next);
						}}
					/>
				</section>
			)}
		</>
	);
}
