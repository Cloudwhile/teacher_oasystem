import { LuCircleAlert, LuLoaderCircle, LuRefreshCw } from "react-icons/lu";
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
				<LuLoaderCircle className="spin" aria-hidden="true" />
			) : (
				<LuCircleAlert aria-hidden="true" />
			)}
			<p>{message}</p>
			{onRetry && (
				<IconButton label="重新加载" onClick={onRetry}>
					<LuRefreshCw />
				</IconButton>
			)}
		</div>
	);
}
