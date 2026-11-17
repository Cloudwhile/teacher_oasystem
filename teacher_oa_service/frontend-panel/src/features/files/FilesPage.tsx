import { type DragEvent, useCallback, useRef, useState } from "react";
import { Folder, Upload } from "reicon-react";
import { MaterialList } from "@/components/achievements/MaterialList";
import { ActionButton } from "@/components/common/ActionButton";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { PageHeading } from "@/components/common/PageHeading";
import { Pagination } from "@/components/common/Pagination";
import { achievementApi, errorMessage } from "@/features/achievements/api";
import { useAuth } from "@/features/auth/AuthProvider";
import { HttpError } from "@/services/httpClient";
import { useResource } from "@/services/useResource";
import { useUnsavedChanges } from "@/services/useUnsavedChanges";

export function FilesPage() {
	const { session, refresh } = useAuth();
	const [page, setPage] = useState(1);
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.files(page, signal),
		[page],
	);
	const result = useResource(load);
	const input = useRef<HTMLInputElement>(null);
	const [busy, setBusy] = useState(false);
	const [dragging, setDragging] = useState(false);
	const [error, setError] = useState("");
	const pending = useRef(false);
	const canUpload = Boolean(
		session?.user.grants.some((grant) => grant.permission === "files:write"),
	);
	useUnsavedChanges(false, busy);
	const upload = async (file?: File) => {
		if (!file || !session || pending.current) return;
		pending.current = true;
		setBusy(true);
		setError("");
		try {
			await achievementApi.upload(file, session.csrf_token);
			setPage(1);
			result.reload();
		} catch (error) {
			if (error instanceof HttpError && error.status === 401) void refresh();
			setError(errorMessage(error, "上传失败，请重新选择后重试。"));
		} finally {
			pending.current = false;
			setBusy(false);
			if (input.current) input.current.value = "";
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
		void upload(event.dataTransfer.files[0]);
	};
	return (
		<>
			<PageHeading title="文件资料">
				{canUpload && (
					<ActionButton
						variant="primary"
						disabled={busy}
						onClick={() => input.current?.click()}
					>
						<Upload aria-hidden="true" />
						上传材料
					</ActionButton>
				)}
			</PageHeading>
			<input
				type="file"
				ref={input}
				className="visually-hidden"
				tabIndex={-1}
				aria-label="选择上传材料"
				accept=".pdf,.png,.jpg,.jpeg"
				disabled={busy}
				onChange={(event) => {
					void upload(event.target.files?.[0]);
				}}
			/>
			{busy && (
				<p className="form-note" role="status">
					材料上传中…
				</p>
			)}
			{error && (
				<p className="inline-error" role="alert">
					{error}
				</p>
			)}
			{result.loading ? (
				<Feedback loading message="正在读取材料…" />
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
							<MaterialList files={result.data.items} />
						) : canUpload ? (
							// biome-ignore lint/a11y/noStaticElementInteractions: keyboard users activate upload via the labelled IconButton in PageHeading.
							// biome-ignore lint/a11y/useKeyWithClickEvents: pointer/drag convenience target paired with the accessible upload button above.
							<div
								className="upload-dropzone"
								onClick={() => !busy && input.current?.click()}
							>
								<span className="upload-dropzone-icon" aria-hidden="true">
									<Upload />
								</span>
								<p className="empty-message">暂无可查看的材料。</p>
								<small className="upload-dropzone-hint">
									点击此处或拖拽 PDF、JPG、PNG 文件上传证明材料
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
							onChange={setPage}
						/>
					</section>
				)
			)}
		</>
	);
}
