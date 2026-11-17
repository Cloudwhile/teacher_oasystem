import { useCallback, useState } from "react";
import { FolderOpen, Plus } from "reicon-react";
import { EmptyState } from "@/components/common/EmptyState";
import { Feedback } from "@/components/common/Feedback";
import { IconButton } from "@/components/common/IconButton";
import { Pagination } from "@/components/common/Pagination";
import { TableSkeleton } from "@/components/common/Skeleton";
import {
	achievementApi,
	errorMessage,
	type FileMetadata,
} from "@/features/achievements/api";
import { formatAppDate } from "@/services/dateFormat";
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
			{result.loading && !result.data ? (
				<TableSkeleton rows={3} message="正在读取材料…" />
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
											: formatAppDate(file.created_at)}
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
						<EmptyState
							compact
							icon={<FolderOpen />}
							title="暂无已上传材料"
							description="上传过证明材料后，可在此快速复用。"
						/>
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
