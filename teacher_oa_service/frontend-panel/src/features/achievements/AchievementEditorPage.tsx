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
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { safeReturnPath } from "@/services/returnPath";
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
	const back = safeReturnPath(location.state?.from, "/achievements");
	const { session, refresh } = useAuth();
	const catalog = useResource(achievementApi.catalog);
	const code = params.get("type") ?? "open_class";
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
	const pending = useRef(false);
	const form = useRef<HTMLFormElement>(null);
	const dirty = data !== null && draftFingerprint(data) !== baseline;
	const guard = useUnsavedChanges(!loading && dirty, busy || uploading);
	useEffect(() => {
		const controller = new AbortController();
		setLoading(true);
		setLoadError("");
		setError("");
		setErrors({});
		setConflict(false);
		const load = async () => {
			try {
				if (id) {
					const detail = await achievementApi.detail(
						Number(id),
						controller.signal,
					);
					if (controller.signal.aborted) return;
					setSaved(detail);
					setData(detail.data);
					setBaseline(draftFingerprint(detail.data));
					setFiles(detail.files);
					setSchema(detail.schema);
				} else {
					const next = await achievementApi.schema(
						code,
						undefined,
						controller.signal,
					);
					if (controller.signal.aborted) return;
					setSchema(next);
					setSaved(null);
					const draft = emptyDraft(next);
					setData(draft);
					setBaseline(draftFingerprint(draft));
					setFiles([]);
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
	}, [id, code, refresh]);
	useEffect(() => {
		if (Object.keys(errors).length)
			form.current
				?.querySelector<HTMLElement>('[aria-invalid="true"]')
				?.focus();
	}, [errors]);
	const save = async (submit: boolean) => {
		if (
			pending.current ||
			busy ||
			uploading ||
			!data ||
			!session ||
			!form.current?.reportValidity()
		)
			return;
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
			setSaved(detail);
			setData(detail.data);
			setBaseline(draftFingerprint(detail.data));
			persisted = detail;
			if (submit) {
				detail = await achievementApi.submit(
					detail.record.id,
					detail.record.version,
					session.csrf_token,
				);
				setSaved(detail);
			}
			guard.allowLeave();
			navigate(`/achievements/${detail.record.id}`, { state: { from: back } });
		} catch (error) {
			setError(
				`${submit && persisted ? "填报内容已保存。" : ""}${errorMessage(error)}`,
			);
			setErrors(validationErrors(error));
			setConflict(error instanceof HttpError && error.status === 409);
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
		return <Feedback loading message="正在读取填报信息…" />;
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
	return (
		<>
			<PageHeading
				title={id || saved ? "编辑成果" : "新建成果"}
				eyebrow={schema.name}
			>
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
			{saved && <CorrectionNotice detail={saved} />}
			{busy && (
				<p className="form-note" role="status">
					正在保存成果…
				</p>
			)}
			{error && (
				<div className="validation-summary" role="alert">
					<p>{error}</p>
					{Object.entries(errors).map(([key, message]) => (
						<p key={key}>{message}</p>
					))}
					{conflict && (
						<IconButton
							label="重新加载成果"
							onClick={() => {
								if (!guard.confirmLeave()) return;
								guard.allowLeave();
								window.location.assign(
									saved
										? `/achievements/${saved.record.id}/edit`
										: window.location.href,
								);
							}}
						>
							<Refresh />
						</IconButton>
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
						onChange={setData}
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
										onChange={(event) =>
											setData({
												...data,
												term_mismatch_confirmed: event.target.checked,
											})
										}
									/>
									日期不在所选学期，我已核对归属学期。
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
							onChange={(participants) => setData({ ...data, participants })}
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
