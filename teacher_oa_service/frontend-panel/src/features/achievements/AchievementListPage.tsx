import { useCallback } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { Plus, Search } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { useResource } from "@/services/useResource";
import { achievementApi, errorMessage, statusLabels } from "./api";

export function AchievementListPage() {
	const [params, setParams] = useSearchParams();
	const queryParams = new URLSearchParams(params);
	if (!queryParams.has("scope")) queryParams.set("scope", "mine");
	const query = queryParams.toString();
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.list(query, signal),
		[query],
	);
	const catalog = useResource(achievementApi.catalog);
	const result = useResource(load);
	const filter = (key: string, value: string) => {
		const next = new URLSearchParams(params);
		if (value) next.set(key, value);
		else next.delete(key);
		next.delete("page");
		setParams(next);
	};
	return (
		<>
			<PageHeading
				title={params.get("scope") === "all" ? "成果查询" : "我的成果"}
				eyebrow="成果管理"
			>
				<Link
					to="/achievements/new"
					className="icon-button primary-action"
					aria-label="新建成果"
					title="新建成果"
				>
					<Plus />
				</Link>
			</PageHeading>
			<form
				className="achievement-filters with-scope"
				onSubmit={(event) => {
					event.preventDefault();
					const data = new FormData(event.currentTarget);
					filter("search", String(data.get("search") ?? ""));
				}}
			>
				<select
					aria-label="成果范围"
					value={params.get("scope") ?? "mine"}
					onChange={(e) => filter("scope", e.target.value)}
				>
					<option value="mine">我的成果</option>
					<option value="all">可查看的全部成果</option>
				</select>
				<div className="search-control">
					<IconButton label="搜索成果" type="submit">
						<Search />
					</IconButton>
					<input
						key={params.get("search") ?? ""}
						maxLength={100}
						name="search"
						aria-label="搜索成果名称"
						placeholder="搜索成果名称"
						defaultValue={params.get("search") ?? ""}
					/>
				</div>
				<select
					aria-label="筛选成果类型"
					value={params.get("type_code") ?? ""}
					onChange={(event) => filter("type_code", event.target.value)}
				>
					<option value="">全部类型</option>
					{catalog.data?.types.map((item) => (
						<option key={item.code} value={item.code}>
							{item.name}
						</option>
					))}
				</select>
				<select
					aria-label="筛选学期"
					value={params.get("academic_term_id") ?? ""}
					onChange={(event) => filter("academic_term_id", event.target.value)}
				>
					<option value="">全部学期</option>
					{catalog.data?.terms.map((item) => (
						<option key={item.id} value={item.id}>
							{item.name}
						</option>
					))}
				</select>
				<select
					aria-label="筛选状态"
					value={params.get("status") ?? ""}
					onChange={(event) => filter("status", event.target.value)}
				>
					<option value="">全部状态</option>
					{Object.entries(statusLabels).map(([code, label]) => (
						<option key={code} value={code}>
							{label}
						</option>
					))}
				</select>
				<select
					aria-label="筛选级别"
					value={params.get("level_code") ?? ""}
					onChange={(event) => filter("level_code", event.target.value)}
				>
					<option value="">全部级别</option>
					{catalog.data?.dictionaries
						.filter((item) => item.group_code === "achievement_levels")
						.map((item) => (
							<option key={item.code} value={item.code}>
								{item.name}
							</option>
						))}
				</select>
			</form>
			{catalog.error && (
				<Feedback message="筛选项暂时无法读取。" onRetry={catalog.reload} />
			)}
			{result.loading ? (
				<Feedback loading message="正在读取成果…" />
			) : result.error ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				result.data && (
					<section className="panel">
						<div className="section-heading">
							<h2>成果记录</h2>
							<span className="section-meta">共 {result.data.total} 条</span>
						</div>
						{result.data.items.length ? (
							<AchievementTable items={result.data.items} />
						) : (
							<p className="empty-message">没有符合条件的成果。</p>
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
				)
			)}
		</>
	);
}
