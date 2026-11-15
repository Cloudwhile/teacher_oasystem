const singaporeDateFormatter = new Intl.DateTimeFormat("en-CA", {
	timeZone: "Asia/Singapore",
	year: "numeric",
	month: "2-digit",
	day: "2-digit",
});

export function formatSingaporeDate(
	value: string | Date | null | undefined,
): string {
	if (!value) return "";
	const date = value instanceof Date ? value : new Date(value);
	if (Number.isNaN(date.getTime()))
		return typeof value === "string" ? value.slice(0, 10) : "";
	const parts = singaporeDateFormatter.formatToParts(date);
	const year = parts.find((part) => part.type === "year")?.value;
	const month = parts.find((part) => part.type === "month")?.value;
	const day = parts.find((part) => part.type === "day")?.value;
	return year && month && day ? `${year}-${month}-${day}` : "";
}
