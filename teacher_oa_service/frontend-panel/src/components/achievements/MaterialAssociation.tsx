import type { FileMetadata } from "@/features/achievements/api";

export function MaterialAssociation({ file }: { file: FileMetadata }) {
	const labels = {
		linked: "已关联成果",
		historical: "历史材料保留",
		unlinked: "未关联成果",
	};
	return file.association ? <span> · {labels[file.association]}</span> : null;
}
