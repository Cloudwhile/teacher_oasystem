import { create } from "zustand";

export type ThemePreference = "system" | "light" | "dark";

export const themeStorageKey = "teacher-oa:color-scheme";
export const onboardingStorageKey = "teacher-oa:onboarding-dismissed";

export function normalizeTheme(
	value: string | null | undefined,
): ThemePreference {
	if (value === "light" || value === "dark") return value;
	return "system";
}

export function applyThemeToDocument(theme: ThemePreference) {
	if (typeof document === "undefined") return;
	const meta = document.querySelector<HTMLMetaElement>(
		'meta[name="color-scheme"]',
	);
	if (meta) {
		meta.content = theme === "system" ? "light dark" : theme;
	}
	if (theme === "system") {
		delete document.documentElement.dataset.theme;
	} else {
		document.documentElement.dataset.theme = theme;
	}
}

function readInitialTheme(): ThemePreference {
	if (typeof window === "undefined") return "system";
	try {
		return normalizeTheme(window.localStorage.getItem(themeStorageKey));
	} catch {
		return "system";
	}
}

function readOnboardingDismissed(): boolean {
	if (typeof window === "undefined") return false;
	try {
		return window.localStorage.getItem(onboardingStorageKey) === "1";
	} catch {
		return false;
	}
}

type UiPreferencesState = {
	theme: ThemePreference;
	onboardingDismissed: boolean;
	setTheme: (theme: ThemePreference) => void;
	dismissOnboarding: () => void;
	syncFromStorage: () => void;
};

export const useUiPreferencesStore = create<UiPreferencesState>((set) => ({
	theme: readInitialTheme(),
	onboardingDismissed: readOnboardingDismissed(),
	setTheme(theme) {
		if (typeof window !== "undefined") {
			try {
				if (theme === "system") {
					window.localStorage.removeItem(themeStorageKey);
				} else {
					window.localStorage.setItem(themeStorageKey, theme);
				}
			} catch {
				// Ignore storage quota / privacy errors and still apply to DOM.
			}
		}
		applyThemeToDocument(theme);
		set({ theme });
	},
	dismissOnboarding() {
		if (typeof window !== "undefined") {
			try {
				window.localStorage.setItem(onboardingStorageKey, "1");
			} catch {
				// Ignore storage errors.
			}
		}
		set({ onboardingDismissed: true });
	},
	syncFromStorage() {
		const theme = readInitialTheme();
		const onboardingDismissed = readOnboardingDismissed();
		applyThemeToDocument(theme);
		set({ theme, onboardingDismissed });
	},
}));
