import { useAuth } from "@/features/auth/AuthProvider";
import { hasPermission } from "@/features/auth/permissions";
import type { NavigationState } from "@/routes/navigationState";
import { NavGroup } from "./NavGroup";
import { NavItem } from "./NavItem";
import { navigationSections, visibleSections } from "./navigation.config";

export function Navigation({ state }: { state: NavigationState }) {
	const { session } = useAuth();
	const sections = visibleSections(navigationSections, (permission) =>
		hasPermission(session?.user, permission),
	);
	return (
		<div className="navigation">
			{sections.map((section) => (
				<div className="nav-section" key={section.id}>
					<p className="nav-group-label" id={`nav-section-${section.id}`}>
						{section.label}
					</p>
					<ul
						className="nav-list"
						aria-labelledby={`nav-section-${section.id}`}
					>
						{section.items.map((item) =>
							item.kind === "group" ? (
								<NavGroup key={item.key} group={item} state={state} />
							) : (
								<NavItem key={item.key} item={item} state={state} />
							),
						)}
					</ul>
				</div>
			))}
		</div>
	);
}
