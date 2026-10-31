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
		[next[index], next[target]] = [next[target], next[index]];
		onChange(next.map((person, i) => ({ ...person, sort_order: i })));
	};
	return (
		<section className="panel">
			<div className="section-heading">
				<h2>{field.label}</h2>
				<IconButton
					label="添加相关人员"
					disabled={disabled || value.length >= 50}
					onClick={() =>
						onChange([
							...value,
							{
								teacher_id: null,
								external_name: "",
								role_code: roles[0]?.code ?? "participant",
								sort_order: value.length,
							},
						])
					}
				>
					<Plus />
				</IconButton>
			</div>
			{value.length === 0 && <p className="empty-message">尚未添加相关人员</p>}
			{value.map((person, index) => (
				<div className="participant-row" key={person.sort_order}>
					<span className="participant-order">{index + 1}</span>
					<FormField
						id={`person-${index}`}
						label="关联教师"
						error={errors[`participants.${index}.teacher_id`]}
					>
						<select
							id={`person-${index}`}
							value={person.teacher_id ?? ""}
							aria-invalid={Boolean(errors[`participants.${index}.teacher_id`])}
							aria-describedby={
								errors[`participants.${index}.teacher_id`]
									? `person-${index}-error`
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
										{person.external_name}（已关联）
									</option>
								)}
							{catalog.teachers.map((teacher) => (
								<option
									key={teacher.id}
									value={teacher.id}
									disabled={value.some(
										(item, i) => i !== index && item.teacher_id === teacher.id,
									)}
								>
									{teacher.name}
								</option>
							))}
						</select>
					</FormField>
					<FormField
						id={`name-${index}`}
						label="姓名"
						required={!person.teacher_id}
						error={errors[`participants.${index}.external_name`]}
					>
						<input
							id={`name-${index}`}
							value={person.external_name}
							maxLength={100}
							required={!person.teacher_id}
							aria-invalid={Boolean(
								errors[`participants.${index}.external_name`],
							)}
							aria-describedby={
								errors[`participants.${index}.external_name`]
									? `name-${index}-error`
									: undefined
							}
							disabled={disabled || Boolean(person.teacher_id)}
							onChange={(event) =>
								update(index, { ...person, external_name: event.target.value })
							}
						/>
					</FormField>
					<FormField
						id={`role-${index}`}
						label="人员角色"
						error={errors[`participants.${index}.role_code`]}
					>
						<select
							id={`role-${index}`}
							value={person.role_code}
							aria-invalid={Boolean(errors[`participants.${index}.role_code`])}
							aria-describedby={
								errors[`participants.${index}.role_code`]
									? `role-${index}-error`
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
							onClick={() =>
								onChange(
									value
										.filter((_, i) => i !== index)
										.map((item, i) => ({ ...item, sort_order: i })),
								)
							}
						>
							<Trash />
						</IconButton>
					</div>
					{errors[`participants.${index}`] && (
						<p className="field-error">{errors[`participants.${index}`]}</p>
					)}
				</div>
			))}
		</section>
	);
}
