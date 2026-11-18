import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useLocation, useSearchParams } from "react-router-dom";
import { ArrowUpRight, Refresh, Search, Users, X } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { PermissionDenied } from "@/components/common/PermissionDenied";
import { subjectLabels } from "@/components/common/ProfileFields";
import { TableSkeleton } from "@/components/common/Skeleton";
import { errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { roleLabels } from "@/features/dashboard/DashboardPage";
import { useResource } from "@/services/useResource";
import { teacherDirectoryApi } from "./api";

function formatRoles(roles: string[]) {
	const display = roles.filter((r) => r !== "teacher");
	const effective = display.length ? display : ["teacher"];
	return effective.map((role) => roleLabels[role] ?? role).join(" / ");
}

export function TeacherDirectoryPage() {
	const { session } = useAuth();
	const canAccess = hasPermission(session?.user, "teachers:directory");
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

	const query = params.toString();
	const load = useCallback(
		(signal: AbortSignal) =>
			canAccess
				? teacherDirectoryApi.list(query, signal)
				: Promise.resolve({
						departments: [],
						items: [],
						page: 1,
						page_size: 20,
						total: 0,
					}),
		[canAccess, query],
	);
	const result = useResource(load);

	if (!canAccess) {
		return (
			<PermissionDenied
				title="教师资料"
				message="仅系统管理员与校级行政可查阅全校教师资料"
			/>
		);
	}

	const commitParams = (updater: (next: URLSearchParams) => void) => {
		if (debounceRef.current) {
			clearTimeout(debounceRef.current);
			debounceRef.current = null;
		}
		const next = new URLSearchParams(params);
		updater(next);
		setParams(next);
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
			for (const key of Array.from(next.keys())) next.delete(key);
		});
	};

	const hasFilters = Boolean(
		params.get("search") ||
			params.get("department_id") ||
			params.get("subject_code") ||
			params.get("employment_status"),
	);
	const from = location.pathname + location.search;

	return (
		<>
			<PageHeading title="教师资料" eyebrow="全校教师名录与成果档案总览">
				<IconButton
					label="刷新教师名录"
					disabled={result.loading || result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
			</PageHeading>
			<form
				className="achievement-filters directory-filters"
				onSubmit={(event) => {
					event.preventDefault();
					filter("search", searchInput.trim());
				}}
			>
				<div className="search-control">
					<IconButton label="搜索教师" type="submit">
						<Search />
					</IconButton>
					<input
						maxLength={100}
						name="search"
						aria-label="搜索教师姓名、工号或部门"
						placeholder="搜索姓名、工号或部门"
						value={searchInput}
						onChange={(event) => handleSearchChange(event.target.value)}
					/>
					{searchInput && (
						<IconButton
							label="清空教师搜索"
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
					aria-label="筛选所属部门"
					value={params.get("department_id") ?? ""}
					onChange={(event) => filter("department_id", event.target.value)}
				>
					<option value="">全部部门</option>
					{result.data?.departments.map((dept) => (
						<option key={dept.id} value={dept.id}>
							{dept.name}
						</option>
					))}
				</select>
				<select
					aria-label="筛选任教学科"
					value={params.get("subject_code") ?? ""}
					onChange={(event) => filter("subject_code", event.target.value)}
				>
					<option value="">全部学科</option>
					{Object.entries(subjectLabels).map(([code, label]) => (
						<option key={code} value={code}>
							{label}
						</option>
					))}
				</select>
				<select
					aria-label="筛选在职状态"
					value={params.get("employment_status") ?? ""}
					onChange={(event) => filter("employment_status", event.target.value)}
				>
					<option value="">全部状态</option>
					<option value="active">在职</option>
					<option value="inactive">离职</option>
				</select>
			</form>
			{result.loading && !result.data ? (
				<TableSkeleton message="正在读取全校教师资料…" />
			) : result.error ? (
				<Feedback
					message={errorMessage(result.error, "教师资料暂时无法读取。")}
					onRetry={result.reload}
				/>
			) : (
				result.data && (
					<section className="panel">
						<div className="section-heading">
							<h2>教师名录</h2>
							<span className="section-meta">共 {result.data.total} 人</span>
						</div>
						{result.data.items.length ? (
							<section
								className="table-scroll"
								// biome-ignore lint/a11y/noNoninteractiveTabindex: horizontal scrolling must be keyboard accessible.
								tabIndex={0}
								aria-label="教师资料列表"
							>
								<table className="achievement-table">
									<thead>
										<tr>
											<th scope="col">教师</th>
											<th scope="col">部门与学科</th>
											<th scope="col">角色</th>
											<th scope="col">状态</th>
											<th scope="col">成果与材料</th>
											<th scope="col">
												<span className="visually-hidden">操作</span>
											</th>
										</tr>
									</thead>
									<tbody>
										{result.data.items.map((teacher) => (
											<tr key={teacher.id}>
												<td data-label="教师">
													<div>
														<Link
															className="record-title"
															to={`/teachers/${teacher.id}`}
															state={{ from }}
														>
															{teacher.name}
														</Link>
														<small className="cell-sub">
															{teacher.employee_no}
														</small>
													</div>
												</td>
												<td data-label="部门与学科">
													<div>
														<span>{teacher.department_name}</span>
														<small className="cell-sub">
															{subjectLabels[teacher.subject_code] ??
																teacher.subject_code}
														</small>
													</div>
												</td>
												<td data-label="角色">{formatRoles(teacher.roles)}</td>
												<td data-label="状态">
													<div className="status-group">
														<span
															className={`status-badge${teacher.employment_status === "active" ? " status-badge-success" : ""}`}
														>
															{teacher.employment_status === "active"
																? "在职"
																: "离职"}
														</span>
														{teacher.account_status !== "active" && (
															<span className="status-badge status-badge-danger">
																已停用
															</span>
														)}
													</div>
												</td>
												<td data-label="成果与材料">
													<div>
														<span>
															{teacher.achievement_count} 项
															{teacher.approved_count > 0
																? `（已认定 ${teacher.approved_count}）`
																: ""}
														</span>
														<small className="cell-sub">
															证明材料 {teacher.material_count} 份
														</small>
													</div>
												</td>
												<td data-label="操作">
													<div className="row-actions">
														<Link
															className="icon-button"
															to={`/teachers/${teacher.id}`}
															state={{ from }}
															aria-label={`查看${teacher.name}的资料`}
															title="查看教师资料"
														>
															<ArrowUpRight />
														</Link>
													</div>
												</td>
											</tr>
										))}
									</tbody>
								</table>
							</section>
						) : (
							<EmptyState
								icon={hasFilters ? <Search /> : <Users />}
								title="没有符合条件的教师记录。"
								description={
									hasFilters
										? "请尝试调整姓名、工号、部门或学科筛选条件。"
										: "当前系统中暂无教师档案记录。"
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
				)
			)}
		</>
	);
}
