import { Link } from "react-router-dom";
import { ArrowLeft, ShieldAlert } from "reicon-react";
import { appPaths } from "@/routes/routePaths";
import { PageHeading } from "./PageHeading";
import { StatusStateCard } from "./StatusState";

export function PermissionDenied({
	title,
	message,
	description = "如需开通该功能权限，请联系学校系统管理员。",
}: {
	title: string;
	message: string;
	description?: string;
}) {
	return (
		<>
			<PageHeading title={title} />
			<div className="panel status-panel">
				<StatusStateCard
					role="alert"
					headingLevel="h2"
					tone="warning"
					icon={<ShieldAlert />}
					code="403 · 权限受限"
					title={message}
					description={description}
					actions={
						<Link
							className="action-button action-button-secondary"
							to={appPaths.dashboard}
						>
							<ArrowLeft aria-hidden="true" />
							返回工作台
						</Link>
					}
				/>
			</div>
		</>
	);
}
