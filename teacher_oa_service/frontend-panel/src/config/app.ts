export const serviceName = "teacher_oa_service";

export const appTimeZone =
	(typeof import.meta !== "undefined" &&
		(import.meta.env?.VITE_APP_TIMEZONE as string | undefined)) ||
	"Asia/Singapore";

export const maxUploadBytes = 20 * 1024 * 1024;

export function formatUploadLimitMb(bytes = maxUploadBytes): string {
	return `${Math.max(1, Math.round(bytes / (1024 * 1024)))}MB`;
}
