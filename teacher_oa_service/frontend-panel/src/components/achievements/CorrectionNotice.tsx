import { Link } from "react-router-dom";
import { Pen } from "reicon-react";
import type { AchievementDetail } from "@/features/achievements/api";
import { formatAppDateTime } from "@/services/dateFormat";

export function CorrectionNotice({
	detail,
	editTo,
	from,
}: {
	detail: AchievementDetail;
	editTo?: string;
	from?: string;
}) {
	if (detail.record.status !== "returned") return null;
	const event = [...detail.events].reverse().find((e) => e.action === "return");
	if (!event) return null;
	return (
		<section className="correction-notice" aria-label="退回意见">
			<h2>待补正</h2>
			<p className="opinion-text">{event.opinion}</p>
			<div className="correction-actions">
				<p className="muted">
					{event.reviewer_name} · {formatAppDateTime(event.created_at)} ·
					修改完善后请重新提交审核
				</p>
				{editTo && (
					<Link
						className="action-button action-button-primary"
						to={editTo}
						state={from ? { from } : undefined}
					>
						<Pen aria-hidden="true" />
						去修改并重新提交
					</Link>
				)}
			</div>
		</section>
	);
}
