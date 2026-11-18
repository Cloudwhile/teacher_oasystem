import { type FormEvent, useState } from "react";
import { ChevronLeft, ChevronRight } from "reicon-react";
import { IconButton } from "./IconButton";

function buildPageRange(current: number, totalPages: number): (number | "…")[] {
	if (totalPages <= 7) {
		return Array.from({ length: totalPages }, (_, index) => index + 1);
	}
	const items: (number | "…")[] = [1];
	const left = Math.max(2, current - 1);
	const right = Math.min(totalPages - 1, current + 1);
	if (left > 2) items.push("…");
	for (let p = left; p <= right; p += 1) {
		items.push(p);
	}
	if (right < totalPages - 1) items.push("…");
	items.push(totalPages);
	return items;
}

export function Pagination({
	page,
	pageSize,
	total,
	pageSizeOptions = [10, 20, 50],
	onChange,
	onPageSizeChange,
}: {
	page: number;
	pageSize: number;
	total: number;
	pageSizeOptions?: number[];
	onChange: (page: number) => void;
	onPageSizeChange?: (pageSize: number) => void;
}) {
	const [jumpInput, setJumpInput] = useState("");
	if (total <= 0) return null;
	const totalPages = Math.max(1, Math.ceil(total / pageSize));
	const minPageSize = pageSizeOptions[0] ?? 10;
	const defaultPageSize = pageSizeOptions[1] ?? 20;
	const showSizeSelect =
		Boolean(onPageSizeChange) &&
		(total > minPageSize || pageSize !== defaultPageSize);
	// §8.4 Omit single-page pagination bar when neither page navigation nor page-size switching applies (P2-2).
	if (totalPages <= 1 && !showSizeSelect) return null;
	const pages = buildPageRange(page, totalPages);

	const submitJump = (event: FormEvent) => {
		event.preventDefault();
		const target = Number.parseInt(jumpInput, 10);
		if (Number.isFinite(target) && target >= 1 && target <= totalPages) {
			onChange(target);
			setJumpInput("");
		}
	};

	return (
		<nav className="pagination" aria-label="分页导航">
			<div className="pagination-summary">
				<span>
					共 {total} 条 · 第 {page} / {totalPages} 页
				</span>
				{showSizeSelect && onPageSizeChange && (
					<select
						className="pagination-size"
						aria-label="每页显示条数"
						value={pageSize}
						onChange={(event) => onPageSizeChange(Number(event.target.value))}
					>
						{pageSizeOptions.map((size) => (
							<option key={size} value={size}>
								{size} 条/页
							</option>
						))}
					</select>
				)}
			</div>
			{totalPages > 1 && (
				<div className="pagination-controls">
					<IconButton
						label="上一页"
						disabled={page <= 1}
						onClick={() => onChange(page - 1)}
					>
						<ChevronLeft />
					</IconButton>
					<div className="pagination-pages">
						{pages.map((entry, idx) =>
							entry === "…" ? (
								<span
									key={`ellipsis-${idx < 3 ? "start" : "end"}`}
									className="pagination-ellipsis"
									aria-hidden="true"
								>
									…
								</span>
							) : (
								<button
									key={entry}
									type="button"
									className="pagination-page"
									aria-label={`第 ${entry} 页`}
									aria-current={entry === page ? "page" : undefined}
									onClick={() => {
										if (entry !== page) onChange(entry);
									}}
								>
									{entry}
								</button>
							),
						)}
					</div>
					<IconButton
						label="下一页"
						disabled={page * pageSize >= total}
						onClick={() => onChange(page + 1)}
					>
						<ChevronRight />
					</IconButton>
					{totalPages > 5 && (
						<form className="pagination-jump" onSubmit={submitJump}>
							<input
								type="text"
								inputMode="numeric"
								pattern="[0-9]*"
								aria-label="跳转页码"
								placeholder="页码"
								value={jumpInput}
								onChange={(event) => setJumpInput(event.target.value)}
							/>
						</form>
					)}
				</div>
			)}
		</nav>
	);
}
