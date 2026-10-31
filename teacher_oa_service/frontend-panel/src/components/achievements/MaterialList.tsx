import { Download, Eye, Trash } from "reicon-react";
import { IconButton } from "@/components/common/IconButton";
import type { FileMetadata } from "@/features/achievements/api";
import { MaterialAssociation } from "./MaterialAssociation";

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
							<MaterialAssociation file={file} />
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
							<Eye />
						</a>
						<a
							className="icon-button"
							href={`/api/v1/files/${file.id}/download`}
							aria-label={`下载${file.filename}`}
							title="下载"
						>
							<Download />
						</a>
						{onRemove && (
							<IconButton
								label={`移除${file.filename}`}
								onClick={() => onRemove(file.id)}
							>
								<Trash />
							</IconButton>
						)}
					</div>
				</li>
			))}
		</ul>
	);
}
