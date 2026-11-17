import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
} from "react";
import { useBeforeUnload, useBlocker } from "react-router-dom";
import { useConfirm } from "@/components/common/ConfirmDialog";

export const LeaveGuardContext = createContext<{
	register: (guard: () => Promise<boolean>) => () => void;
} | null>(null);

export function useUnsavedChanges(dirty: boolean, busy: boolean) {
	const registry = useContext(LeaveGuardContext);
	const confirm = useConfirm();
	const bypass = useRef(false);

	const confirmLeave = useCallback(async () => {
		if (busy) return false;
		if (bypass.current || !dirty) return true;
		return confirm({
			title: "放弃未保存的内容？",
			description: "当前页面尚有未保存的修改，离开后这些修改将不会自动提交。",
			confirmLabel: "放弃并离开",
			cancelLabel: "继续编辑",
			tone: "danger",
		});
	}, [busy, dirty, confirm]);

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
		if (busy) {
			blocker.reset();
			return;
		}
		let cancelled = false;
		void confirmLeave().then((allowed) => {
			if (cancelled) return;
			if (allowed) blocker.proceed();
			else blocker.reset();
		});
		return () => {
			cancelled = true;
		};
	}, [blocker, busy, confirmLeave]);

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
