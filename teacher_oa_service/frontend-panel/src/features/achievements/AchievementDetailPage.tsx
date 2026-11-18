import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useLocation, useParams } from "react-router-dom";
import { ArrowLeft, Clock, Folder, Pen, Refresh } from "reicon-react";
import { CorrectionNotice } from "@/components/achievements/CorrectionNotice";
import { fieldValue } from "@/components/achievements/DynamicFields";
import { MaterialList } from "@/components/achievements/MaterialList";
import { ParticipantList } from "@/components/achievements/ParticipantList";
import { ActionButton } from "@/components/common/ActionButton";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { DetailSkeleton } from "@/components/common/Skeleton";
import { ReviewActions } from "@/components/reviews/ReviewActions";
import { useAuth } from "@/features/auth/AuthProvider";
import { formatAppDateTime } from "@/services/dateFormat";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";
import {
	readLastAchievementListPath,
	safeReturnPath,
} from "@/services/returnPath";
import { useResource } from "@/services/useResource";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";
import {
	type AchievementInput,
	type AchievementSchema,
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
	const back = safeReturnPath(
		location.state?.from,
		readLastAchievementListPath("/achievements"),
	);
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
			const schemas: Record<number, AchievementSchema> = {
				[detail.schema.version]: detail.schema,
			};
			const extraVersions = Array.from(
				new Set(
					revisions
						.map((revision) => revision.schema_version)
						.filter(
							(version): version is number =>
								typeof version === "number" &&
								version > 0 &&
								version !== detail.schema.version,
						),
				),
			);
			await Promise.all(
				extraVersions.map(async (version) => {
					try {
						schemas[version] = await achievementApi.schema(
							detail.record.type_code,
							version,
							signal,
						);
					} catch {
						// Fallback to detail.schema if historical version lookup is unavailable.
					}
				}),
			);
			return { detail, catalog, revisions, schemas };
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
			notifySuccess("成果已提交审核");
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
	if (result.loading && !result.data)
		return <DetailSkeleton message="正在读取成果…" />;
	if (result.error || !result.data)
		return (
			<Feedback message={errorMessage(result.error)} onRetry={result.reload} />
		);
	const { detail, catalog, revisions, schemas } = result.data;
	const hasPanelPrimaryAction =
		(detail.can_edit && detail.record.status === "returned") ||
		detail.can_review ||
		detail.can_archive;

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
						back.startsWith("/reviews")
							? "返回审核列表"
							: back.startsWith("/teachers")
								? "返回教师资料"
								: "返回成果列表"
					}
				>
					<ArrowLeft />
				</Link>
				<IconButton
					label="刷新成果"
					disabled={busy || result.refreshing}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
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
							variant={hasPanelPrimaryAction ? "secondary" : "primary"}
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
				<span>当前第 {detail.record.version} 版</span>
			</div>
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			<div className="editor-sections">
				<CorrectionNotice
					detail={detail}
					editTo={detail.can_edit ? `/achievements/${id}/edit` : undefined}
					from={back}
				/>
				<ReviewActions
					key={`${id}-${detail.record.version}`}
					detail={detail}
					initialOpinion={opinion}
					onOpinionChange={(opinion) => setDraft({ key: draftKey, opinion })}
					onReload={result.reload}
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
						<EmptyState compact icon={<Folder />} title="尚未关联证明材料" />
					)}
				</section>
				<section className="panel">
					<div className="section-heading">
						<h2>办理记录</h2>
					</div>
					{detail.events.length ? (
						// biome-ignore lint/a11y/noRedundantRoles: Safari removes list semantics when list-style: none is applied.
						<ol className="event-list" role="list">
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
										{formatAppDateTime(event.created_at)}
									</time>
								</li>
							))}
						</ol>
					) : (
						<EmptyState
							compact
							icon={<Clock />}
							title="尚未提交审核"
							description="提交审核后，办理时间与审核意见将在此展示。"
						/>
					)}
				</section>
				{revisions.length > 0 && (
					<section className="panel">
						<div className="section-heading">
							<h2>历史版本</h2>
						</div>
						{revisions.map((revision) => {
							const revisionSchema =
								schemas[revision.schema_version] ?? detail.schema;
							return (
								<details className="revision-row" key={revision.version}>
									<summary>
										第 {revision.version} 版 ·{" "}
										{formatAppDateTime(revision.created_at)}
									</summary>
									<p>
										{revision.snapshot.data.title} ·{" "}
										{statusLabels[revision.snapshot.status]}
									</p>
									<dl className="achievement-fields">
										{revisionSchema.fields
											.filter(
												(field) =>
													field.visible &&
													!["file_list", "participant_list"].includes(
														field.type,
													),
											)
											.map((field) => (
												<div key={field.key}>
													<dt>{field.label}</dt>
													<dd>
														{display(revision.snapshot.data, field, catalog)}
													</dd>
												</div>
											))}
									</dl>
									<ParticipantList
										participants={revision.snapshot.data.participants ?? []}
										catalog={catalog}
									/>
									<MaterialList files={revision.files} />
								</details>
							);
						})}
					</section>
				)}
			</div>
		</>
	);
}
