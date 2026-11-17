import { useState } from "react";
import { Bullhorn } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import { useConfirm } from "@/components/common/ConfirmDialog";
import { EmptyState } from "@/components/common/EmptyState";
import { FormField } from "@/components/common/FormField";
import type { LoginAnnouncement } from "@/features/auth/loginPageApi";
import { formatAppDate } from "@/services/dateFormat";

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
	const confirm = useConfirm();
	const [draft, setDraft] = useState<Draft>(empty);
	const [editing, setEditing] = useState(false);
	const [dirty, setDirty] = useState(false);
	const [error, setError] = useState("");
	const change = (next: Draft) => {
		setDraft(next);
		setDirty(true);
		onDirty(true);
	};
	const edit = async (item?: LoginAnnouncement) => {
		if (dirty) {
			const discard = await confirm({
				title: "放弃未保存的公告内容？",
				description: "切换编辑对象将丢弃当前正在编写的公告草稿。",
				confirmLabel: "放弃修改",
				cancelLabel: "继续编辑",
				tone: "danger",
			});
			if (!discard) return;
		}
		setDraft(
			item ? { id: item.id, title: item.title, content: item.content } : empty,
		);
		setEditing(true);
		setDirty(false);
		onDirty(false);
		setError("");
	};
	const close = async () => {
		if (dirty) {
			const discard = await confirm({
				title: "放弃未保存的公告内容？",
				description: "关闭编辑器将丢弃当前尚未保存的公告内容。",
				confirmLabel: "放弃并关闭",
				cancelLabel: "继续编辑",
				tone: "danger",
			});
			if (!discard) return;
		}
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
				{!editing && (
					<ActionButton
						disabled={disabled || items.length >= 30}
						onClick={() => void edit()}
					>
						新增公告
					</ActionButton>
				)}
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
						<ActionButton
							variant="tertiary"
							disabled={disabled}
							onClick={() => void close()}
						>
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
										? `已发布 · ${formatAppDate(item.published_at)}`
										: "草稿"}
								</span>
							</div>
							<div className="row-actions">
								<ActionButton
									size="sm"
									disabled={disabled}
									onClick={() => void edit(item)}
								>
									编辑
								</ActionButton>
								{item.published && (
									<ActionButton
										size="sm"
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
								<ActionButton
									variant="danger-tertiary"
									size="sm"
									disabled={disabled}
									onClick={() => {
										void confirm({
											title: `确定删除公告「${item.title}」吗？`,
											description:
												"删除后该公告将从登录页公告栏移除，且无法直接恢复。",
											confirmLabel: "确认删除",
											cancelLabel: "保留公告",
											tone: "danger",
										}).then((accepted) => {
											if (!accepted) return;
											void onSave(
												items.filter((row) => row.id !== item.id),
											).then((ok) => {
												if (ok && draft.id === item.id) {
													setDraft(empty);
													setEditing(false);
													setDirty(false);
													onDirty(false);
													setError("");
												}
											});
										});
									}}
								>
									删除
								</ActionButton>
							</div>
						</li>
					))}
				</ul>
			) : (
				!editing && (
					<EmptyState
						icon={<Bullhorn />}
						title="暂无公告"
						description="发布学校通知或填报说明后，将显示在登录页公告栏。"
						action={
							<ActionButton disabled={disabled} onClick={() => void edit()}>
								新增公告
							</ActionButton>
						}
					/>
				)
			)}
		</section>
	);
}
