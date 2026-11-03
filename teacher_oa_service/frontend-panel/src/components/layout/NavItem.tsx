import { Link } from "react-router-dom";
import { type NavigationState, navLabels } from "@/routes/navigationState";
import type { NavLinkItem } from "./navigation.config";

export function NavItem({
	item,
	state,
	nested = false,
}: {
	item: NavLinkItem;
	state: NavigationState;
	nested?: boolean;
}) {
	const active = state.activeKey === item.key;
	const Icon = item.icon;
	return (
		<li>
			<Link
				to={item.to}
				className={`nav-link${nested ? " nav-sublink" : ""}${active ? " active" : ""}`}
				aria-current={
					active
						? state.activeMatch === "page"
							? "page"
							: "location"
						: undefined
				}
			>
				{Icon && <Icon aria-hidden="true" />}
				<span className="nav-text">{navLabels[item.key]}</span>
			</Link>
		</li>
	);
}
