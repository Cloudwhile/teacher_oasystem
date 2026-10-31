import { requestJson } from "@/services/httpClient";
import type {
	OperationData,
	OperationJsonResponse,
	operations,
} from "@/types/api";
import type { components } from "@/types/api.generated";
export type AccountItem = components["schemas"]["AccountItem"];
export type AccountInput = components["schemas"]["AccountInput"];
export type WecomSettings = components["schemas"]["WecomSettings"];
export type SaveWecomSettings = components["schemas"]["SaveWecomSettings"];
async function call<K extends keyof operations>(
	url: string,
	init: RequestInit = {},
): Promise<OperationData<K>> {
	const response = await requestJson<OperationJsonResponse<K>>(url, init);
	if (!response || typeof response !== "object" || !("data" in response))
		throw new Error("响应缺少数据");
	return response.data as OperationData<K>;
}
const write = (method: string, body: unknown, csrf: string) => ({
	method,
	headers: { "Content-Type": "application/json", "X-CSRF-Token": csrf },
	body: JSON.stringify(body),
});
export const adminApi = {
	accounts: (signal?: AbortSignal) =>
		call<"admin_accounts">("/api/v1/admin/accounts", { signal }),
	saveAccount: (body: AccountInput, csrf: string) =>
		call<"admin_save_account">(
			"/api/v1/admin/accounts",
			write("POST", body, csrf),
		),
	settings: (signal?: AbortSignal) =>
		call<"admin_wecom">("/api/v1/admin/wecom", { signal }),
	saveSettings: (body: SaveWecomSettings, csrf: string) =>
		call<"admin_save_wecom">("/api/v1/admin/wecom", write("PUT", body, csrf)),
};
