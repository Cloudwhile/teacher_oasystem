import type { AchievementDetail } from "@/features/achievements/api";
export function CorrectionNotice({ detail }: { detail: AchievementDetail }) {
	if (detail.record.status !== "returned") return null;
	const event = [...detail.events].reverse().find((e) => e.action === "return");
	if (!event) return null;
	return (
		<section className="correction-notice" aria-label="退回意见">
			<h2>待补正</h2>
			<p className="opinion-text">{event.opinion}</p>
			<p className="muted">
				{event.reviewer_name} ·{" "}
				{new Date(event.created_at).toLocaleString("zh-CN", {
					timeZone: "Asia/Singapore",
				})}
			</p>
		</section>
	);
}
