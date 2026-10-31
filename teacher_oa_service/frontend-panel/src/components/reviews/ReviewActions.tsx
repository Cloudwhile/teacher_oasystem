import { useRef, useState } from "react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import {
	type AchievementDetail,
	achievementApi,
	errorMessage,
} from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";

export function ReviewActions({
	detail,
	onComplete,
	onBusy,
	initialOpinion = "",
	onOpinionChange,
}: {
	detail: AchievementDetail;
	onComplete: () => void;
	onBusy: (busy: boolean) => void;
	initialOpinion?: string;
	onOpinionChange?: (value: string) => void;
}) {
	const { session, refresh } = useAuth();
	const [opinion, setOpinion] = useState(initialOpinion);
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	const [stale, setStale] = useState(false);
	const pending = useRef(false);
	const act = async (action: "approve" | "return" | "archive") => {
		if (!session || pending.current || stale) return;
		if (action === "return" && !opinion.trim()) {
			setError("请填写退回意见");
			return;
		}
		pending.current = true;
		setBusy(true);
		onBusy(true);
		setError("");
		try {
			await achievementApi.review(
				detail.record.id,
				action,
				detail.record.version,
				opinion,
				session.csrf_token,
			);
			onComplete();
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			if (error instanceof HttpError && error.status === 409) setStale(true);
			setError(errorMessage(error));
		} finally {
			pending.current = false;
			setBusy(false);
			onBusy(false);
		}
	};
	if (!detail.can_review && !detail.can_archive) return null;
	return (
		<section className="panel">
			<div className="section-heading">
				<h2>{detail.can_review ? "审核办理" : "成果归档"}</h2>
				<div className="row-actions">
					{detail.can_review && (
						<>
							<ActionButton
								variant="primary"
								disabled={busy || stale}
								onClick={() => void act("approve")}
							>
								审核通过
							</ActionButton>
							<ActionButton
								disabled={busy || stale}
								onClick={() => void act("return")}
							>
								退回补正
							</ActionButton>
						</>
					)}
					{detail.can_archive && (
						<ActionButton
							variant="primary"
							disabled={busy || stale}
							onClick={() => void act("archive")}
						>
							归档成果
						</ActionButton>
					)}
				</div>
			</div>
			<FormField id="review-opinion" label="办理意见" error={error}>
				<textarea
					id="review-opinion"
					value={opinion}
					maxLength={2000}
					rows={3}
					disabled={busy || stale}
					aria-invalid={Boolean(error)}
					aria-describedby={error ? "review-opinion-error" : undefined}
					onChange={(event) => {
						setOpinion(event.target.value);
						onOpinionChange?.(event.target.value);
					}}
				/>
			</FormField>
		</section>
	);
}
