import { defineConfig, devices } from "playwright/test";

export default defineConfig({
	testDir: "./e2e",
	testMatch: "**/*.spec.ts",
	timeout: 30_000,
	use: {
		baseURL: process.env.TEACHEROA_BASE_URL ?? "http://127.0.0.1:3000",
		trace: "retain-on-failure",
	},
	projects: [
		{
			name: "chromium",
			use: {
				...devices["Desktop Chrome"],
				channel: process.env.CI ? undefined : "chrome",
			},
		},
	],
});
