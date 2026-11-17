import { Link } from "react-router-dom";
import { AlertCircle, ArrowLeft } from "reicon-react";
import { appPaths } from "@/routes/routePaths";

export function ErrorPage() {
	return (
		<section className="error-page">
			<span className="empty-state-icon" aria-hidden="true">
				<AlertCircle />
			</span>
			<h1>页面暂不可用</h1>
			<p className="empty-state-description">
				访问的地址不存在、已移动，或当前暂时无法打开。
			</p>
			<Link
				className="action-button action-button-primary"
				to={appPaths.dashboard}
			>
				<ArrowLeft aria-hidden="true" />
				返回工作台
			</Link>
		</section>
	);
}
