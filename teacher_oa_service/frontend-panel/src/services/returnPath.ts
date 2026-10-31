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
