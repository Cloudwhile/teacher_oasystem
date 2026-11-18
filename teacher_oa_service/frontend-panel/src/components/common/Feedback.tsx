import { AlertCircle, Loader, Refresh } from "reicon-react";
import { ActionButton } from "./ActionButton";

export function Feedback({
	message,
	loading = false,
	onRetry,
}: {
	message: string;
	loading?: boolean;
	onRetry?: () => void;
}) {
	if (loading) {
		return (
			<div className="feedback feedback-loading" role="status">
				<Loader className="spin" aria-hidden="true" />
				<p>{message}</p>
			</div>
		);
	}
	return (
		<div className="feedback" role="alert">
			<div className="feedback-body">
				<AlertCircle aria-hidden="true" />
				<p>{message}</p>
			</div>
			{onRetry && (
				<ActionButton variant="secondary" size="sm" onClick={onRetry}>
					<Refresh aria-hidden="true" />
					重新加载
				</ActionButton>
			)}
		</div>
	);
}
