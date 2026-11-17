import type { ReactNode } from "react";
import { GraduationCap } from "reicon-react";
import type { LoginPageSettings } from "@/features/auth/loginPageApi";
import { AnnouncementBoard } from "./AnnouncementBoard";

export function LoginPageLayout({
	settings,
	loading,
	error,
	onRetry,
	children,
}: {
	settings: LoginPageSettings;
	loading: boolean;
	error: boolean;
	onRetry: () => void;
	children: ReactNode;
}) {
	return (
		<main
			className={`login-page${settings.background_url ? " has-background" : ""}`}
		>
			{settings.background_url && (
				<div
					className="login-backdrop"
					aria-hidden="true"
					style={{ backgroundImage: `url("${settings.background_url}")` }}
				/>
			)}
			<header className="public-header">
				<div className="public-container public-brand">
					{settings.logo_url ? (
						<img src={settings.logo_url} alt="网站标志" />
					) : (
						<GraduationCap aria-hidden="true" />
					)}
					<span>{settings.title}</span>
				</div>
			</header>
			<div className="public-container login-workspace">
				<section aria-label="登录入口">{children}</section>
				<AnnouncementBoard
					items={settings.announcements}
					loading={loading}
					error={error}
					onRetry={onRetry}
				/>
			</div>
			<footer className="public-footer">
				<div className="public-container">{settings.footer}</div>
			</footer>
		</main>
	);
}
