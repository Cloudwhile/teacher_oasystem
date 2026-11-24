import { type ReactNode, useEffect } from "react";
import { Toaster } from "sonner";
import { ConfirmProvider } from "@/components/common/ConfirmDialog";
import { AuthProvider } from "@/features/auth/AuthProvider";
import { useMediaQuery } from "@/services/useMediaQuery";
import {
	applyThemeToDocument,
	themeStorageKey,
	useUiPreferencesStore,
} from "@/stores/uiPreferencesStore";

export function AppProviders({ children }: { children: ReactNode }) {
	const theme = useUiPreferencesStore((state) => state.theme);
	const prefersDark = useMediaQuery("(prefers-color-scheme: dark)");
	const resolvedTheme =
		theme === "system" ? (prefersDark ? "dark" : "light") : theme;
	const syncFromStorage = useUiPreferencesStore(
		(state) => state.syncFromStorage,
	);

	useEffect(() => {
		applyThemeToDocument(theme);
	}, [theme]);

	useEffect(() => {
		const handleStorage = (event: StorageEvent) => {
			if (event.key === null || event.key === themeStorageKey) {
				syncFromStorage();
			}
		};
		window.addEventListener("storage", handleStorage);
		return () => window.removeEventListener("storage", handleStorage);
	}, [syncFromStorage]);

	return (
		<AuthProvider>
			<ConfirmProvider>
				{children}
				<Toaster
					position="bottom-right"
					closeButton
					theme={resolvedTheme}
					containerAriaLabel="系统通知"
					style={{
						zIndex: "var(--z-toast)",
						fontFamily: "var(--font-sans)",
					}}
					toastOptions={{
						className: "oa-toast",
						style: {
							border: "var(--border-thin) solid var(--color-border)",
							borderRadius: "var(--radius-DEFAULT)",
							background: "var(--color-surface-overlay)",
							color: "var(--color-text)",
							boxShadow: "var(--shadow-lg)",
						},
					}}
				/>
			</ConfirmProvider>
		</AuthProvider>
	);
}
