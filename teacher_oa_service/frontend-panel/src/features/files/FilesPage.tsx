import { type DragEvent, useCallback, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Camera, Folder, Refresh, Upload } from "reicon-react";
import { MaterialList } from "@/components/achievements/MaterialList";
import { ActionButton } from "@/components/common/ActionButton";
import { useConfirm } from "@/components/common/ConfirmDialog";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { TableSkeleton } from "@/components/common/Skeleton";
import { formatUploadLimitMb, maxUploadBytes } from "@/config/app";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { notifySuccess } from "@/services/notify";
import { useResource } from "@/services/useResource";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";

export function FilesPage() {
	const { session, refresh } = useAuth();
	const confirm = useConfirm();
	const [searchParams, setSearchParams] = useSearchParams();
	const rawPage = Number(searchParams.get("page") ?? "1");
	const page = Number.isInteger(rawPage) && rawPage >= 1 ? rawPage : 1;
	const rawPageSize = Number(searchParams.get("page_size") ?? "20");
	const pageSize = [10, 20, 50].includes(rawPageSize) ? rawPageSize : 20;
	const setPage = useCallback(
		(nextPage: number, nextPageSize = pageSize) => {
			const next = new URLSearchParams(searchParams);
			if (nextPage <= 1) next.delete("page");
			else next.set("page", String(nextPage));
			if (nextPageSize === 20) next.delete("page_size");
			else next.set("page_size", String(nextPageSize));
			setSearchParams(next);
		},
		[pageSize, searchParams, setSearchParams],
	);
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.files(page, pageSize, signal),
		[page, pageSize],
	);
	const result = useResource(load);
	const input = useRef<HTMLInputElement>(null);
	const cameraInput = useRef<HTMLInputElement>(null);
	const [busy, setBusy] = useState(false);
	const [deletingId, setDeletingId] = useState<number | null>(null);
	const [progressText, setProgressText] = useState("");
	const [dragging, setDragging] = useState(false);
	const [error, setError] = useState("");
	const pending = useRef(false);
	const canUpload = Boolean(
		session?.user.grants.some((grant) => grant.permission === "files:write"),
	);
	const canManageAll = Boolean(
		session?.user.grants.some((grant) => grant.permission === "admin:manage"),
	);
	const limitBytes = session?.max_upload_bytes ?? maxUploadBytes;
	const limitLabel = formatUploadLimitMb(limitBytes);
	useUnsavedChanges(false, busy || deletingId !== null);

	const canDeleteFile = (file: FileMetadata) =>
		canUpload &&
		file.association === "unlinked" &&
		(file.uploaded_by === session?.user.id || canManageAll);

	const handleDeleteFile = async (file: FileMetadata) => {
		if (!session || busy || deletingId !== null) return;
		const ok = await confirm({
			title: "删除证明材料",
			description: `确认永久删除“${file.filename}”？此操作不可撤销。`,
			confirmLabel: "确认删除",
			tone: "danger",
		});
		if (!ok) return;
		setDeletingId(file.id);
		setError("");
		try {
			await achievementApi.deleteFile(file.id, session.csrf_token);
			result.reload();
			notifySuccess("材料已删除");
		} catch (err) {
			if (err instanceof HttpError && err.status === 401) {
				void refresh();
				return;
			}
			setError(errorMessage(err));
		} finally {
			setDeletingId(null);
		}
	};

	const uploadSelection = async (selection: FileList | null | undefined) => {
		if (!selection?.length || !session || pending.current) return;
		const files = Array.from(selection);
		const validFiles: File[] = [];
		const failures: string[] = [];

		for (const file of files) {
			if (file.size > limitBytes) {
				failures.push(
					`${file.name}：文件超过 ${limitLabel} 大小限制，请压缩后重试。`,
				);
			} else {
				validFiles.push(file);
			}
		}

		if (!validFiles.length) {
			setError(failures.join("；"));
			if (input.current) input.current.value = "";
			if (cameraInput.current) cameraInput.current.value = "";
			return;
		}

		pending.current = true;
		setBusy(true);
		setError("");
		let uploadedCount = 0;
		try {
			for (let idx = 0; idx < validFiles.length; idx += 1) {
				const file = validFiles[idx];
				setProgressText(
					validFiles.length > 1
						? `正在上传第 ${idx + 1} / ${validFiles.length} 份材料（${file.name}）…`
						: `材料上传中（${file.name}）…`,
				);
				try {
					await achievementApi.upload(file, session.csrf_token);
					uploadedCount += 1;
				} catch (error) {
					if (error instanceof HttpError && error.status === 401) {
						void refresh();
						break;
					}
					failures.push(
						`${file.name}：${errorMessage(error, "上传失败，请重新选择后重试。")}`,
					);
				}
			}
			if (uploadedCount > 0) {
				setPage(1);
				result.reload();
				notifySuccess(
					uploadedCount > 1
						? `已成功上传 ${uploadedCount} 份证明材料`
						: "证明材料已上传",
				);
			}
			setError(failures.join("；"));
		} finally {
			pending.current = false;
			setBusy(false);
			setProgressText("");
			if (input.current) input.current.value = "";
			if (cameraInput.current) cameraInput.current.value = "";
		}
	};

	const handleDragOver = (event: DragEvent<HTMLElement>) => {
		if (!canUpload || busy) return;
		event.preventDefault();
		if (!dragging) setDragging(true);
	};
	const handleDragLeave = (event: DragEvent<HTMLElement>) => {
		if (event.currentTarget.contains(event.relatedTarget as Node | null))
			return;
		setDragging(false);
	};
	const handleDrop = (event: DragEvent<HTMLElement>) => {
		event.preventDefault();
		setDragging(false);
		if (!canUpload || busy) return;
		void uploadSelection(event.dataTransfer.files);
	};
	return (
		<>
			<PageHeading title="文件资料">
				<IconButton
					label="刷新材料列表"
					disabled={result.loading || result.refreshing || busy}
					onClick={result.reload}
				>
					<Refresh className={result.refreshing ? "spin" : undefined} />
				</IconButton>
				{canUpload && (
					<>
						<ActionButton
							variant="secondary"
							disabled={busy}
							onClick={() => cameraInput.current?.click()}
						>
							<Camera aria-hidden="true" />
							拍照上传
						</ActionButton>
						<ActionButton
							variant="primary"
							disabled={busy}
							onClick={() => input.current?.click()}
						>
							<Upload aria-hidden="true" />
							上传材料
						</ActionButton>
					</>
				)}
			</PageHeading>
			<input
				type="file"
				ref={input}
				className="visually-hidden"
				tabIndex={-1}
				aria-label="选择上传材料"
				accept=".pdf,.png,.jpg,.jpeg"
				multiple
				disabled={busy}
				onChange={(event) => {
					void uploadSelection(event.target.files);
				}}
			/>
			<input
				type="file"
				ref={cameraInput}
				className="visually-hidden"
				tabIndex={-1}
				aria-label="拍照上传材料"
				accept="image/jpeg,image/png"
				capture="environment"
				disabled={busy}
				onChange={(event) => {
					void uploadSelection(event.target.files);
				}}
			/>
			{busy && (
				<p className="form-note" role="status">
					{progressText || "材料上传中…"}
				</p>
			)}
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			{result.loading && !result.data ? (
				<TableSkeleton message="正在读取材料…" />
			) : result.error ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				result.data && (
					// biome-ignore lint/a11y/noStaticElementInteractions: file drop target paired with the labelled keyboard upload button.
					<section
						className={`panel material-upload-panel${dragging ? " is-dragging" : ""}`}
						onDragOver={handleDragOver}
						onDragLeave={handleDragLeave}
						onDrop={handleDrop}
					>
						<div className="section-heading">
							<h2>材料列表</h2>
							<span className="section-meta">共 {result.data.total} 份</span>
						</div>
						{result.data.items.length ? (
							<MaterialList
								files={result.data.items}
								canDeleteFile={canDeleteFile}
								onDeleteFile={(file) => {
									void handleDeleteFile(file);
								}}
								deletingId={deletingId}
							/>
						) : canUpload ? (
							// biome-ignore lint/a11y/noStaticElementInteractions: keyboard users activate upload via the labelled button in PageHeading.
							// biome-ignore lint/a11y/useKeyWithClickEvents: pointer/drag convenience target paired with the accessible upload button above.
							<div
								className="upload-dropzone"
								onClick={() => !busy && input.current?.click()}
							>
								<span className="upload-dropzone-icon" aria-hidden="true">
									<Upload />
								</span>
								<p className="upload-dropzone-title">暂无可查看的材料。</p>
								<small className="upload-dropzone-hint">
									点击此处或拖拽 PDF、JPG、PNG
									文件上传证明材料（支持多选，单文件 ≤ {limitLabel}）
								</small>
							</div>
						) : (
							<EmptyState
								icon={<Folder />}
								title="暂无可查看的材料。"
								description="关联到您有权查看的成果的证明材料会展示在此处。"
							/>
						)}
						<Pagination
							page={result.data.page}
							pageSize={result.data.page_size}
							total={result.data.total}
							onChange={(nextPage) => setPage(nextPage)}
							onPageSizeChange={(nextSize) => setPage(1, nextSize)}
						/>
					</section>
				)
			)}
		</>
	);
}
