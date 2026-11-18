import type { ReactNode } from "react";
import { GraduationCap } from "reicon-react";

export type StatusTone = "neutral" | "warning" | "danger";

export function StatusStateCard({
	icon,
	tone = "neutral",
	code,
	title,
	description,
	actions,
	headingLevel = "h1",
	framed = false,
	role,
}: {
	icon: ReactNode;
	tone?: StatusTone;
	code?: string;
	title: string;
	description?: string;
	actions?: ReactNode;
	headingLevel?: "h1" | "h2";
	framed?: boolean;
	role?: "alert" | "status";
}) {
	const Heading = headingLevel;
	return (
		<section
			className={`status-state${framed ? " status-state-card" : ""}`}
			role={role}
		>
			<span
				className={`status-state-icon status-state-icon-${tone}`}
				aria-hidden="true"
			>
				{icon}
			</span>
			{code && <p className="status-state-code">{code}</p>}
			<Heading className="status-state-title">{title}</Heading>
			{description && <p className="status-state-description">{description}</p>}
			{actions && <div className="status-state-actions">{actions}</div>}
		</section>
	);
}

export function StandaloneStatusShell({
	children,
	loading = false,
	loadingLabel = "正在加载…",
}: {
	children?: ReactNode;
	loading?: boolean;
	loadingLabel?: string;
}) {
	return (
		<div className="standalone-status-shell">
			<header className="public-header">
				<div className="public-container public-brand">
					<GraduationCap aria-hidden="true" />
					<span>TeacherOA</span>
				</div>
			</header>
			<main className="standalone-status-main">
				{loading ? (
					<div className="standalone-loading" role="status">
						<span className="standalone-spinner" aria-hidden="true" />
						<p>{loadingLabel}</p>
					</div>
				) : (
					children
				)}
			</main>
			<footer className="public-footer">
				<div className="public-container">TeacherOA · 教师工作空间</div>
			</footer>
		</div>
	);
}
