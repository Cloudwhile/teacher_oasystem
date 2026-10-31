import { requestJson } from "@/services/httpClient";
import type { components } from "@/types/api.generated";

export type LoginPageSettings = components["schemas"]["LoginPageSettings"];
export type LoginAnnouncement = components["schemas"]["LoginAnnouncement"];

export const defaultLoginPage: LoginPageSettings = {
	title: "TeacherOA · 教师综合 OA",
	background_url: "",
	logo_url: "",
	footer: "TeacherOA · 教师工作空间",
	announcements: [],
	version: 0,
};

export const loginPageApi = {
	async public(signal?: AbortSignal) {
		const response = await requestJson<{ data: LoginPageSettings }>(
			"/api/v1/login-page",
			{ signal },
		);
		return response.data;
	},
	async read(signal?: AbortSignal) {
		const response = await requestJson<{ data: LoginPageSettings }>(
			"/api/v1/admin/login-page",
			{ signal },
		);
		return response.data;
	},
	async save(settings: LoginPageSettings, csrf: string) {
		const response = await requestJson<{ data: LoginPageSettings }>(
			"/api/v1/admin/login-page",
			{
				method: "PUT",
				headers: { "Content-Type": "application/json", "X-CSRF-Token": csrf },
				body: JSON.stringify(settings),
			},
		);
		return response.data;
	},
	async upload(file: File, csrf: string) {
		const body = new FormData();
		body.append("file", file);
		const response = await requestJson<{ data: { url: string } }>(
			"/api/v1/admin/login-page/assets",
			{
				method: "POST",
				headers: { "X-CSRF-Token": csrf },
				body,
			},
		);
		return response.data.url;
	},
};
