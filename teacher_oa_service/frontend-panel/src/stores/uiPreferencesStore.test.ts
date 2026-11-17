// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
	onboardingStorageKey,
	themeStorageKey,
	useUiPreferencesStore,
} from "./uiPreferencesStore";

describe("uiPreferencesStore", () => {
	beforeEach(() => {
		window.localStorage.clear();
		delete document.documentElement.dataset.theme;
		let meta = document.querySelector<HTMLMetaElement>(
			'meta[name="color-scheme"]',
		);
		if (!meta) {
			meta = document.createElement("meta");
			meta.name = "color-scheme";
			document.head.append(meta);
		}
		meta.content = "light dark";
		useUiPreferencesStore.getState().syncFromStorage();
	});

	afterEach(() => {
		window.localStorage.clear();
		delete document.documentElement.dataset.theme;
	});

	it("switches between system, dark, and light themes and syncs DOM + localStorage", () => {
		const meta = document.querySelector<HTMLMetaElement>(
			'meta[name="color-scheme"]',
		);
		expect(useUiPreferencesStore.getState().theme).toBe("system");
		expect(meta?.content).toBe("light dark");
		expect(document.documentElement.dataset.theme).toBeUndefined();

		useUiPreferencesStore.getState().setTheme("dark");
		expect(useUiPreferencesStore.getState().theme).toBe("dark");
		expect(window.localStorage.getItem(themeStorageKey)).toBe("dark");
		expect(meta?.content).toBe("dark");
		expect(document.documentElement.dataset.theme).toBe("dark");

		useUiPreferencesStore.getState().setTheme("light");
		expect(window.localStorage.getItem(themeStorageKey)).toBe("light");
		expect(meta?.content).toBe("light");
		expect(document.documentElement.dataset.theme).toBe("light");

		useUiPreferencesStore.getState().setTheme("system");
		expect(window.localStorage.getItem(themeStorageKey)).toBeNull();
		expect(meta?.content).toBe("light dark");
		expect(document.documentElement.dataset.theme).toBeUndefined();
	});

	it("persists onboarding dismissal across storage syncs", () => {
		expect(useUiPreferencesStore.getState().onboardingDismissed).toBe(false);
		useUiPreferencesStore.getState().dismissOnboarding();
		expect(useUiPreferencesStore.getState().onboardingDismissed).toBe(true);
		expect(window.localStorage.getItem(onboardingStorageKey)).toBe("1");
	});
});
