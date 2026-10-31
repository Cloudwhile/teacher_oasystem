import { useState } from "react";
import { ActionButton } from "@/components/common/ActionButton";
import { FormField } from "@/components/common/FormField";
import type { LoginAnnouncement } from "@/features/auth/loginPageApi";

type Draft = { id: string; title: string; content: string };
const empty: Draft = { id: "", title: "", content: "" };

export function AnnouncementManager({
	items,
	disabled,
	onSave,
	onDirty,
}: {
	items: LoginAnnouncement[];
	disabled: boolean;
	onSave: (items: LoginAnnouncement[]) => Promise<boolean>;
	onDirty: (dirty: boolean) => void;
}) {
	const [draft, setDraft] = useState<Draft>(empty);
	const [editing, setEditing] = useState(false);
	const [dirty, setDirty] = useState(false);
	const [error, setError] = useState("");
	const change = (next: Draft) => {
		setDraft(next);
		setDirty(true);
		onDirty(true);
	};
	const edit = (item?: LoginAnnouncement) => {
		if (dirty && !window.confirm("放弃尚未保存的公告内容吗？")) return;
		setDraft(
			item ? { id: item.id, title: item.title, content: item.content } : empty,
		);
		setEditing(true);
		setDirty(false);
		onDirty(false);
		setError("");
	};
	const close = () => {
		if (dirty && !window.confirm("放弃尚未保存的公告内容吗？")) return;
		setEditing(false);
		setDirty(false);
		onDirty(false);
	};
	const save = async (published: boolean) => {
		if (disabled) return;
		if (!draft.title.trim() || !draft.content.trim()) {
			setError("请填写公告标题和正文");
			return;
		}
		const previous = items.find((item) => item.id === draft.id);
		const next: LoginAnnouncement = {
			...draft,
			id: draft.id || crypto.randomUUID(),
			published,
			published_at: previous?.published_at ?? null,
		};
		const updated = previous
			? items.map((item) => (item.id === next.id ? next : item))
			: [next, ...items];
		if (await onSave(updated)) {
			setDraft(empty);
			setEditing(false);
			setDirty(false);
			onDirty(false);
			setError("");
		}
	};
	return (
		<section className="panel announcement-manager">
			<div className="section-heading">
				<h2>网站公告</h2>
				<ActionButton
					disabled={disabled || items.length >= 30}
					onClick={() => edit()}
				>
					新增公告
				</ActionButton>
			</div>
			{editing && (
				<div className="notice-editor">
					<FormField id="notice-title" label="公告标题" required>
						<input
							id="notice-title"
							value={draft.title}
							maxLength={120}
							disabled={disabled}
							onChange={(event) =>
								change({ ...draft, title: event.target.value })
							}
						/>
					</FormField>
					<FormField id="notice-content" label="公告正文" required>
						<textarea
							id="notice-content"
							rows={7}
							maxLength={4000}
							value={draft.content}
							disabled={disabled}
							onChange={(event) =>
								change({ ...draft, content: event.target.value })
							}
						/>
					</FormField>
					{error && (
						<p role="alert" className="inline-error">
							{error}
						</p>
					)}
					<div className="row-actions">
						<ActionButton disabled={disabled} onClick={close}>
							取消
						</ActionButton>
						<ActionButton disabled={disabled} onClick={() => void save(false)}>
							保存草稿
						</ActionButton>
						<ActionButton
							variant="primary"
							disabled={disabled}
							onClick={() => void save(true)}
						>
							发布公告
						</ActionButton>
					</div>
				</div>
			)}
			{items.length ? (
				<ul className="managed-notices">
					{items.map((item) => (
						<li key={item.id}>
							<div>
								<strong>{item.title}</strong>
								<span className="muted">
									{item.published
										? `已发布 · ${item.published_at?.slice(0, 10) ?? ""}`
										: "草稿"}
								</span>
							</div>
							<div className="row-actions">
								<ActionButton disabled={disabled} onClick={() => edit(item)}>
									编辑
								</ActionButton>
								{item.published && (
									<ActionButton
										disabled={disabled || dirty}
										onClick={() =>
											void onSave(
												items.map((row) =>
													row.id === item.id
														? { ...row, published: false }
														: row,
												),
											)
										}
									>
										撤回
									</ActionButton>
								)}
							</div>
						</li>
					))}
				</ul>
			) : (
				<p className="empty-message">暂无公告</p>
			)}
		</section>
	);
}
