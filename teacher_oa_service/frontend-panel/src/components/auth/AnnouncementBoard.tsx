import { Dialog } from "@base-ui/react/dialog";
import { Bullhorn, ChevronRight } from "reicon-react";
import type { LoginAnnouncement } from "@/features/auth/loginPageApi";
import { ActionButton } from "../common/ActionButton";

export function AnnouncementBoard({
	items,
	loading,
	error,
	onRetry,
}: {
	items: LoginAnnouncement[];
	loading: boolean;
	error: boolean;
	onRetry: () => void;
}) {
	return (
		<section
			className="announcement-board public-container"
			aria-labelledby="announcement-heading"
		>
			<h2 id="announcement-heading">
				<Bullhorn aria-hidden="true" />
				网站公告
			</h2>
			{loading ? (
				<p className="notice-status" role="status">
					正在读取公告…
				</p>
			) : error ? (
				<div className="notice-status">
					<p role="alert">公告暂时无法加载</p>
					<ActionButton onClick={onRetry}>重新加载</ActionButton>
				</div>
			) : items.length ? (
				<ul className="announcement-list">
					{items.map((item) => (
						<li key={item.id}>
							<Dialog.Root>
								<Dialog.Trigger className="announcement-link">
									<time dateTime={item.published_at ?? undefined}>
										{item.published_at?.slice(0, 10)}
									</time>
									<span>{item.title}</span>
									<ChevronRight aria-hidden="true" />
								</Dialog.Trigger>
								<Dialog.Portal>
									<Dialog.Backdrop className="notice-backdrop" />
									<Dialog.Popup className="notice-dialog">
										<div className="section-heading">
											<Dialog.Title>{item.title}</Dialog.Title>
											<Dialog.Close className="action-button action-button-secondary">
												关闭
											</Dialog.Close>
										</div>
										<time
											className="muted"
											dateTime={item.published_at ?? undefined}
										>
											{item.published_at?.slice(0, 10)}
										</time>
										<Dialog.Description className="notice-body">
											{item.content}
										</Dialog.Description>
									</Dialog.Popup>
								</Dialog.Portal>
							</Dialog.Root>
						</li>
					))}
				</ul>
			) : (
				<p className="notice-status">暂无公告</p>
			)}
		</section>
	);
}
