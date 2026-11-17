import { appTimeZone } from "@/config/app";

const isoDateFormatter = new Intl.DateTimeFormat("en-CA", {
	timeZone: appTimeZone,
	year: "numeric",
	month: "2-digit",
	day: "2-digit",
});

const dateTimeFormatter = new Intl.DateTimeFormat("zh-CN", {
	timeZone: appTimeZone,
	year: "numeric",
	month: "2-digit",
	day: "2-digit",
	hour: "2-digit",
	minute: "2-digit",
	hour12: false,
});

const longDateFormatter = new Intl.DateTimeFormat("zh-CN", {
	timeZone: appTimeZone,
	year: "numeric",
	month: "long",
	day: "numeric",
});

export function formatAppDate(value: string | Date | null | undefined): string {
	if (!value) return "";
	if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/.test(value)) {
		return value;
	}
	const date = value instanceof Date ? value : new Date(value);
	if (Number.isNaN(date.getTime()))
		return typeof value === "string" ? value.slice(0, 10) : "";
	const parts = isoDateFormatter.formatToParts(date);
	const year = parts.find((part) => part.type === "year")?.value;
	const month = parts.find((part) => part.type === "month")?.value;
	const day = parts.find((part) => part.type === "day")?.value;
	return year && month && day ? `${year}-${month}-${day}` : "";
}

export function formatAppDateTime(
	value: string | Date | null | undefined,
): string {
	if (!value) return "";
	const date = value instanceof Date ? value : new Date(value);
	if (Number.isNaN(date.getTime())) return String(value);
	return dateTimeFormatter.format(date).replace(/\//g, "-");
}

export function formatAppLongDate(value: Date): string {
	return longDateFormatter.format(value);
}

/** Backwards-compatible alias delegating to the configured appTimeZone. */
export const formatSingaporeDate = formatAppDate;
