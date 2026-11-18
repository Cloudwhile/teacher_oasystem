import { Link, useLocation } from "react-router-dom";
import { ArrowUpRight, Pen } from "reicon-react";
import { statusLabels } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import { formatAppDate } from "@/services/dateFormat";
import type { components } from "@/types/api.generated";

export function AchievementTable({
	items,
	hideTeacher = false,
	selectable = false,
	selectedIds,
	onToggleSelect,
	onToggleSelectAll,
}: {
	items: components["schemas"]["AchievementItem"][];
	hideTeacher?: boolean;
	selectable?: boolean;
	selectedIds?: Set<number>;
	onToggleSelect?: (id: number) => void;
	onToggleSelectAll?: () => void;
}) {
	const location = useLocation();
	const { session } = useAuth();
	const canWrite = hasPermission(session?.user, "achievements:write");
	const canReadDirectory = hasPermission(session?.user, "teachers:directory");
	const ownTeacherId = session?.user.teacher?.id;
	const from = location.pathname + location.search;
	const allSelected =
		selectable &&
		items.length > 0 &&
		items.every((item) => selectedIds?.has(item.id));

	return (
		<section
			className="table-scroll"
			// biome-ignore lint/a11y/noNoninteractiveTabindex: horizontal scrolling must be keyboard accessible.
			tabIndex={0}
			aria-label="成果列表"
		>
			<table className="achievement-table">
				<thead>
					<tr>
						{selectable && (
							<th scope="col" className="col-select">
								<input
									type="checkbox"
									className="table-check"
									aria-label="全选当前页记录"
									checked={allSelected}
									onChange={() => onToggleSelectAll?.()}
								/>
							</th>
						)}
						<th scope="col">成果名称</th>
						<th scope="col">类型</th>
						{!hideTeacher && <th scope="col">填报教师</th>}
						<th scope="col">日期</th>
						<th scope="col">状态</th>
						<th scope="col">
							<span className="visually-hidden">操作</span>
						</th>
					</tr>
				</thead>
				<tbody>
					{items.map((item) => {
						const canQuickEdit =
							canWrite &&
							item.status === "returned" &&
							ownTeacherId !== undefined &&
							item.teacher_id === ownTeacherId;
						return (
							<tr key={item.id}>
								{selectable && (
									<td data-label="选择" className="col-select">
										<input
											type="checkbox"
											className="table-check"
											aria-label={`选择${item.title}`}
											checked={Boolean(selectedIds?.has(item.id))}
											onChange={() => onToggleSelect?.(item.id)}
										/>
									</td>
								)}
								<td data-label="成果名称">
									<Link
										className="record-title"
										to={`/achievements/${item.id}`}
										state={{ from }}
									>
										{item.title}
									</Link>
								</td>
								<td data-label="类型">{item.type_name}</td>
								{!hideTeacher && (
									<td data-label="填报教师">
										{canReadDirectory ? (
											<Link
												className="text-link"
												to={`/teachers/${item.teacher_id}`}
												state={{ from }}
											>
												{item.teacher_name}
											</Link>
										) : (
											item.teacher_name
										)}
									</td>
								)}
								<td data-label="日期">
									{item.occurred_at ? formatAppDate(item.occurred_at) : "—"}
								</td>
								<td data-label="状态">
									<span className={`achievement-status status-${item.status}`}>
										{statusLabels[item.status]}
									</span>
								</td>
								<td data-label="操作">
									<div className="row-actions">
										{canQuickEdit && (
											<Link
												className="action-button action-button-secondary action-button-sm"
												to={`/achievements/${item.id}/edit`}
												state={{ from }}
												aria-label={`修改${item.title}`}
											>
												<Pen aria-hidden="true" />
												去修改
											</Link>
										)}
										<Link
											className="icon-button"
											to={`/achievements/${item.id}`}
											state={{ from }}
											aria-label={`查看${item.title}`}
										>
											<ArrowUpRight />
										</Link>
									</div>
								</td>
							</tr>
						);
					})}
				</tbody>
			</table>
		</section>
	);
}
