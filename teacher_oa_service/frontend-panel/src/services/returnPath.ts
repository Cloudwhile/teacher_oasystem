export function safeReturnPath(
	value: unknown,
	fallback = "/dashboard",
): string {
	if (
		typeof value !== "string" ||
		value.length > 2048 ||
		value.includes("\\") ||
		[...value].some((char) => char.charCodeAt(0) < 32)
	)
		return fallback;
	return [
		"/dashboard",
		"/achievements",
		"/reviews",
		"/files",
		"/teachers",
		"/account",
		"/admin",
	].some(
		(prefix) =>
			value === prefix ||
			value.startsWith(`${prefix}/`) ||
			value.startsWith(`${prefix}?`),
	)
		? value
		: fallback;
}

const lastListStorageKey = "teacher-oa:last-list-path";

export function rememberListPath(pathname: string, search: string) {
	if (typeof window === "undefined") return;
	if (pathname !== "/achievements" && pathname !== "/reviews") return;
	try {
		window.sessionStorage.setItem(lastListStorageKey, pathname + search);
	} catch {
		// Ignore storage errors.
	}
}

export function readLastAchievementListPath(
	fallback = "/achievements",
): string {
	if (typeof window === "undefined") return fallback;
	try {
		const saved = window.sessionStorage.getItem(lastListStorageKey);
		if (saved?.startsWith("/achievements")) {
			return safeReturnPath(saved, fallback);
		}
	} catch {
		return fallback;
	}
	return fallback;
}
