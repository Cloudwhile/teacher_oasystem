import type { TeacherProfile } from "@/features/auth/api";
export const subjectLabels: Record<string, string> = {
	chinese: "语文",
	math: "数学",
	english: "英语",
	information_technology: "信息技术",
	physics: "物理",
	chemistry: "化学",
	biology: "生物",
	history: "历史",
	geography: "地理",
	politics: "政治",
	pe: "体育",
	music: "音乐",
	art: "美术",
	education_management: "教育管理",
	moral_education: "德育",
	other: "其他",
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
