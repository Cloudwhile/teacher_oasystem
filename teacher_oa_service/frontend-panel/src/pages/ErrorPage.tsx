import { LuArrowLeft } from "react-icons/lu";
import { Link } from "react-router-dom";
import { appPaths } from "@/routes/routePaths";

export function ErrorPage() {
	return (
		<div className="error-page">
			<h1>页面暂不可用</h1>
			<Link className="text-link" to={appPaths.dashboard}>
				<LuArrowLeft aria-hidden="true" />
				返回工作台
			</Link>
		</div>
	);
}
