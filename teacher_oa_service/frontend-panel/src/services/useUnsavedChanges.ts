import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
} from "react";
import { useBeforeUnload, useBlocker } from "react-router-dom";

export const LeaveGuardContext = createContext<{
	register: (guard: () => boolean) => () => void;
} | null>(null);

export function useUnsavedChanges(dirty: boolean, busy: boolean) {
	const registry = useContext(LeaveGuardContext);
	const bypass = useRef(false);
	const confirmLeave = useCallback(
		() =>
			!busy &&
			(bypass.current ||
				!dirty ||
				window.confirm("尚有未保存的内容，确定放弃并离开吗？")),
		[busy, dirty],
	);
	const blocker = useBlocker(
		useCallback(
			({ currentLocation, nextLocation }) =>
				!bypass.current &&
				(dirty || busy) &&
				(currentLocation.pathname !== nextLocation.pathname ||
					currentLocation.search !== nextLocation.search),
			[dirty, busy],
		),
	);
	useEffect(() => {
		if (blocker.state !== "blocked") return;
		if (confirmLeave()) blocker.proceed();
		else blocker.reset();
	}, [blocker, confirmLeave]);
	useEffect(() => registry?.register(confirmLeave), [registry, confirmLeave]);
	useBeforeUnload(
		useCallback(
			(event) => {
				if (!bypass.current && (dirty || busy)) event.preventDefault();
			},
			[dirty, busy],
		),
	);
	return {
		confirmLeave,
		allowLeave: () => {
			bypass.current = true;
		},
	};
}
