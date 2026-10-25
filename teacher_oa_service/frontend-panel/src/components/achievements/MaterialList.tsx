import { LuDownload, LuEye, LuTrash2 } from "react-icons/lu";
import { IconButton } from "@/components/common/IconButton";
import type { FileMetadata } from "@/features/achievements/api";

export function MaterialList({
	files,
	onRemove,
}: {
	files: FileMetadata[];
	onRemove?: (id: number) => void;
}) {
	return (
		<ul className="material-list">
			{files.map((file) => (
				<li key={file.id}>
					<div className="material-name">
						<strong>{file.filename}</strong>
						<small>
							{file.mime_type === "application/pdf" ? "PDF" : "图片"} ·{" "}
							{(file.size_bytes / 1024).toFixed(1)} KB
						</small>
					</div>
					<div className="row-actions">
						<a
							className="icon-button"
							href={`/api/v1/files/${file.id}/preview`}
							target="_blank"
							rel="noreferrer"
							aria-label={`预览${file.filename}`}
							title="预览"
						>
							<LuEye />
						</a>
						<a
							className="icon-button"
							href={`/api/v1/files/${file.id}/download`}
							aria-label={`下载${file.filename}`}
							title="下载"
						>
							<LuDownload />
						</a>
						{onRemove && (
							<IconButton
								label={`移除${file.filename}`}
								onClick={() => onRemove(file.id)}
							>
								<LuTrash2 />
							</IconButton>
						)}
					</div>
				</li>
			))}
		</ul>
	);
}
