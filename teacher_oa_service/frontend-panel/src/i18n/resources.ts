import enErrors from "@/i18n/locales/en-US/errors.json";
import enShell from "@/i18n/locales/en-US/shell.json";
import zhErrors from "@/i18n/locales/zh-CN/errors.json";
import zhShell from "@/i18n/locales/zh-CN/shell.json";

export const resources = {
	"en-US": {
		shell: enShell,
		errors: enErrors,
	},
	"zh-CN": {
		shell: zhShell,
		errors: zhErrors,
	},
} as const;
