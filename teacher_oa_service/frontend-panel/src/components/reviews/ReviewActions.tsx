import { useRef, useState } from "react";
import { Refresh } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { FormField } from "@/components/common/FormField";
import {
	type AchievementDetail,
	achievementApi,
	errorMessage,
} from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";

type ReviewActionType = "approve" | "return" | "archive";

const actionMeta: Record<
	ReviewActionType,
	{
		title: string;
		description: string;
		confirmLabel: string;
		successToast: string;
		tone: "primary" | "danger";
	}
> = {
	approve: {
		title: "确认审核通过？",
		description: "审核通过后，该成果将更新为「已通过」状态并进入待归档阶段。",
		confirmLabel: "确认通过",
		successToast: "成果已审核通过",
		tone: "primary",
	},
	return: {
		title: "确认退回补正？",
		description: "退回后，填报教师将收到您的补正意见并可修改后重新提交。",
		confirmLabel: "确认退回",
		successToast: "已退回教师补正",
		tone: "danger",
	},
	archive: {
		title: "确认归档该成果？",
		description: "归档操作不可撤销，归档后成果将锁定为正式档案。",
		confirmLabel: "确认归档",
		successToast: "成果已正式归档",
		tone: "danger",
	},
};

export function ReviewActions({
	detail,
	onComplete,
	onReload,
	onBusy,
	initialOpinion = "",
	onOpinionChange,
}: {
	detail: AchievementDetail;
	onComplete: () => void;
	onReload?: () => void;
	onBusy: (busy: boolean) => void;
	initialOpinion?: string;
	onOpinionChange?: (value: string) => void;
}) {
	const { session, refresh } = useAuth();
	const [opinion, setOpinion] = useState(initialOpinion);
	const [error, setError] = useState("");
	const [busy, setBusy] = useState(false);
	const [stale, setStale] = useState(false);
	const [confirming, setConfirming] = useState<ReviewActionType | null>(null);
	const pending = useRef(false);

	const requestAction = (action: ReviewActionType) => {
		if (!session || pending.current || stale || busy) return;
		if (action === "return" && !opinion.trim()) {
			setError("请填写退回意见");
			return;
		}
		setError("");
		setConfirming(action);
	};

	const executeAction = async (action: ReviewActionType) => {
		if (!session || pending.current || stale) return;
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
			setConfirming(null);
			notifySuccess(actionMeta[action].successToast);
			onComplete();
		} catch (error) {
			setConfirming(null);
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
	const activeMeta = confirming ? actionMeta[confirming] : null;

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
								onClick={() => requestAction("approve")}
							>
								审核通过
							</ActionButton>
							<ActionButton
								disabled={busy || stale}
								onClick={() => requestAction("return")}
							>
								退回补正
							</ActionButton>
						</>
					)}
					{detail.can_archive && (
						<ActionButton
							variant="primary"
							disabled={busy || stale}
							onClick={() => requestAction("archive")}
						>
							归档成果
						</ActionButton>
					)}
				</div>
			</div>
			{stale && (
				<div className="conflict-banner" role="status">
					<span>
						该记录已被其他用户更新，当前审核按钮已锁定；重新加载后将保留您已填写的办理意见。
					</span>
					{onReload && (
						<ActionButton
							variant="secondary"
							size="sm"
							disabled={busy}
							onClick={onReload}
						>
							<Refresh aria-hidden="true" />
							重新加载最新记录
						</ActionButton>
					)}
				</div>
			)}
			<FormField id="review-opinion" label="办理意见" error={error}>
				<textarea
					id="review-opinion"
					value={opinion}
					maxLength={2000}
					rows={3}
					disabled={busy || stale}
					aria-invalid={error ? "true" : "false"}
					aria-describedby={error ? "review-opinion-error" : undefined}
					onChange={(event) => {
						if (error && !stale) setError("");
						setOpinion(event.target.value);
						onOpinionChange?.(event.target.value);
					}}
				/>
			</FormField>
			<ConfirmDialog
				open={confirming !== null}
				busy={busy}
				title={activeMeta?.title ?? ""}
				description={
					confirming && opinion.trim()
						? `${activeMeta?.description ?? ""}\n办理意见：${opinion.trim()}`
						: activeMeta?.description
				}
				confirmLabel={activeMeta?.confirmLabel}
				tone={activeMeta?.tone}
				onCancel={() => {
					if (!busy) setConfirming(null);
				}}
				onConfirm={() => {
					if (confirming) void executeAction(confirming);
				}}
			/>
		</section>
	);
}
