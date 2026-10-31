import { useCallback, useState } from "react";
import { Plus } from "reicon-react";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { Pagination } from "@/components/common/Pagination";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
import { useResource } from "@/services/useResource";
import { MaterialAssociation } from "./MaterialAssociation";
export function ExistingMaterials({
	selected,
	onAdd,
	disabled,
}: {
	selected: FileMetadata[];
	onAdd: (file: FileMetadata) => void;
	disabled: boolean;
}) {
	const [page, setPage] = useState(1);
	const load = useCallback(
		(signal: AbortSignal) => achievementApi.ownFiles(page, signal),
		[page],
	);
	const result = useResource(load);
	return (
		<div className="existing-materials">
			<h3>我的材料</h3>
			{result.loading ? (
				<Feedback loading message="正在读取材料…" />
			) : result.error || !result.data ? (
				<Feedback
					message={errorMessage(result.error)}
					onRetry={result.reload}
				/>
			) : (
				<>
					<ul className="material-list">
						{result.data.items.map((file) => (
							<li key={file.id}>
								<div className="material-name">
									<strong>{file.filename}</strong>
									<small>
										{selected.some((f) => f.id === file.id)
											? "已选入本次成果"
											: new Date(file.created_at).toLocaleDateString("zh-CN")}
										<MaterialAssociation file={file} />
									</small>
								</div>
								<IconButton
									label={`关联${file.filename}`}
									disabled={
										disabled ||
										selected.length >= 20 ||
										selected.some((f) => f.id === file.id)
									}
									onClick={() => onAdd(file)}
								>
									<Plus />
								</IconButton>
							</li>
						))}
					</ul>
					{!result.data.items.length && (
						<p className="empty-message">暂无已上传材料</p>
					)}
					<Pagination
						page={result.data.page}
						pageSize={result.data.page_size}
						total={result.data.total}
						onChange={setPage}
					/>
				</>
			)}
		</div>
	);
}
