// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FormField } from "./FormField";
import { Pagination } from "./Pagination";
import { PasswordField } from "./PasswordField";
import { TextField } from "./TextField";

afterEach(() => {
	cleanup();
});

describe("FormField ARIA linkage", () => {
	it("sets aria-invalid='false' by default and links error id only when invalid", () => {
		const { rerender } = render(
			<FormField id="email" label="邮箱" hint="学校工作邮箱">
				<input id="email" />
			</FormField>,
		);
		const input = screen.getByLabelText("邮箱");
		expect(input.getAttribute("aria-invalid")).toBe("false");
		expect(input.getAttribute("aria-describedby")).toBe("email-hint");

		rerender(
			<FormField
				id="email"
				label="邮箱"
				hint="学校工作邮箱"
				error="请输入有效邮箱"
			>
				<input id="email" />
			</FormField>,
		);
		expect(input.getAttribute("aria-invalid")).toBe("true");
		expect(input.getAttribute("aria-describedby")).toBe(
			"email-hint email-error",
		);
	});

	it("wires TextField and PasswordField error states automatically", () => {
		render(
			<>
				<TextField id="user" label="账号" error="请输入账号" />
				<PasswordField
					id="pwd"
					label="密码"
					value=""
					onChange={vi.fn()}
					error="请输入密码"
				/>
			</>,
		);
		const userInput = screen.getByRole("textbox", { name: /^账号/ });
		expect(userInput.getAttribute("aria-invalid")).toBe("true");
		expect(userInput.getAttribute("aria-describedby")).toBe("user-error");

		const pwdInput = screen.getByLabelText(/^密码/);
		expect(pwdInput.getAttribute("aria-invalid")).toBe("true");
		expect(pwdInput.getAttribute("aria-describedby")).toBe("pwd-error");
	});
});

describe("Pagination", () => {
	it("renders page numbers, page-size selector, and direct page jump", () => {
		const onChange = vi.fn();
		const onSize = vi.fn();
		render(
			<Pagination
				page={3}
				pageSize={20}
				total={160}
				onChange={onChange}
				onPageSizeChange={onSize}
			/>,
		);
		expect(
			screen
				.getByRole("button", { name: "第 3 页" })
				.getAttribute("aria-current"),
		).toBe("page");
		fireEvent.click(screen.getByRole("button", { name: "第 4 页" }));
		expect(onChange).toHaveBeenCalledWith(4);

		fireEvent.change(screen.getByLabelText("每页显示条数"), {
			target: { value: "50" },
		});
		expect(onSize).toHaveBeenCalledWith(50);

		const jump = screen.getByLabelText("跳转页码");
		fireEvent.change(jump, { target: { value: "7" } });
		fireEvent.submit(jump.closest("form") as HTMLFormElement);
		expect(onChange).toHaveBeenCalledWith(7);
	});
});
