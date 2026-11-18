import type { CurrentUser } from "./api";

export type Permission =
	| "achievements:write"
	| "teachers:directory"
	| "reviews:write"
	| "admin:manage";

/** UI visibility only; the API enforces every permission independently. */
export function hasPermission(
	user: Pick<CurrentUser, "grants"> | null | undefined,
	permission: Permission,
) {
	return Boolean(user?.grants.some((grant) => grant.permission === permission));
}
