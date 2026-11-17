import { Link } from "react-router-dom";
import { ArrowLeft, ShieldAlert } from "reicon-react";
import { appPaths } from "@/routes/routePaths";
import { EmptyState } from "./EmptyState";
import { PageHeading } from "./PageHeading";

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
			<section className="panel permission-feedback" role="alert">
				<EmptyState
					icon={<ShieldAlert />}
					title={message}
					description={description}
					action={
						<Link
							className="action-button action-button-secondary"
							to={appPaths.dashboard}
						>
							<ArrowLeft aria-hidden="true" />
							返回工作台
						</Link>
					}
				/>
			</section>
		</>
	);
}
