import { type FormEvent, useEffect, useRef, useState } from "react";
import {
	Link,
	useLocation,
	useNavigate,
	useParams,
	useSearchParams,
} from "react-router-dom";
import { ArrowLeft, Refresh } from "reicon-react";
import { CorrectionNotice } from "@/components/achievements/CorrectionNotice";
import {
	DynamicFields,
	fieldErrorKey,
	fieldValue,
} from "@/components/achievements/DynamicFields";
import { MaterialUpload } from "@/components/achievements/MaterialUpload";
import { ParticipantsEditor } from "@/components/achievements/ParticipantsEditor";
import { ActionButton } from "@/components/common/ActionButton";
import { Feedback } from "@/components/common/Feedback";
import { PageHeading } from "@/components/common/PageHeading";
import { FormSkeleton } from "@/components/common/Skeleton";
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
	type AchievementDetail,
	type AchievementInput,
	type AchievementSchema,
	achievementApi,
	errorMessage,
	type FileMetadata,
	validationErrors,
} from "./api";
import { draftFingerprint } from "./draftFingerprint";

type StoredAchievementDraft = {
	data: AchievementInput;
	files: FileMetadata[];
	savedAt: string;
};

function buildDraftStorageKey(
	userId: number | undefined,
	id: string | undefined,
	code: string,
) {
	if (!userId) return "";
	return `teacher-oa:achievement-draft:${userId}:${id ? `edit:${id}` : `new:${code}`}`;
}

function readStoredDraft(key: string): StoredAchievementDraft | null {
	if (!key || typeof window === "undefined") return null;
	try {
		const raw = window.localStorage.getItem(key);
		if (!raw) return null;
		const parsed = JSON.parse(raw) as StoredAchievementDraft;
		if (!parsed || typeof parsed !== "object" || !parsed.data) return null;
		return parsed;
	} catch {
		return null;
	}
}

function clearStoredDraft(key: string) {
	if (!key || typeof window === "undefined") return;
	try {
		window.localStorage.removeItem(key);
	} catch {
		// Ignore storage errors.
	}
}

export function emptyDraft(schema: AchievementSchema): AchievementInput {
	return {
		type_code: schema.code,
		schema_version: schema.version,
		title: "",
		attributes: {},
		remarks: "",
		participants: [],
		file_ids: [],
		term_mismatch_confirmed: false,
	};
}

export function AchievementEditorPage() {
	const { id } = useParams();
	const [params, setParams] = useSearchParams();
	const navigate = useNavigate();
	const location = useLocation();
	const back = safeReturnPath(
		location.state?.from,
		readLastAchievementListPath("/achievements"),
	);
	const { session, refresh } = useAuth();
	const catalog = useResource(achievementApi.catalog);
	const code = params.get("type") ?? "open_class";
	const storageKey = buildDraftStorageKey(session?.user.id, id, code);
	const [schema, setSchema] = useState<AchievementSchema | null>(null);
	const [data, setData] = useState<AchievementInput | null>(null);
	const [saved, setSaved] = useState<AchievementDetail | null>(null);
	const [files, setFiles] = useState<FileMetadata[]>([]);
	const [loading, setLoading] = useState(true);
	const [loadError, setLoadError] = useState("");
	const [busy, setBusy] = useState(false);
	const [uploading, setUploading] = useState(false);
	const [error, setError] = useState("");
	const [errors, setErrors] = useState<Record<string, string>>({});
	const [baseline, setBaseline] = useState("");
	const [conflict, setConflict] = useState(false);
	const [recoverableDraft, setRecoverableDraft] =
		useState<StoredAchievementDraft | null>(null);
	const [autoSavedAt, setAutoSavedAt] = useState("");
	const pending = useRef(false);
	const savedId = useRef<number | null>(null);
	const form = useRef<HTMLFormElement>(null);
	const dirty = data !== null && draftFingerprint(data) !== baseline;
	const guard = useUnsavedChanges(!loading && dirty, busy || uploading);

	useEffect(() => {
		if (id && savedId.current === Number(id)) return;
		const controller = new AbortController();
		setLoading(true);
		setLoadError("");
		setError("");
		setErrors({});
		setConflict(false);
		setAutoSavedAt("");
		setRecoverableDraft(null);
		const load = async () => {
			try {
				if (id) {
					const detail = await achievementApi.detail(
						Number(id),
						controller.signal,
					);
					if (controller.signal.aborted) return;
					savedId.current = detail.record.id;
					setSaved(detail);
					setData(detail.data);
					const baseFp = draftFingerprint(detail.data);
					setBaseline(baseFp);
					setFiles(detail.files);
					setSchema(detail.schema);
					const cached = readStoredDraft(storageKey);
					if (cached && draftFingerprint(cached.data) !== baseFp) {
						setRecoverableDraft(cached);
					}
				} else {
					const next = await achievementApi.schema(
						code,
						undefined,
						controller.signal,
					);
					if (controller.signal.aborted) return;
					savedId.current = null;
					setSchema(next);
					setSaved(null);
					const draft = emptyDraft(next);
					const baseFp = draftFingerprint(draft);
					setData(draft);
					setBaseline(baseFp);
					setFiles([]);
					const cached = readStoredDraft(storageKey);
					if (cached && draftFingerprint(cached.data) !== baseFp) {
						setRecoverableDraft(cached);
					}
				}
			} catch (error) {
				if (!controller.signal.aborted) {
					setLoadError(errorMessage(error));
					if (error instanceof HttpError && error.status === 401)
						void refresh();
				}
			} finally {
				if (!controller.signal.aborted) setLoading(false);
			}
		};
		void load();
		return () => controller.abort();
	}, [id, code, refresh, storageKey]);

	useEffect(() => {
		if (loading || !storageKey || !data) return;
		if (!dirty) return;
		const timer = setTimeout(() => {
			try {
				const savedAt = new Date().toISOString();
				const payload: StoredAchievementDraft = {
					data,
					files,
					savedAt,
				};
				window.localStorage.setItem(storageKey, JSON.stringify(payload));
				setAutoSavedAt(savedAt);
			} catch {
				// Ignore localStorage quota failures.
			}
		}, 400);
		return () => clearTimeout(timer);
	}, [loading, storageKey, data, files, dirty]);

	useEffect(() => {
		if (Object.keys(errors).length)
			form.current
				?.querySelector<HTMLElement>('[aria-invalid="true"]')
				?.focus();
	}, [errors]);

	const save = async (submit: boolean) => {
		if (pending.current || busy || uploading || !data || !session) return;
		if (!submit) {
			if (!data.title.trim()) {
				setErrors({ title: "请填写成果名称" });
				setError("请先填写成果名称再保存草稿。");
				return;
			}
		} else if (!form.current?.reportValidity()) {
			return;
		}
		if (submit && schema) {
			const missing: Record<string, string> = {};
			for (const field of schema.fields.filter(
				(field) => field.visible && field.required,
			)) {
				const value = fieldValue(data, field.key);
				if (
					value == null ||
					(typeof value === "string" && !value.trim()) ||
					(Array.isArray(value) && value.length === 0)
				) {
					missing[fieldErrorKey(field.key)] = `请填写${field.label}`;
				}
			}
			if (Object.keys(missing).length) {
				setErrors(missing);
				setError("请补全必填内容后提交。");
				return;
			}
		}
		pending.current = true;
		setBusy(true);
		setError("");
		setErrors({});
		setConflict(false);
		let persisted = saved && !dirty ? saved : null;
		try {
			let detail =
				saved && !dirty
					? saved
					: saved
						? await achievementApi.update(
								saved.record.id,
								saved.record.version,
								data,
								session.csrf_token,
							)
						: await achievementApi.create(data, session.csrf_token);
			savedId.current = detail.record.id;
			setSaved(detail);
			setData(detail.data);
			setFiles(detail.files);
			setBaseline(draftFingerprint(detail.data));
			persisted = detail;
			if (submit) {
				detail = await achievementApi.submit(
					detail.record.id,
					detail.record.version,
					session.csrf_token,
				);
				setSaved(detail);
				setFiles(detail.files);
			}
			clearStoredDraft(storageKey);
			setRecoverableDraft(null);
			setAutoSavedAt("");
			notifySuccess(submit ? "成果已提交审核" : "草稿已保存");
			guard.allowLeave();
			navigate(`/achievements/${detail.record.id}`, { state: { from: back } });
		} catch (error) {
			setError(
				`${submit && persisted ? "填报内容已保存。" : ""}${errorMessage(error)}`,
			);
			setErrors(validationErrors(error));
			setConflict(error instanceof HttpError && error.status === 409);
			if (!id && persisted) {
				clearStoredDraft(storageKey);
				guard.allowLeave();
				navigate(`/achievements/${persisted.record.id}/edit`, {
					replace: true,
					state: { from: back },
				});
			}
			if (
				error &&
				typeof error === "object" &&
				"status" in error &&
				error.status === 401
			)
				void refresh();
		} finally {
			pending.current = false;
			setBusy(false);
		}
	};
	const submit = (event: FormEvent) => {
		event.preventDefault();
		void save(true);
	};
	if (loading || catalog.loading)
		return <FormSkeleton message="正在读取填报信息…" />;
	if (
		loadError ||
		catalog.error ||
		!schema ||
		!data ||
		!catalog.data ||
		!session
	)
		return (
			<Feedback
				message={loadError || "填报信息暂时无法读取，请刷新后重试。"}
				onRetry={() => window.location.reload()}
			/>
		);
	if (saved && !saved.can_edit)
		return <Feedback message="当前成果状态不能修改。" />;
	const catalogData = catalog.data;
	const inlineErrorKeys = new Set(
		schema.fields
			.filter((field) => field.visible)
			.map((field) => fieldErrorKey(field.key)),
	);
	const unmappedErrors = Object.entries(errors).filter(
		([key]) =>
			!inlineErrorKeys.has(key) &&
			key !== "file_ids" &&
			key !== "participants" &&
			!key.startsWith("participants."),
	);
	const hasInlineFieldAlerts =
		Object.keys(errors).length > unmappedErrors.length;
	return (
		<>
			<PageHeading
				title={id || saved ? "编辑成果" : "新建成果"}
				eyebrow={schema.name}
			>
				{autoSavedAt && dirty && (
					<span className="autosave-status" role="status">
						本地已暂存
					</span>
				)}
				<Link
					to={saved ? `/achievements/${saved.record.id}` : back}
					state={{ from: back }}
					className="icon-button"
					aria-label={
						saved
							? "返回成果详情"
							: back.startsWith("/reviews")
								? "返回审核列表"
								: "返回成果列表"
					}
				>
					<ArrowLeft />
				</Link>
				<ActionButton
					disabled={busy || uploading}
					onClick={() => {
						void save(false);
					}}
				>
					保存草稿
				</ActionButton>
				<ActionButton
					type="submit"
					form="achievement-form"
					variant="primary"
					disabled={busy || uploading}
				>
					提交审核
				</ActionButton>
			</PageHeading>
			{recoverableDraft && (
				<div className="draft-recovery-banner" role="status">
					<span>
						检测到上次未提交的本地暂存草稿（保存于{" "}
						{formatAppDateTime(recoverableDraft.savedAt)}），是否恢复？
					</span>
					<div className="row-actions">
						<ActionButton
							variant="secondary"
							size="sm"
							onClick={() => {
								setData(recoverableDraft.data);
								setFiles(recoverableDraft.files ?? []);
								setRecoverableDraft(null);
							}}
						>
							恢复暂存草稿
						</ActionButton>
						<ActionButton
							variant="tertiary"
							size="sm"
							onClick={() => {
								clearStoredDraft(storageKey);
								setRecoverableDraft(null);
							}}
						>
							忽略暂存
						</ActionButton>
					</div>
				</div>
			)}
			{saved && <CorrectionNotice detail={saved} />}
			{busy && (
				<p className="form-note" role="status">
					正在保存成果…
				</p>
			)}
			{error && (
				<div
					className="validation-summary"
					role={hasInlineFieldAlerts ? "status" : "alert"}
				>
					<p>{error}</p>
					{unmappedErrors.map(([key, message]) => (
						<p key={key}>{message}</p>
					))}
					{conflict && (
						<div className="row-actions">
							<ActionButton
								variant="secondary"
								size="sm"
								aria-label="重新加载成果"
								onClick={() => {
									void guard.confirmLeave().then((allowed) => {
										if (!allowed) return;
										clearStoredDraft(storageKey);
										guard.allowLeave();
										window.location.assign(
											saved
												? `/achievements/${saved.record.id}/edit`
												: window.location.href,
										);
									});
								}}
							>
								<Refresh aria-hidden="true" />
								重新加载成果
							</ActionButton>
						</div>
					)}
				</div>
			)}
			<form
				id="achievement-form"
				ref={form}
				onSubmit={submit}
				className="editor-sections"
			>
				<section className="panel">
					<div className="section-heading">
						<h2>成果基本信息</h2>
						{!id && !saved && (
							<select
								aria-label="成果类型"
								value={code}
								disabled={busy || uploading}
								onChange={(event) => {
									const next = new URLSearchParams(params);
									next.set("type", event.target.value);
									setParams(next);
								}}
							>
								{catalogData.types.map((type) => (
									<option key={type.code} value={type.code}>
										{type.name}
									</option>
								))}
							</select>
						)}
					</div>
					<DynamicFields
						fields={schema.fields}
						data={data}
						catalog={catalogData}
						disabled={busy || uploading}
						errors={errors}
						onChange={(nextData) => {
							if (recoverableDraft) setRecoverableDraft(null);
							setData(nextData);
						}}
						onClearError={(key) =>
							setErrors((previous) => {
								if (!(key in previous)) return previous;
								const next = { ...previous };
								delete next[key];
								return next;
							})
						}
					/>
					{data.occurred_at &&
						data.academic_term_id &&
						(() => {
							const term = catalogData.terms.find(
								(item) => item.id === data.academic_term_id,
							);
							return term &&
								(data.occurred_at < term.starts_on ||
									data.occurred_at > term.ends_on) ? (
								<label className="confirmation-row">
									<input
										type="checkbox"
										disabled={busy || uploading}
										checked={data.term_mismatch_confirmed ?? false}
										onChange={(event) => {
											setErrors((previous) => {
												if (!("term_mismatch_confirmed" in previous))
													return previous;
												const next = { ...previous };
												delete next.term_mismatch_confirmed;
												return next;
											});
											setData({
												...data,
												term_mismatch_confirmed: event.target.checked,
											});
										}}
									/>
									<span>日期不在所选学期，我已核对归属学期。</span>
								</label>
							) : null;
						})()}
				</section>
				{schema.fields
					.filter((field) => field.type === "participant_list" && field.visible)
					.map((field) => (
						<ParticipantsEditor
							key={field.key}
							value={data.participants ?? []}
							field={field}
							catalog={catalogData}
							errors={errors}
							disabled={busy || uploading}
							onChange={(participants) => {
								setErrors((previous) => {
									const next = { ...previous };
									let changed = false;
									for (const key of Object.keys(next)) {
										if (
											key === "participants" ||
											key.startsWith("participants.")
										) {
											delete next[key];
											changed = true;
										}
									}
									return changed ? next : previous;
								});
								setData({ ...data, participants });
							}}
						/>
					))}
				<MaterialUpload
					files={files}
					csrf={session.csrf_token}
					fieldError={errors.file_ids}
					onSessionExpired={refresh}
					disabled={busy}
					onBusy={setUploading}
					onAdd={(file) => {
						setErrors((previous) => {
							const next = { ...previous };
							delete next.file_ids;
							return next;
						});
						setFiles((previous) => [...previous, file]);
						setData((previous) =>
							previous
								? {
										...previous,
										file_ids: [...(previous.file_ids ?? []), file.id],
									}
								: previous,
						);
					}}
					onRemove={(id) => {
						setFiles((previous) => previous.filter((file) => file.id !== id));
						setData((previous) =>
							previous
								? {
										...previous,
										file_ids: previous.file_ids?.filter(
											(fileId) => fileId !== id,
										),
									}
								: previous,
						);
					}}
				/>
			</form>
		</>
	);
}
