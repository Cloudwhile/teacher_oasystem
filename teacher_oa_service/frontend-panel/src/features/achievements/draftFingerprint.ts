import type { AchievementInput } from "./api";

function canonical(value: unknown): unknown {
	if (Array.isArray(value)) return value.map(canonical);
	if (value && typeof value === "object") {
		return Object.fromEntries(
			Object.entries(value)
				.filter(([, item]) => item != null && item !== "")
				.sort(([left], [right]) => left.localeCompare(right))
				.map(([key, item]) => [key, canonical(item)]),
		);
	}
	return value;
}

export function draftFingerprint(data: AchievementInput) {
	return JSON.stringify(canonical(data));
}
