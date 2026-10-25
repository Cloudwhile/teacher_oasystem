import { HttpError, requestJson } from "@/services/httpClient";
import type {
	OperationData,
	OperationJsonResponse,
	operations,
} from "@/types/api";
import type { components } from "@/types/api.generated";

export type AchievementInput = components["schemas"]["AchievementInput"];
export type AchievementDetail = components["schemas"]["AchievementDetail"];
export type AchievementSchema = components["schemas"]["AchievementSchema"];
export type FieldDefinition = components["schemas"]["FieldDefinition"];
export type Participant = components["schemas"]["ParticipantInput"];
export type FileMetadata = components["schemas"]["FileMetadata"];
export type AcademicTerm = components["schemas"]["AcademicTerm"];
export type DictionaryItem = components["schemas"]["DictionaryItem"];
export type TeacherOption = components["schemas"]["TeacherOption"];

async function call<K extends keyof operations>(
	url: string,
	init: RequestInit = {},
): Promise<OperationData<K>> {
	const response = await requestJson<OperationJsonResponse<K>>(url, init);
	if (!response || typeof response !== "object" || !("data" in response))
		throw new Error("响应缺少数据");
	return response.data as OperationData<K>;
}
function write(method: string, csrf: string, body: unknown): RequestInit {
	return {
		method,
		headers: { "Content-Type": "application/json", "X-CSRF-Token": csrf },
		body: JSON.stringify(body),
	};
}
export const achievementApi = {
	async catalog(signal?: AbortSignal) {
		const [types, terms, dictionaries, teachers] = await Promise.all([
			call<"achievement_types">("/api/v1/achievement-types", { signal }),
			call<"academic_terms">("/api/v1/academic-terms", { signal }),
			call<"dictionaries">("/api/v1/dictionaries", { signal }),
			call<"teacher_options">("/api/v1/teachers", { signal }),
		]);
		return { types, terms, dictionaries, teachers };
	},
	schema: (code: string, version?: number, signal?: AbortSignal) =>
		call<"achievement_schema">(
			`/api/v1/achievement-types/${encodeURIComponent(code)}/schema${version ? `?version=${version}` : ""}`,
			{ signal },
		),
	list: (query: string, signal?: AbortSignal) =>
		call<"achievements_list">(`/api/v1/achievements?${query}`, { signal }),
	detail: (id: number, signal?: AbortSignal) =>
		call<"achievement_detail">(`/api/v1/achievements/${id}`, { signal }),
	revisions: (id: number, signal?: AbortSignal) =>
		call<"achievement_revisions">(`/api/v1/achievements/${id}/revisions`, {
			signal,
		}),
	create: (data: AchievementInput, csrf: string) =>
		call<"achievement_create">(
			"/api/v1/achievements",
			write("POST", csrf, data),
		),
	update: (id: number, version: number, data: AchievementInput, csrf: string) =>
		call<"achievement_update">(
			`/api/v1/achievements/${id}`,
			write("PATCH", csrf, { version, data }),
		),
	submit: (id: number, version: number, csrf: string) =>
		call<"achievement_submit">(
			`/api/v1/achievements/${id}/submit`,
			write("POST", csrf, { version }),
		),
	files: (page: number, signal?: AbortSignal) =>
		call<"files_list">(`/api/v1/files?page=${page}&page_size=20`, { signal }),
	upload: (file: File, csrf: string) => {
		const body = new FormData();
		body.append("file", file);
		return call<"file_upload">("/api/v1/files/upload", {
			method: "POST",
			headers: { "X-CSRF-Token": csrf },
			body,
		});
	},
};
export function errorMessage(
	error: unknown,
	fallback = "暂时无法完成，请稍后重试。",
) {
	if (error instanceof HttpError) {
		const failure = error.payload as components["schemas"]["Failure"] | null;
		return typeof failure?.msg === "string" ? failure.msg : fallback;
	}
	return fallback;
}
export function validationErrors(error: unknown): Record<string, string> {
	if (!(error instanceof HttpError)) return {};
	const failure = error.payload as components["schemas"]["Failure"] | null;
	return failure?.error?.diagnostic?.fields ?? {};
}
export type Catalog = Awaited<ReturnType<typeof achievementApi.catalog>>;
export const statusLabels: Record<string, string> = {
	draft: "草稿",
	pending: "审核中",
	returned: "待补正",
	approved: "已通过",
	archived: "已归档",
};
