import type { ReactNode } from "react";
import { FormField } from "@/components/common/FormField";
import type {
	AchievementInput,
	Catalog,
	FieldDefinition,
} from "@/features/achievements/api";

const common = new Set([
	"title",
	"remarks",
	"subject_code",
	"academic_term_id",
	"occurred_at",
	"level_code",
	"award_grade_code",
]);
export function fieldValue(data: AchievementInput, key: string): unknown {
	if (key === "files") return data.file_ids;
	if (key === "participants") return data.participants;
	return common.has(key)
		? data[key as keyof AchievementInput]
		: data.attributes?.[key];
}
export function fieldErrorKey(key: string) {
	return key === "files"
		? "file_ids"
		: key === "participants" || common.has(key)
			? key
			: `attributes.${key}`;
}
export function changeField(
	data: AchievementInput,
	field: FieldDefinition,
	raw: string | string[],
): AchievementInput {
	const value =
		field.type === "multi_select"
			? raw
			: field.key === "academic_term_id" ||
					(field.type === "number" && field.format !== "decimal")
				? raw === ""
					? null
					: Number(raw)
				: raw;
	if (common.has(field.key))
		return {
			...data,
			term_mismatch_confirmed:
				field.key === "occurred_at" || field.key === "academic_term_id"
					? false
					: data.term_mismatch_confirmed,
			[field.key]: ["title", "remarks"].includes(field.key)
				? raw
				: value || null,
		};
	const attributes = { ...data.attributes };
	if (value === "" || value === null) delete attributes[field.key];
	else attributes[field.key] = value;
	return { ...data, attributes };
}
export function DynamicFields({
	fields,
	data,
	catalog,
	disabled,
	errors,
	onChange,
}: {
	fields: FieldDefinition[];
	data: AchievementInput;
	catalog: Catalog;
	disabled: boolean;
	errors: Record<string, string>;
	onChange: (data: AchievementInput) => void;
}) {
	return (
		<div className="form-grid">
			{fields
				.filter(
					(field) =>
						field.visible &&
						!["file_list", "participant_list"].includes(field.type),
				)
				.map((field) => {
					const value = fieldValue(data, field.key);
					const error = errors[fieldErrorKey(field.key)];
					const props = {
						id: field.key,
						disabled,
						"aria-invalid": Boolean(error),
						"aria-describedby": error ? `${field.key}-error` : undefined,
					};
					const options =
						field.key === "academic_term_id"
							? catalog.terms
									.filter((term) => term.enabled || term.id === value)
									.map((term) => ({
										value: String(term.id),
										label: term.name,
									}))
							: field.dictionary
								? catalog.dictionaries
										.filter(
											(item) =>
												item.group_code === field.dictionary &&
												(!field.options?.length ||
													field.options.includes(item.code)),
										)
										.map((item) => ({ value: item.code, label: item.name }))
								: (field.options ?? []).map((option) => ({
										value: option,
										label: option,
									}));
					let control: ReactNode;
					if (field.type === "select" || field.type === "multi_select")
						control = (
							<select
								{...props}
								multiple={field.type === "multi_select"}
								value={
									field.type === "multi_select"
										? Array.isArray(value)
											? value.filter(
													(item): item is string => typeof item === "string",
												)
											: []
										: String(value ?? "")
								}
								onChange={(event) =>
									onChange(
										changeField(
											data,
											field,
											field.type === "multi_select"
												? Array.from(
														event.target.selectedOptions,
														(option) => option.value,
													)
												: event.target.value,
										),
									)
								}
							>
								{field.type === "select" && <option value="">请选择</option>}
								{options.map((option) => (
									<option key={option.value} value={option.value}>
										{option.label}
									</option>
								))}
							</select>
						);
					else if (field.type === "textarea")
						control = (
							<textarea
								{...props}
								rows={3}
								maxLength={field.maxLength ?? undefined}
								value={String(value ?? "")}
								onChange={(event) =>
									onChange(changeField(data, field, event.target.value))
								}
							/>
						);
					else
						control = (
							<input
								{...props}
								type={
									field.type === "date"
										? "date"
										: field.type === "number" && field.format !== "decimal"
											? "number"
											: "text"
								}
								inputMode={field.type === "number" ? "decimal" : undefined}
								min={field.type === "number" ? 0 : undefined}
								step={field.type === "number" ? 1 : undefined}
								maxLength={field.maxLength ?? undefined}
								required={field.key === "title"}
								value={String(value ?? "")}
								onChange={(event) =>
									onChange(changeField(data, field, event.target.value))
								}
							/>
						);
					return (
						<FormField
							key={field.key}
							id={field.key}
							label={field.label}
							required={field.required}
							error={error}
						>
							{control}
						</FormField>
					);
				})}
		</div>
	);
}
