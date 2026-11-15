import { useRef } from "react";
import { ArrowDown, ArrowUp, Plus, Trash } from "reicon-react";
import { FormField } from "@/components/common/FormField";
import { IconButton } from "@/components/common/IconButton";
import type {
	Catalog,
	FieldDefinition,
	Participant,
} from "@/features/achievements/api";

export function ParticipantsEditor({
	value,
	field,
	catalog,
	disabled,
	onChange,
	errors = {},
}: {
	value: Participant[];
	field: FieldDefinition;
	catalog: Catalog;
	disabled: boolean;
	onChange: (value: Participant[]) => void;
	errors?: Record<string, string>;
}) {
	const rowKeys = useRef<number[]>([]);
	const nextRowKey = useRef(0);
	while (rowKeys.current.length < value.length) {
		rowKeys.current.push(nextRowKey.current);
		nextRowKey.current += 1;
	}
	if (rowKeys.current.length > value.length) {
		rowKeys.current.length = value.length;
	}
	const roles = catalog.dictionaries.filter(
		(item) =>
			item.group_code === "participant_roles" &&
			field.roles?.includes(item.code),
	);
	const update = (index: number, person: Participant) =>
		onChange(value.map((item, i) => (i === index ? person : item)));
	const move = (index: number, offset: number) => {
		const next = [...value];
		const target = index + offset;
		if (target < 0 || target >= next.length) return;
		[rowKeys.current[index], rowKeys.current[target]] = [
			rowKeys.current[target],
			rowKeys.current[index],
		];
		[next[index], next[target]] = [next[target], next[index]];
		onChange(next.map((person, i) => ({ ...person, sort_order: i })));
	};
	return (
		<section className="panel">
			<div className="section-heading">
				<h2>
					{field.label}
					{field.required && (
						<span className="required-mark" aria-hidden="true">
							{" "}
							*
						</span>
					)}
				</h2>
				<IconButton
					label="添加相关人员"
					aria-invalid={Boolean(errors.participants)}
					aria-describedby={
						errors.participants ? "participants-error" : undefined
					}
					disabled={disabled || value.length >= 50}
					onClick={() => {
						rowKeys.current.push(nextRowKey.current);
						nextRowKey.current += 1;
						onChange([
							...value,
							{
								teacher_id: null,
								external_name: "",
								role_code: roles[0]?.code ?? "participant",
								sort_order: value.length,
							},
						]);
					}}
				>
					<Plus />
				</IconButton>
			</div>
			{errors.participants && (
				<p className="field-error" id="participants-error" role="alert">
					{errors.participants}
				</p>
			)}
			{value.length === 0 && <p className="empty-message">尚未添加相关人员</p>}
			{value.map((person, index) => {
				const rowKey = rowKeys.current[index] ?? index;
				const personId = `person-${rowKey}`;
				const nameId = `name-${rowKey}`;
				const roleId = `role-${rowKey}`;
				return (
					<div className="participant-row" key={rowKey}>
						<span className="participant-order">{index + 1}</span>
						<FormField
							id={personId}
							label="关联教师"
							error={errors[`participants.${index}.teacher_id`]}
						>
							<select
								id={personId}
								value={person.teacher_id ?? ""}
								aria-invalid={Boolean(
									errors[`participants.${index}.teacher_id`],
								)}
								aria-describedby={
									errors[`participants.${index}.teacher_id`]
										? `${personId}-error`
										: undefined
								}
								disabled={disabled}
								onChange={(event) => {
									const teacher = catalog.teachers.find(
										(item) => item.id === Number(event.target.value),
									);
									update(index, {
										...person,
										teacher_id: teacher?.id ?? null,
										teacher_name: teacher?.name,
										external_name: teacher?.name ?? "",
									});
								}}
							>
								<option value="">校外 / 未关联人员</option>
								{person.teacher_id &&
									!catalog.teachers.some(
										(teacher) => teacher.id === person.teacher_id,
									) && (
										<option value={person.teacher_id}>
											{person.external_name.trim() ||
												person.teacher_name ||
												`已关联教师 (#${person.teacher_id})`}
											（已关联）
										</option>
									)}
								{catalog.teachers.map((teacher) => (
									<option
										key={teacher.id}
										value={teacher.id}
										disabled={value.some(
											(item, i) =>
												i !== index && item.teacher_id === teacher.id,
										)}
									>
										{teacher.name}
									</option>
								))}
							</select>
						</FormField>
						<FormField
							id={nameId}
							label="姓名"
							required={!person.teacher_id}
							error={errors[`participants.${index}.external_name`]}
						>
							<input
								id={nameId}
								value={person.external_name}
								maxLength={100}
								required={!person.teacher_id}
								aria-invalid={Boolean(
									errors[`participants.${index}.external_name`],
								)}
								aria-describedby={
									errors[`participants.${index}.external_name`]
										? `${nameId}-error`
										: undefined
								}
								disabled={disabled || Boolean(person.teacher_id)}
								onChange={(event) =>
									update(index, {
										...person,
										external_name: event.target.value,
									})
								}
							/>
						</FormField>
						<FormField
							id={roleId}
							label="人员角色"
							error={errors[`participants.${index}.role_code`]}
						>
							<select
								id={roleId}
								value={person.role_code}
								aria-invalid={Boolean(
									errors[`participants.${index}.role_code`],
								)}
								aria-describedby={
									errors[`participants.${index}.role_code`]
										? `${roleId}-error`
										: undefined
								}
								disabled={disabled}
								onChange={(event) =>
									update(index, { ...person, role_code: event.target.value })
								}
							>
								{roles.map((role) => (
									<option key={role.code} value={role.code}>
										{role.name}
									</option>
								))}
							</select>
						</FormField>
						<div className="row-actions">
							<IconButton
								label={`上移第${index + 1}位人员`}
								disabled={disabled || index === 0}
								onClick={() => move(index, -1)}
							>
								<ArrowUp />
							</IconButton>
							<IconButton
								label={`下移第${index + 1}位人员`}
								disabled={disabled || index === value.length - 1}
								onClick={() => move(index, 1)}
							>
								<ArrowDown />
							</IconButton>
							<IconButton
								label={`移除第${index + 1}位人员`}
								disabled={disabled}
								onClick={() => {
									rowKeys.current.splice(index, 1);
									onChange(
										value
											.filter((_, i) => i !== index)
											.map((item, i) => ({ ...item, sort_order: i })),
									);
								}}
							>
								<Trash />
							</IconButton>
						</div>
						{errors[`participants.${index}`] && (
							<p className="field-error" role="alert">
								{errors[`participants.${index}`]}
							</p>
						)}
					</div>
				);
			})}
		</section>
	);
}
