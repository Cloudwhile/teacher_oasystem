import { Link, useLocation } from "react-router-dom";
import { ArrowUpRight } from "reicon-react";
import { statusLabels } from "@/features/achievements/api";
import type { components } from "@/types/api.generated";
export function AchievementTable({
	items,
}: {
	items: components["schemas"]["AchievementItem"][];
}) {
	const location = useLocation();
	const from = location.pathname + location.search;
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
						<th>成果名称</th>
						<th>类型</th>
						<th>填报教师</th>
						<th>日期</th>
						<th>状态</th>
						<th>
							<span className="visually-hidden">操作</span>
						</th>
					</tr>
				</thead>
				<tbody>
					{items.map((item) => (
						<tr key={item.id}>
							<td>
								<Link
									className="record-title"
									to={`/achievements/${item.id}`}
									state={{ from }}
								>
									{item.title}
								</Link>
							</td>
							<td>{item.type_name}</td>
							<td>{item.teacher_name}</td>
							<td>{item.occurred_at ?? "—"}</td>
							<td>
								<span className={`achievement-status status-${item.status}`}>
									{statusLabels[item.status]}
								</span>
							</td>
							<td>
								<Link
									className="icon-button"
									to={`/achievements/${item.id}`}
									state={{ from }}
									aria-label={`查看${item.title}`}
								>
									<ArrowUpRight />
								</Link>
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</section>
	);
}
