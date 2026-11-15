import type { Catalog, Participant } from "@/features/achievements/api";
export function ParticipantList({
	participants,
	catalog,
}: {
	participants: Participant[];
	catalog: Catalog;
}) {
	if (!participants.length) return null;
	return (
		<section
			className="table-scroll"
			// biome-ignore lint/a11y/noNoninteractiveTabindex: horizontal scrolling must be keyboard accessible.
			tabIndex={0}
			aria-label="相关人员列表"
		>
			<table>
				<thead>
					<tr>
						<th scope="col">顺序</th>
						<th scope="col">姓名</th>
						<th scope="col">角色</th>
					</tr>
				</thead>
				<tbody>
					{participants.map((person, index) => (
						<tr
							key={`${person.sort_order}-${person.teacher_id ?? person.external_name}`}
						>
							<td>{index + 1}</td>
							<td>
								{person.external_name.trim() ||
									person.teacher_name ||
									catalog.teachers.find(
										(teacher) => teacher.id === person.teacher_id,
									)?.name ||
									"—"}
							</td>
							<td>
								{catalog.dictionaries.find(
									(d) =>
										d.group_code === "participant_roles" &&
										d.code === person.role_code,
								)?.name ?? person.role_code}
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</section>
	);
}
