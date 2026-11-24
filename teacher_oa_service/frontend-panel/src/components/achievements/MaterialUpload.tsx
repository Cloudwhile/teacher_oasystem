import { type DragEvent, useRef, useState } from "react";
import { Camera, FolderOpen, Upload } from "reicon-react";
import { EmptyState } from "@/components/common/EmptyState";
import { IconButton } from "@/components/common/IconButton";
import { formatUploadLimitMb, maxUploadBytes } from "@/config/app";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
import { useOptionalAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { ExistingMaterials } from "./ExistingMaterials";
import { MaterialList } from "./MaterialList";

export function MaterialUpload({
	files,
	csrf,
	disabled,
	onAdd,
	onRemove,
	onBusy,
	fieldError,
	onSessionExpired,
}: {
	files: FileMetadata[];
	csrf: string;
	disabled: boolean;
	onAdd: (file: FileMetadata) => void;
	onRemove: (id: number) => void;
	onBusy: (busy: boolean) => void;
	fieldError?: string;
	onSessionExpired?: () => void;
}) {
	const auth = useOptionalAuth();
	const limitBytes = auth?.session?.max_upload_bytes ?? maxUploadBytes;
	const limitLabel = formatUploadLimitMb(limitBytes);
	const input = useRef<HTMLInputElement>(null);
	const cameraInput = useRef<HTMLInputElement>(null);
	const [error, setError] = useState("");
	const [picking, setPicking] = useState(false);
	const [uploading, setUploading] = useState(false);
	const [progressCount, setProgressCount] = useState<{
		current: number;
		total: number;
	} | null>(null);
	const [dragging, setDragging] = useState(false);
	const pending = useRef(false);
	const blocked = disabled || uploading || files.length >= 20;
	const upload = async (selection: FileList | null) => {
		if (!selection?.length || disabled || pending.current) return;
		if (files.length + selection.length > 20) {
			setError("每个成果最多关联20份材料。");
			if (input.current) input.current.value = "";
			if (cameraInput.current) cameraInput.current.value = "";
			return;
		}
		const chosen = Array.from(selection);
		const validFiles: File[] = [];
		const failures: string[] = [];
		for (const file of chosen) {
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
		setError("");
		setUploading(true);
		pending.current = true;
		onBusy(true);
		try {
			for (let idx = 0; idx < validFiles.length; idx += 1) {
				const file = validFiles[idx];
				setProgressCount(
					validFiles.length > 1
						? { current: idx + 1, total: validFiles.length }
						: null,
				);
				try {
					onAdd(await achievementApi.upload(file, csrf));
				} catch (error) {
					failures.push(
						`${file.name}：${errorMessage(error, "材料上传失败，请重新选择后重试。")}`,
					);
					if (error instanceof HttpError && error.status === 401) {
						onSessionExpired?.();
						break;
					}
				}
			}
			setError(failures.join("；"));
		} finally {
			pending.current = false;
			setUploading(false);
			setProgressCount(null);
			onBusy(false);
			if (input.current) input.current.value = "";
			if (cameraInput.current) cameraInput.current.value = "";
		}
	};
	const handleDragOver = (event: DragEvent<HTMLElement>) => {
		if (blocked) return;
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
		if (blocked) return;
		void upload(event.dataTransfer.files);
	};
	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: file drop target paired with the labelled keyboard upload button.
		<section
			className={`panel material-upload-panel${dragging ? " is-dragging" : ""}`}
			onDragOver={handleDragOver}
			onDragLeave={handleDragLeave}
			onDrop={handleDrop}
		>
			<div className="section-heading">
				<h2>
					证明材料<span className="required-mark"> *</span>
				</h2>
				<div className="row-actions">
					<span className="section-meta">{files.length} / 20</span>
					<IconButton
						label="选择已有材料"
						aria-expanded={picking}
						disabled={disabled || uploading}
						onClick={() => setPicking(!picking)}
					>
						<FolderOpen />
					</IconButton>
					<IconButton
						label="拍照上传材料"
						disabled={blocked}
						onClick={() => cameraInput.current?.click()}
					>
						<Camera />
					</IconButton>
					<IconButton
						label="上传证明材料"
						aria-invalid={fieldError ? "true" : undefined}
						aria-describedby={fieldError ? "proofs-error" : undefined}
						disabled={blocked}
						onClick={() => input.current?.click()}
					>
						<Upload />
					</IconButton>
				</div>
			</div>
			<input
				ref={input}
				type="file"
				className="visually-hidden"
				aria-label="选择证明材料"
				accept=".pdf,.jpg,.jpeg,.png"
				multiple
				tabIndex={-1}
				disabled={blocked}
				onChange={(event) => {
					void upload(event.target.files);
				}}
			/>
			<input
				ref={cameraInput}
				type="file"
				className="visually-hidden"
				aria-label="相机拍照上传材料"
				accept="image/jpeg,image/png"
				capture="environment"
				tabIndex={-1}
				disabled={blocked}
				onChange={(event) => {
					void upload(event.target.files);
				}}
			/>
			{uploading && (
				<div className="row-actions" role="status">
					<p className="form-note">材料上传中…</p>
					{progressCount && (
						<span className="section-meta">
							第 {progressCount.current} / {progressCount.total} 份
						</span>
					)}
				</div>
			)}
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			{fieldError && (
				<p className="field-error" id="proofs-error" role="alert">
					{fieldError}
				</p>
			)}
			{picking && (
				<ExistingMaterials
					selected={files}
					onAdd={onAdd}
					disabled={disabled || uploading}
				/>
			)}
			{files.length ? (
				<>
					<MaterialList
						files={files}
						onRemove={disabled || uploading ? undefined : onRemove}
					/>
					{!blocked && (
						// biome-ignore lint/a11y/noStaticElementInteractions: keyboard users activate upload via the labelled IconButton in the section header.
						// biome-ignore lint/a11y/useKeyWithClickEvents: pointer/drag convenience target paired with the accessible upload button above.
						<div
							className="upload-dropzone upload-dropzone-compact"
							onClick={() => input.current?.click()}
						>
							<Upload aria-hidden="true" />
							<span>
								继续点击或拖拽补充材料（支持 PDF、JPG、PNG，单份 ≤ {limitLabel}
								）
							</span>
						</div>
					)}
				</>
			) : !blocked ? (
				// biome-ignore lint/a11y/noStaticElementInteractions: keyboard users activate upload via the labelled IconButton in the section header.
				// biome-ignore lint/a11y/useKeyWithClickEvents: pointer/drag convenience target paired with the accessible upload button above.
				<div
					className={`upload-dropzone${fieldError ? " has-error" : ""}`}
					onClick={() => input.current?.click()}
				>
					<span className="upload-dropzone-icon" aria-hidden="true">
						<Upload />
					</span>
					<p className="upload-dropzone-title">尚未关联证明材料</p>
					<small className="upload-dropzone-hint">
						点击此处、右上角相机拍照或拖拽 PDF、JPG、PNG 文件上传（单份 ≤{" "}
						{limitLabel}，最多 20 份）
					</small>
				</div>
			) : (
				<EmptyState compact icon={<Upload />} title="尚未关联证明材料" />
			)}
		</section>
	);
}
