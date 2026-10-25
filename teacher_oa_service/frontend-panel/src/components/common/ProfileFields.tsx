import type { TeacherProfile } from "@/features/auth/api";
export const subjectLabels: Record<string, string> = {
	math: "数学",
	chinese: "语文",
	english: "英语",
	information_technology: "信息技术",
};
export function ProfileFields({ profile }: { profile: TeacherProfile }) {
	return (
		<dl className="profile-fields">
			<div>
				<dt>姓名</dt>
				<dd>{profile.name}</dd>
			</div>
			<div>
				<dt>教师工号</dt>
				<dd>{profile.employee_no}</dd>
			</div>
			<div>
				<dt>所属部门</dt>
				<dd>{profile.department_name}</dd>
			</div>
			<div>
				<dt>任教学科</dt>
				<dd>{subjectLabels[profile.subject_code] ?? profile.subject_code}</dd>
			</div>
			<div>
				<dt>在职状态</dt>
				<dd>
					<span className="status-badge">
						{profile.employment_status === "active" ? "在职" : "离职"}
					</span>
				</dd>
			</div>
		</dl>
	);
}
