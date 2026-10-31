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
		<main className="login-page">
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
			<section
				className="login-banner"
				aria-label="登录入口"
				style={
					settings.background_url
						? { backgroundImage: `url("${settings.background_url}")` }
						: undefined
				}
			>
				<div className="public-container login-banner-inner">{children}</div>
			</section>
			<AnnouncementBoard
				items={settings.announcements}
				loading={loading}
				error={error}
				onRetry={onRetry}
			/>
			<footer className="public-footer">
				<div className="public-container">{settings.footer}</div>
			</footer>
		</main>
	);
}
