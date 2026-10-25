import { useCallback, useRef, useState } from "react";
import { LuUpload } from "react-icons/lu";
import { MaterialList } from "@/components/achievements/MaterialList";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
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
	const [error, setError] = useState("");
	const pending = useRef(false);
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
	return (
		<>
			<PageHeading title="文件资料" eyebrow="证明材料">
				{session?.user.grants.some(
					(grant) => grant.permission === "files:write",
				) && (
					<IconButton
						label="上传材料"
						disabled={busy}
						onClick={() => input.current?.click()}
					>
						<LuUpload />
					</IconButton>
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
					<section className="panel">
						<div className="section-heading">
							<h2>材料列表</h2>
							<span className="section-meta">共 {result.data.total} 份</span>
						</div>
						{result.data.items.length ? (
							<MaterialList files={result.data.items} />
						) : (
							<p className="empty-message">暂无可查看的材料。</p>
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
