import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useLocation, useSearchParams } from "react-router-dom";
import { FileText, Plus, Refresh, Search, X } from "reicon-react";
import { AchievementTable } from "@/components/achievements/AchievementTable";
import { ActionButton } from "@/components/common/ActionButton";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { TableSkeleton } from "@/components/common/Skeleton";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { useResource } from "@/services/useResource";
import { achievementApi, errorMessage, statusLabels } from "./api";

export function AchievementListPage() {
	const { session } = useAuth();
	const [params, setParams] = useSearchParams();
	const location = useLocation();
	const urlSearch = params.get("search") ?? "";
	const [searchInput, setSearchInput] = useState(urlSearch);
	const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	useEffect(() => {
		setSearchInput(urlSearch);
	}, [urlSearch]);

	useEffect(
		() => () => {
			if (debounceRef.current) clearTimeout(debounceRef.current);
		},
		[],
	);

	const queryParams = new URLSearchParams(params);
	if (!queryParams.has("scope")) queryParams.set("scope", "mine");
	const query = queryParams.toString();
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.list(query, signal),
		[query],
	);
	const isAllScope = params.get("scope") === "all";
	const canFilterPeople =
		isAllScope &&
		Boolean(
			session?.user.grants.some(
				(g) =>
					(g.permission === "achievements:read" ||
						g.permission === "teachers:read" ||
						g.permission === "teachers:directory") &&
					g.scope !== "SELF",
			),
		);
	const loadPeople = useCallback(
		(signal: AbortSignal) =>
			canFilterPeople
				? achievementApi.scopePeople(signal)
				: Promise.resolve({ departments: [], teachers: [] }),
		[canFilterPeople],
	);
	const catalog = useResource(achievementApi.catalog);
	const people = useResource(loadPeople);
	const result = useResource(load);

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

	const filter = (key: string, value: string) => {
		const trimmedSearch = searchInput.trim();
		commitParams((next) => {
			if (key !== "search") {
				if (trimmedSearch) next.set("search", trimmedSearch);
				else next.delete("search");
			}
			if (value) next.set(key, value);
			else next.delete(key);
			next.delete("page");
		});
	};

	const handleSearchChange = (value: string) => {
		setSearchInput(value);
		if (debounceRef.current) clearTimeout(debounceRef.current);
		debounceRef.current = setTimeout(() => {
			debounceRef.current = null;
			const trimmed = value.trim();
			if (trimmed === (params.get("search") ?? "")) return;
			const next = new URLSearchParams(params);
			if (trimmed) next.set("search", trimmed);
			else next.delete("search");
			next.delete("page");
			setParams(next);
		}, 350);
	};

	const clearFilters = () => {
		setSearchInput("");
		commitParams((next) => {
			const keepAll = next.get("scope") === "all";
			for (const key of Array.from(next.keys())) next.delete(key);
			if (keepAll) next.set("scope", "all");
		});
	};

	const hasFilters = Boolean(
		params.get("search") ||
			params.get("department_id") ||
			params.get("teacher_id") ||
			params.get("type_code") ||
			params.get("academic_term_id") ||
			params.get("status") ||
			params.get("level_code") ||
			(params.get("sort") && params.get("sort") !== "updated_at_desc"),
	);
	const canWrite = hasPermission(session?.user, "achievements:write");
	const showHeaderCreate =
		canWrite &&
		!result.loading &&
		(Boolean(result.data?.items.length) || hasFilters || Boolean(result.error));
	const isQuietEmpty =
		!isAllScope && !hasFilters && !result.loading && result.data?.total === 0;

	return (
		<>
			<PageHeading
				title={params.get("scope") === "all" ? "成果查询" : "我的成果"}
			>
				<IconButton
					label="刷新成果列表"
					disabled={result.loading || result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
				{showHeaderCreate && (
					<Link
						to="/achievements/new"
						state={{ from: location.pathname + location.search }}
						className="action-button action-button-primary"
					>
						<Plus aria-hidden="true" />
						新建成果
					</Link>
				)}
			</PageHeading>
			<form
				className={`achievement-filters ${canFilterPeople ? "with-people" : "with-scope"}${isQuietEmpty ? " is-quiet-empty" : ""}`}
				onSubmit={(event) => {
					event.preventDefault();
					filter("search", searchInput.trim());
				}}
			>
				<div className="search-control">
					<IconButton label="搜索成果" type="submit">
						<Search />
					</IconButton>
					<input
						maxLength={100}
						name="search"
						aria-label="搜索成果名称"
						placeholder={
							canFilterPeople ? "搜索成果名称、教师姓名或工号" : "搜索成果名称"
						}
						value={searchInput}
						onChange={(event) => handleSearchChange(event.target.value)}
					/>
					{searchInput && (
						<IconButton
							label="清空搜索"
							onClick={() => {
								setSearchInput("");
								filter("search", "");
							}}
						>
							<X />
						</IconButton>
					)}
				</div>
				<select
					aria-label="成果范围"
					value={params.get("scope") ?? "mine"}
					onChange={(e) => {
						const nextScope = e.target.value;
						commitParams((next) => {
							if (nextScope) next.set("scope", nextScope);
							else next.delete("scope");
							if (nextScope !== "all") {
								next.delete("department_id");
								next.delete("teacher_id");
							}
							next.delete("page");
						});
					}}
				>
					<option value="mine">我的成果</option>
					<option value="all">可查看的全部成果</option>
				</select>
				{canFilterPeople && (
					<>
						<select
							aria-label="筛选部门"
							value={params.get("department_id") ?? ""}
							onChange={(event) => {
								const deptId = event.target.value;
								commitParams((next) => {
									if (deptId) next.set("department_id", deptId);
									else next.delete("department_id");
									next.delete("teacher_id");
									next.delete("page");
								});
							}}
						>
							<option value="">全部部门</option>
							{people.data?.departments.map((dept) => (
								<option key={dept.id} value={dept.id}>
									{dept.name}
								</option>
							))}
						</select>
						<select
							aria-label="筛选教师"
							value={params.get("teacher_id") ?? ""}
							onChange={(event) => filter("teacher_id", event.target.value)}
						>
							<option value="">全部教师</option>
							{people.data?.teachers
								.filter(
									(t) =>
										!params.get("department_id") ||
										String(t.department_id) === params.get("department_id"),
								)
								.map((teacher) => (
									<option key={teacher.id} value={teacher.id}>
										{teacher.name}
									</option>
								))}
						</select>
					</>
				)}
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
				<select
					aria-label="排序方式"
					value={params.get("sort") ?? "updated_at_desc"}
					onChange={(event) =>
						filter(
							"sort",
							event.target.value === "updated_at_desc"
								? ""
								: event.target.value,
						)
					}
				>
					<option value="updated_at_desc">按最近更新</option>
					<option value="occurred_at_desc">按成果日期</option>
				</select>
			</form>
			{catalog.error && (
				<Feedback message="筛选项暂时无法读取。" onRetry={catalog.reload} />
			)}
			{result.loading && !result.data ? (
				<TableSkeleton message="正在读取成果…" />
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
							<AchievementTable
								items={result.data.items}
								hideTeacher={!isAllScope}
							/>
						) : (
							<EmptyState
								icon={hasFilters ? <Search /> : <FileText />}
								title="没有符合条件的成果。"
								description={
									hasFilters
										? "请尝试调整筛选条件，或清除条件后查看全部成果。"
										: "当前还没有成果记录，点击新建开始填报。"
								}
								action={
									hasFilters ? (
										<ActionButton variant="secondary" onClick={clearFilters}>
											清除筛选
										</ActionButton>
									) : canWrite ? (
										<Link
											to="/achievements/new"
											state={{ from: location.pathname + location.search }}
											className="action-button action-button-primary"
										>
											新建成果
										</Link>
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
				)
			)}
		</>
	);
}
