import { expect, type Page } from "playwright/test";

// Only for isolated legacy business fixtures; never exposed in the login UI.
export async function fixtureLogin(page: Page, identity = "teacher") {
	const origin = new URL(page.url()).origin;
	const response = await page.request.post("/api/v1/auth/mock/login", {
		headers: { Origin: origin },
		data: { identity },
	});
	expect(response.ok()).toBe(true);
	await page.goto("/dashboard");
	await expect(
		page.getByRole("heading", { name: "工作台", exact: true }),
	).toBeVisible();
}
