import { type DragEvent, useRef, useState } from "react";
import { FolderOpen, Upload } from "reicon-react";
import { IconButton } from "@/components/common/IconButton";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
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
	const input = useRef<HTMLInputElement>(null);
	const [error, setError] = useState("");
	const [picking, setPicking] = useState(false);
	const [uploading, setUploading] = useState(false);
	const [dragging, setDragging] = useState(false);
	const pending = useRef(false);
	const blocked = disabled || uploading || files.length >= 20;
	const upload = async (selection: FileList | null) => {
		if (!selection?.length || disabled || pending.current) return;
		if (files.length + selection.length > 20) {
			setError("每个成果最多关联20份材料。");
			if (input.current) input.current.value = "";
			return;
		}
		setError("");
		setUploading(true);
		pending.current = true;
		onBusy(true);
		const failures: string[] = [];
		try {
			for (const file of Array.from(selection)) {
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
			onBusy(false);
			if (input.current) input.current.value = "";
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
						label="上传证明材料"
						aria-invalid={Boolean(fieldError)}
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
			{uploading && (
				<p className="form-note" role="status">
					材料上传中…
				</p>
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
							<span>继续点击或拖拽补充材料（支持 PDF、JPG、PNG）</span>
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
					<p className="empty-message">尚未关联证明材料</p>
					<small className="upload-dropzone-hint">
						点击此处或拖拽 PDF、JPG、PNG 文件上传（最多 20 份）
					</small>
				</div>
			) : (
				<p className="empty-message">尚未关联证明材料</p>
			)}
		</section>
	);
}
