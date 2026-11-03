import { useCallback, useSyncExternalStore } from "react";

export function useMediaQuery(query: string) {
	const subscribe = useCallback(
		(notify: () => void) => {
			if (typeof window.matchMedia !== "function") return () => {};
			const list = window.matchMedia(query);
			list.addEventListener("change", notify);
			return () => list.removeEventListener("change", notify);
		},
		[query],
	);
	return useSyncExternalStore(
		subscribe,
		() =>
			typeof window.matchMedia === "function" &&
			window.matchMedia(query).matches,
		() => false,
	);
}
