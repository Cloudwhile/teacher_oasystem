import { useCallback, useState } from "react";
import { useLocation } from "react-router-dom";

/**
 * Open state that resets once the router commits a new location. A navigation
 * blocked by the unsaved-changes guard keeps the location, so the panel stays
 * open; returning to the original entry through history does not reopen it.
 */
export function useOpenUntilNavigation() {
	const { key } = useLocation();
	const [open, setOpenState] = useState(false);
	const [openedAt, setOpenedAt] = useState(key);
	if (open && openedAt !== key) setOpenState(false);
	const setOpen = useCallback(
		(next: boolean) => {
			setOpenState(next);
			setOpenedAt(key);
		},
		[key],
	);
	return [open && openedAt === key, setOpen] as const;
}
