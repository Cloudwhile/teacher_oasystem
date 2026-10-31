import { AlertCircle, Loader, Refresh } from "reicon-react";
import { IconButton } from "./IconButton";

export function Feedback({
	message,
	loading = false,
	onRetry,
}: {
	message: string;
	loading?: boolean;
	onRetry?: () => void;
}) {
	return (
		<div className="feedback" role={loading ? "status" : "alert"}>
			{loading ? (
				<Loader className="spin" aria-hidden="true" />
			) : (
				<AlertCircle aria-hidden="true" />
			)}
			<p>{message}</p>
			{onRetry && (
				<IconButton label="重新加载" onClick={onRetry}>
					<Refresh />
				</IconButton>
			)}
		</div>
	);
}
