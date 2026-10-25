import { useRef, useState } from "react";
import { LuUpload } from "react-icons/lu";
import { IconButton } from "@/components/common/IconButton";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
import { HttpError } from "@/services/httpClient";
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
	const [uploading, setUploading] = useState(false);
	const pending = useRef(false);
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
	return (
		<section className="panel">
			<div className="section-heading">
				<h2>
					证明材料<span className="required-mark"> *</span>
				</h2>
				<IconButton
					label="上传证明材料"
					aria-invalid={Boolean(fieldError)}
					aria-describedby={fieldError ? "proofs-error" : undefined}
					disabled={disabled || uploading || files.length >= 20}
					onClick={() => input.current?.click()}
				>
					<LuUpload />
				</IconButton>
			</div>
			<input
				ref={input}
				type="file"
				className="visually-hidden"
				aria-label="选择证明材料"
				accept=".pdf,.jpg,.jpeg,.png"
				multiple
				tabIndex={-1}
				disabled={disabled || uploading || files.length >= 20}
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
			{files.length ? (
				<MaterialList
					files={files}
					onRemove={disabled || uploading ? undefined : onRemove}
				/>
			) : (
				<p className="empty-message">尚未关联证明材料</p>
			)}
		</section>
	);
}
