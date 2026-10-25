import { useCallback } from "react";
import { LuArrowUpRight, LuPlus, LuSearch } from "react-icons/lu";
import { Link, useSearchParams } from "react-router-dom";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { useResource } from "@/services/useResource";
import { achievementApi, errorMessage, statusLabels } from "./api";

export function AchievementListPage() {
	const [params, setParams] = useSearchParams();
	const query = params.toString();
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
			<PageHeading title="我的成果" eyebrow="成果管理">
				<Link
					to="/achievements/new"
					className="icon-button primary-action"
					aria-label="新建成果"
					title="新建成果"
				>
					<LuPlus />
				</Link>
			</PageHeading>
			<form
				className="achievement-filters"
				onSubmit={(event) => {
					event.preventDefault();
					const data = new FormData(event.currentTarget);
					filter("search", String(data.get("search") ?? ""));
				}}
			>
				<div className="search-control">
					<IconButton label="搜索成果" type="submit">
						<LuSearch />
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
							<div className="table-scroll">
								<table>
									<thead>
										<tr>
											<th scope="col">成果名称</th>
											<th scope="col">成果类型</th>
											<th scope="col">填报教师</th>
											<th scope="col">日期</th>
											<th scope="col">状态</th>
											<th scope="col">
												<span className="visually-hidden">操作</span>
											</th>
										</tr>
									</thead>
									<tbody>
										{result.data.items.map((item) => (
											<tr key={item.id}>
												<td>
													<Link
														className="record-title"
														to={`/achievements/${item.id}`}
													>
														{item.title}
													</Link>
												</td>
												<td>{item.type_name}</td>
												<td>{item.teacher_name}</td>
												<td>{item.occurred_at ?? "—"}</td>
												<td>
													<span
														className={`achievement-status status-${item.status}`}
													>
														{statusLabels[item.status]}
													</span>
												</td>
												<td>
													<Link
														className="icon-button"
														to={`/achievements/${item.id}`}
														aria-label={`查看${item.title}`}
													>
														<LuArrowUpRight />
													</Link>
												</td>
											</tr>
										))}
									</tbody>
								</table>
							</div>
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
