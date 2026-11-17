import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "./httpClient";

export function useResource<T>(load: (signal: AbortSignal) => Promise<T>) {
	const [data, setData] = useState<T | null>(null);
	const [loading, setLoading] = useState(true);
	const [refreshing, setRefreshing] = useState(false);
	const [error, setError] = useState<unknown>(null);
	const [revision, setRevision] = useState(0);
	const hasDataRef = useRef(false);
	const { refresh } = useAuth();
	const reload = useCallback(() => setRevision((value) => value + 1), []);
	const request = useMemo(() => ({ load, revision }), [load, revision]);
	useEffect(() => {
		const controller = new AbortController();
		if (hasDataRef.current) {
			setRefreshing(true);
		} else {
			setLoading(true);
		}
		setError(null);
		request
			.load(controller.signal)
			.then((value) => {
				if (!controller.signal.aborted) {
					hasDataRef.current = true;
					setData(value);
				}
			})
			.catch((error: unknown) => {
				if (!controller.signal.aborted) {
					setError(error);
					if (error instanceof HttpError && error.status === 401)
						void refresh();
				}
			})
			.finally(() => {
				if (!controller.signal.aborted) {
					setLoading(false);
					setRefreshing(false);
				}
			});
		return () => controller.abort();
	}, [request, refresh]);
	return { data, loading, refreshing, error, reload };
}
