import { requestJson } from "@/services/httpClient";
import type {
	OperationData,
	OperationJsonResponse,
	operations,
} from "@/types/api";
import type { components } from "@/types/api.generated";

export type TeacherDirectoryItem =
	components["schemas"]["TeacherDirectoryItem"];
export type TeacherDirectoryPage =
	components["schemas"]["TeacherDirectoryPage"];
export type TeacherDossier = components["schemas"]["TeacherDossier"];
export type DepartmentOption = components["schemas"]["DepartmentOption"];

async function call<K extends keyof operations>(
	url: string,
	init: RequestInit = {},
): Promise<OperationData<K>> {
	const response = await requestJson<OperationJsonResponse<K>>(url, init);
	if (!response || typeof response !== "object" || !("data" in response))
		throw new Error("响应缺少数据");
	return response.data as OperationData<K>;
}

export const teacherDirectoryApi = {
	list: (query: string, signal?: AbortSignal) =>
		call<"teachers_directory">(
			`/api/v1/teachers/directory${query ? `?${query}` : ""}`,
			{ signal },
		),
	dossier: (id: number, signal?: AbortSignal) =>
		call<"teachers_dossier">(`/api/v1/teachers/${id}/dossier`, { signal }),
};
