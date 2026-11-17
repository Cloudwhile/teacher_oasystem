export const serviceName = "teacher_oa_service";

export const appTimeZone =
	(typeof import.meta !== "undefined" &&
		(import.meta.env?.VITE_APP_TIMEZONE as string | undefined)) ||
	"Asia/Singapore";

export const maxUploadBytes = 10 * 1024 * 1024;
