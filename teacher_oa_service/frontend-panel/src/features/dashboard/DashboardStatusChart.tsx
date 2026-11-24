import {
	Bar,
	BarChart,
	Cell,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

export type StatusChartDatum = {
	status: string;
	name: string;
	count: number;
};

const statusChartColors: Record<string, string> = {
	draft: "var(--color-neutral-500)",
	pending: "var(--color-warning)",
	returned: "var(--color-danger-fg)",
	approved: "var(--color-success)",
	archived: "var(--color-info)",
};

export default function DashboardStatusChart({
	data,
	totalCount,
}: {
	data: StatusChartDatum[];
	totalCount: number;
}) {
	return (
		<section className="panel status-chart-panel" aria-label="我的成果状态分布">
			<div className="section-heading">
				<h2>状态分布概览</h2>
				<span className="section-meta">共 {totalCount} 项个人成果</span>
			</div>
			<div className="status-chart-box" aria-hidden="true">
				<ResponsiveContainer width="100%" height="100%">
					<BarChart
						data={data}
						layout="vertical"
						margin={{ top: 0, right: 20, bottom: 0, left: 8 }}
					>
						<XAxis type="number" allowDecimals={false} hide />
						<YAxis
							type="category"
							dataKey="name"
							width={64}
							axisLine={false}
							tickLine={false}
							tick={{
								fill: "var(--color-text-secondary)",
								fontSize: 12,
							}}
						/>
						<Tooltip
							cursor={{ fill: "var(--color-surface-subtle)" }}
							contentStyle={{
								background: "var(--color-surface-overlay)",
								borderColor: "var(--color-border)",
								borderRadius: "6px",
								color: "var(--color-text)",
								fontSize: "12px",
							}}
							formatter={(value) => [`${String(value)} 项`, "数量"]}
						/>
						<Bar dataKey="count" radius={[0, 4, 4, 0]} barSize={16}>
							{data.map((entry) => (
								<Cell
									key={entry.status}
									fill={statusChartColors[entry.status]}
								/>
							))}
						</Bar>
					</BarChart>
				</ResponsiveContainer>
			</div>
			<ul className="visually-hidden">
				{data.map((entry) => (
					<li key={entry.status}>
						{entry.name}：{entry.count} 项
					</li>
				))}
			</ul>
		</section>
	);
}
