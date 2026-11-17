import type { CSSProperties } from "react";

const dashboardStatKeys = ["s1", "s2", "s3", "s4", "s5", "s6"];
const dashboardRowKeys = ["r1", "r2", "r3", "r4"];
const detailFieldKeys = ["d1", "d2", "d3", "d4"];
const formFieldKeys = ["f1", "f2", "f3"];

function Bar({
	width = "100%",
	height = 14,
}: {
	width?: string | number;
	height?: number;
}) {
	const style: CSSProperties = { width, height };
	return <div className="skeleton-block" style={style} aria-hidden="true" />;
}

export function DashboardSkeleton({
	message = "正在读取工作台…",
}: {
	message?: string;
}) {
	return (
		<div className="skeleton-stack" role="status" aria-busy="true">
			<span className="visually-hidden">{message}</span>
			<div className="dashboard-stats" aria-hidden="true">
				{dashboardStatKeys.map((slotKey) => (
					<div className="summary-item" key={slotKey}>
						<Bar width="56%" height={12} />
						<Bar width="40%" height={24} />
					</div>
				))}
			</div>
			<div className="panel skeleton-rows" aria-hidden="true">
				<Bar width="120px" height={18} />
				{dashboardRowKeys.map((rowKey) => (
					<div className="skeleton-row" key={rowKey}>
						<Bar width="78%" height={14} />
						<Bar width="60%" height={14} />
						<Bar width="55%" height={14} />
						<Bar width="68%" height={14} />
						<Bar width={32} height={24} />
					</div>
				))}
			</div>
		</div>
	);
}

export function TableSkeleton({
	message = "正在加载列表…",
	rows = 5,
}: {
	message?: string;
	rows?: number;
}) {
	const rowKeys = Array.from(
		{ length: rows },
		(_, index) => `table-row-${index + 1}`,
	);
	return (
		<section className="panel skeleton-rows" role="status" aria-busy="true">
			<span className="visually-hidden">{message}</span>
			<div className="section-heading" aria-hidden="true">
				<Bar width="112px" height={18} />
				<Bar width="64px" height={12} />
			</div>
			{rowKeys.map((rowKey) => (
				<div className="skeleton-row" key={rowKey} aria-hidden="true">
					<Bar width="80%" height={14} />
					<Bar width="64%" height={14} />
					<Bar width="58%" height={14} />
					<Bar width="70%" height={14} />
					<Bar width={32} height={24} />
				</div>
			))}
		</section>
	);
}

export function DetailSkeleton({
	message = "正在读取详情…",
}: {
	message?: string;
}) {
	return (
		<div className="skeleton-stack" role="status" aria-busy="true">
			<span className="visually-hidden">{message}</span>
			<div className="panel skeleton-rows" aria-hidden="true">
				<Bar width="42%" height={22} />
				<Bar width="26%" height={14} />
				<div className="achievement-fields">
					{detailFieldKeys.map((fieldKey) => (
						<div key={fieldKey}>
							<Bar width="32%" height={12} />
							<div style={{ marginTop: 8 }}>
								<Bar width="72%" height={16} />
							</div>
						</div>
					))}
				</div>
			</div>
			<div className="panel skeleton-rows" aria-hidden="true">
				<Bar width="108px" height={18} />
				<Bar width="65%" height={14} />
				<Bar width="48%" height={14} />
			</div>
		</div>
	);
}

export function FormSkeleton({
	message = "正在读取表单…",
}: {
	message?: string;
}) {
	return (
		<div
			className="panel settings-form skeleton-rows"
			role="status"
			aria-busy="true"
		>
			<span className="visually-hidden">{message}</span>
			<Bar width="136px" height={18} />
			{formFieldKeys.map((fieldKey) => (
				<div className="form-field" key={fieldKey} aria-hidden="true">
					<Bar width="96px" height={14} />
					<Bar width="100%" height={38} />
				</div>
			))}
		</div>
	);
}
