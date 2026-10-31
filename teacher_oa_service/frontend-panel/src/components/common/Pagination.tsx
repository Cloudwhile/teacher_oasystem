import { ChevronLeft, ChevronRight } from "reicon-react";
import { IconButton } from "./IconButton";
export function Pagination({
	page,
	pageSize,
	total,
	onChange,
}: {
	page: number;
	pageSize: number;
	total: number;
	onChange: (page: number) => void;
}) {
	return (
		<div className="pagination">
			<span>
				共 {total} 条 · 第 {page} / {Math.max(1, Math.ceil(total / pageSize))}{" "}
				页
			</span>
			<div className="row-actions">
				<IconButton
					label="上一页"
					disabled={page <= 1}
					onClick={() => onChange(page - 1)}
				>
					<ChevronLeft />
				</IconButton>
				<IconButton
					label="下一页"
					disabled={page * pageSize >= total}
					onClick={() => onChange(page + 1)}
				>
					<ChevronRight />
				</IconButton>
			</div>
		</div>
	);
}
