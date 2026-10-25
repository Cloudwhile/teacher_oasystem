import { requestJson } from "@/services/httpClient";
import type { OperationJsonResponse, OperationRequestBody } from "@/types/api";
import type { components } from "@/types/api.generated";

export type AuthResponse = components["schemas"]["AuthResponse"];
export type CurrentUser = components["schemas"]["CurrentUser"];
export type LoginOptions = components["schemas"]["LoginOptions"];
export type TeacherProfile = components["schemas"]["TeacherProfile"];

function requireData<T>(data: T | undefined): T {
	if (data === undefined) throw new Error("响应缺少数据");
	return data;
}

export const authApi = {
	async options() {
		const response = await requestJson<OperationJsonResponse<"auth_options">>(
			"/api/v1/auth/options",
		);
		return requireData(response.data);
	},
	async me(signal?: AbortSignal) {
		const response = await requestJson<OperationJsonResponse<"auth_me">>(
			"/api/v1/auth/me",
			{ signal },
		);
		return requireData(response.data);
	},
	async login(body: OperationRequestBody<"auth_mock_login">) {
		const response = await requestJson<
			OperationJsonResponse<"auth_mock_login">
		>("/api/v1/auth/mock/login", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(body),
		});
		return requireData(response.data);
	},
	async logout(csrfToken: string) {
		await requestJson("/api/v1/auth/logout", {
			method: "POST",
			headers: { "X-CSRF-Token": csrfToken },
		});
	},
	async profile(signal?: AbortSignal) {
		const response = await requestJson<OperationJsonResponse<"teachers_me">>(
			"/api/v1/teachers/me",
			{ signal },
		);
		return requireData(response.data);
	},
};
