import { useId, useState } from "react";
import { ChevronDown } from "reicon-react";
import { type NavigationState, navGroupLabels } from "@/routes/navigationState";
import { NavItem } from "./NavItem";
import type { NavGroupItem } from "./navigation.config";

const storagePrefix = "teacher-oa:nav-group:";

function readPreference(key: string) {
	try {
		const value = window.localStorage.getItem(storagePrefix + key);
		return value === null ? null : value === "open";
	} catch {
		return null;
	}
}

/**
 * Open by default while the group owns the current page and every time the
 * user enters it from elsewhere. A manual toggle is remembered and applies to
 * pages outside the group, so it never hides where the user currently is.
 */
export function useNavGroupExpansion(key: string, containsCurrent: boolean) {
	const [expanded, setExpanded] = useState(
		() => containsCurrent || (readPreference(key) ?? true),
	);
	const [wasCurrent, setWasCurrent] = useState(containsCurrent);
	if (wasCurrent !== containsCurrent) {
		setWasCurrent(containsCurrent);
		if (containsCurrent) setExpanded(true);
	}
	const toggle = () => {
		const next = !expanded;
		setExpanded(next);
		try {
			window.localStorage.setItem(
				storagePrefix + key,
				next ? "open" : "closed",
			);
		} catch {
			return;
		}
	};
	return { expanded, toggle };
}

export function NavGroup({
	group,
	state,
}: {
	group: NavGroupItem;
	state: NavigationState;
}) {
	const containsCurrent = state.activeGroup === group.key;
	const { expanded, toggle } = useNavGroupExpansion(group.key, containsCurrent);
	const listId = useId();
	const Icon = group.icon;
	return (
		<li>
			<button
				type="button"
				className={`nav-link nav-group-toggle${containsCurrent ? " contains-current" : ""}`}
				aria-expanded={expanded}
				aria-controls={listId}
				onClick={toggle}
			>
				<Icon aria-hidden="true" />
				<span className="nav-text">{navGroupLabels[group.key]}</span>
				<ChevronDown className="nav-chevron" aria-hidden="true" />
			</button>
			<ul id={listId} className="nav-sublist" hidden={!expanded}>
				{group.children.map((item) => (
					<NavItem key={item.key} item={item} state={state} nested />
				))}
			</ul>
		</li>
	);
}
