import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useLocation, useParams } from "react-router-dom";
import { ArrowLeft, Pen, Refresh } from "reicon-react";
import { CorrectionNotice } from "@/components/achievements/CorrectionNotice";
import { fieldValue } from "@/components/achievements/DynamicFields";
import { MaterialList } from "@/components/achievements/MaterialList";
import { ParticipantList } from "@/components/achievements/ParticipantList";
import { ActionButton } from "@/components/common/ActionButton";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { ReviewActions } from "@/components/reviews/ReviewActions";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { safeReturnPath } from "@/services/returnPath";
import { useResource } from "@/services/useResource";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import {
	type AchievementInput,
	achievementApi,
	type Catalog,
	errorMessage,
	type FieldDefinition,
	statusLabels,
	validationErrors,
} from "./api";

function display(
	data: AchievementInput,
	field: FieldDefinition,
	catalog: Catalog,
) {
	const value = fieldValue(data, field.key);
	if (field.key === "academic_term_id")
		return catalog.terms.find((term) => term.id === value)?.name ?? "—";
	const label = (value: unknown) =>
		catalog.dictionaries.find(
			(item) => item.group_code === field.dictionary && item.code === value,
		)?.name ??
		(typeof value === "string" || typeof value === "number"
			? String(value)
			: "—");
	return Array.isArray(value)
		? value.map(label).join("、") || "—"
		: label(value);
}

function readOpinionDraft(key: string) {
	try {
		return key ? (window.sessionStorage.getItem(key) ?? "") : "";
	} catch {
		return "";
	}
}

export function AchievementDetailPage() {
	const { id } = useParams();
	const location = useLocation();
	const back = safeReturnPath(location.state?.from, "/achievements");
	const { session, refresh } = useAuth();
	const draftKey =
		session && id ? `teacher-oa:review-opinion:${session.user.id}:${id}` : "";
	const [draft, setDraft] = useState(() => ({
		key: draftKey,
		opinion: readOpinionDraft(draftKey),
	}));
	const opinion = draft.key === draftKey ? draft.opinion : "";
	const load = useCallback(
		async (signal: AbortSignal) => {
			const [detail, catalog, revisions] = await Promise.all([
				achievementApi.detail(Number(id), signal),
				achievementApi.catalog(signal),
				achievementApi.revisions(Number(id), signal),
			]);
			return { detail, catalog, revisions };
		},
		[id],
	);
	const result = useResource(load);
	useEffect(() => {
		if (draftKey && draft.key !== draftKey) {
			setDraft({ key: draftKey, opinion: readOpinionDraft(draftKey) });
		}
	}, [draftKey, draft.key]);
	useEffect(() => {
		if (!draftKey || draft.key !== draftKey) return;
		try {
			if (draft.opinion) window.sessionStorage.setItem(draftKey, draft.opinion);
			else window.sessionStorage.removeItem(draftKey);
		} catch {
			return;
		}
	}, [draftKey, draft]);
	const [busy, setBusy] = useState(false);
	const [failure, setFailure] = useState<{
		id: string | undefined;
		message: string;
	}>({ id, message: "" });
	const error = failure.id === id ? failure.message : "";
	const setError = (message: string) => setFailure({ id, message });
	const pending = useRef(false);
	useUnsavedChanges(Boolean(opinion), busy);
	const submit = async () => {
		if (!result.data || !session || pending.current) return;
		pending.current = true;
		setBusy(true);
		setError("");
		try {
			await achievementApi.submit(
				Number(id),
				result.data.detail.record.version,
				session.csrf_token,
			);
			result.reload();
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			const fields = Object.values(validationErrors(error));
			setError(fields.length ? fields.join("；") : errorMessage(error));
		} finally {
			pending.current = false;
			setBusy(false);
		}
	};
	if (result.loading) return <Feedback loading message="正在读取成果…" />;
	if (result.error || !result.data)
		return (
			<Feedback message={errorMessage(result.error)} onRetry={result.reload} />
		);
	const { detail, catalog, revisions } = result.data;
	return (
		<>
			<PageHeading
				title={detail.record.title}
				eyebrow={detail.record.type_name}
			>
				<Link
					to={back}
					className="icon-button"
					aria-label={
						back.startsWith("/reviews") ? "返回审核列表" : "返回成果列表"
					}
				>
					<ArrowLeft />
				</Link>
				<IconButton label="刷新成果" disabled={busy} onClick={result.reload}>
					<Refresh />
				</IconButton>
				{detail.can_edit && (
					<>
						<Link
							className="icon-button"
							to={`/achievements/${id}/edit`}
							state={{ from: back }}
							aria-label="编辑成果"
							title="编辑成果"
						>
							<Pen />
						</Link>
						<ActionButton
							variant="primary"
							disabled={busy}
							onClick={() => {
								void submit();
							}}
						>
							提交审核
						</ActionButton>
					</>
				)}
			</PageHeading>
			<div className="record-meta">
				<span className={`achievement-status status-${detail.record.status}`}>
					{statusLabels[detail.record.status]}
				</span>
				<span>{detail.record.teacher_name}</span>
				<span>第 {detail.record.version} 次修订</span>
			</div>
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			<div className="editor-sections">
				<CorrectionNotice detail={detail} />
				<ReviewActions
					key={`${id}-${detail.record.version}`}
					detail={detail}
					initialOpinion={opinion}
					onOpinionChange={(opinion) => setDraft({ key: draftKey, opinion })}
					onComplete={() => {
						setDraft({ key: draftKey, opinion: "" });
						result.reload();
					}}
					onBusy={setBusy}
				/>
				<section className="panel">
					<div className="section-heading">
						<h2>成果信息</h2>
					</div>
					<dl className="achievement-fields">
						{detail.schema.fields
							.filter(
								(field) =>
									field.visible &&
									!["file_list", "participant_list"].includes(field.type),
							)
							.map((field) => (
								<div key={field.key}>
									<dt>{field.label}</dt>
									<dd>{display(detail.data, field, catalog)}</dd>
								</div>
							))}
					</dl>
				</section>
				{Boolean(detail.data.participants?.length) && (
					<section className="panel">
						<div className="section-heading">
							<h2>相关人员与署名顺序</h2>
						</div>
						<ParticipantList
							participants={detail.data.participants ?? []}
							catalog={catalog}
						/>
					</section>
				)}
				<section className="panel">
					<div className="section-heading">
						<h2>证明材料</h2>
					</div>
					{detail.files.length ? (
						<MaterialList files={detail.files} />
					) : (
						<p className="empty-message">尚未关联证明材料</p>
					)}
				</section>
				<section className="panel">
					<div className="section-heading">
						<h2>办理记录</h2>
					</div>
					{detail.events.length ? (
						<ol className="event-list">
							{detail.events.map((event) => (
								<li key={event.id}>
									<strong>
										{event.action === "submit"
											? "已提交审核"
											: statusLabels[event.new_status]}
									</strong>
									<span>{event.reviewer_name}</span>
									{event.opinion && <p>{event.opinion}</p>}
									<time dateTime={event.created_at}>
										{new Date(event.created_at).toLocaleString("zh-CN", {
											timeZone: "Asia/Singapore",
										})}
									</time>
								</li>
							))}
						</ol>
					) : (
						<p className="empty-message">尚未提交审核</p>
					)}
				</section>
				<section className="panel">
					<div className="section-heading">
						<h2>历史修订</h2>
					</div>
					{revisions.map((revision) => (
						<details className="revision-row" key={revision.version}>
							<summary>
								第 {revision.version} 次修订 ·{" "}
								{new Date(revision.created_at).toLocaleString("zh-CN", {
									timeZone: "Asia/Singapore",
								})}
							</summary>
							<p>
								{revision.snapshot.data.title} ·{" "}
								{statusLabels[revision.snapshot.status]}
							</p>
							<dl className="achievement-fields">
								{detail.schema.fields
									.filter(
										(field) =>
											field.visible &&
											!["file_list", "participant_list"].includes(field.type),
									)
									.map((field) => (
										<div key={field.key}>
											<dt>{field.label}</dt>
											<dd>{display(revision.snapshot.data, field, catalog)}</dd>
										</div>
									))}
							</dl>
							<ParticipantList
								participants={revision.snapshot.data.participants ?? []}
								catalog={catalog}
							/>
							<MaterialList files={revision.files} />
						</details>
					))}
				</section>
			</div>
		</>
	);
}
