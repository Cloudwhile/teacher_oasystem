import type { ReactNode } from "react";

/**
 * Refactoring UI §8.4 + §7.3.1 empty state: centered stack with an icon kept at
 * its natural scale inside a soft tinted container, concise heading/copy, and an
 * optional call to action.
 */
export function EmptyState({
	icon,
	title,
	description,
	action,
	compact = false,
}: {
	icon: ReactNode;
	title: string;
	description?: string;
	action?: ReactNode;
	compact?: boolean;
}) {
	return (
		<div className={`empty-state${compact ? " empty-state-compact" : ""}`}>
			<span className="empty-state-icon" aria-hidden="true">
				{icon}
			</span>
			<p className="empty-state-title">{title}</p>
			{description && <p className="empty-state-description">{description}</p>}
			{action && <div className="empty-state-action">{action}</div>}
		</div>
	);
}
