// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
	AchievementInput,
	Catalog,
	FieldDefinition,
} from "@/features/achievements/api";
import { changeField, DynamicFields } from "./DynamicFields";

const input: AchievementInput = {
	title: "测试",
	type_code: "research_project",
	schema_version: 1,
	attributes: {},
};
const base = { required: false, visible: true, order: 1 };
const amount: FieldDefinition = {
	...base,
	key: "project_funding",
	label: "项目经费（元）",
	type: "number",
	format: "decimal",
};
const catalog: Catalog = {
	types: [],
	terms: [],
	teachers: [],
	dictionaries: [
		{
			group_code: "evaluations",
			code: "excellent",
			name: "优秀",
			sort_order: 1,
		},
		{
			group_code: "award_grades",
			code: "first",
			name: "一等奖",
			sort_order: 1,
		},
	],
};
afterEach(cleanup);
describe("动态字段", () => {
	it("经费保持十进制字符串", () => {
		expect(
			changeField(input, amount, "123456789.01").attributes?.project_funding,
		).toBe("123456789.01");
	});
	it("更换日期或学期后需要重新确认日期差异", () => {
		const confirmed = { ...input, term_mismatch_confirmed: true };
		for (const key of ["occurred_at", "academic_term_id"]) {
			expect(
				changeField(
					confirmed,
					{
						...base,
						key,
						label: key,
						type: key === "occurred_at" ? "date" : "select",
					},
					key === "occurred_at" ? "2026-10-18" : "2",
				).term_mismatch_confirmed,
			).toBe(false);
		}
		expect(changeField(confirmed, amount, "0.01").term_mismatch_confirmed).toBe(
			true,
		);
	});
	it("评价与奖项等级分别使用对应字典", () => {
		const fields: FieldDefinition[] = [
			{
				...base,
				key: "evaluation",
				label: "评价",
				type: "select",
				dictionary: "evaluations",
			},
			{
				...base,
				key: "award_grade_code",
				label: "奖项等级",
				type: "select",
				dictionary: "award_grades",
			},
		];
		render(
			<DynamicFields
				fields={fields}
				data={input}
				catalog={catalog}
				disabled={false}
				errors={{}}
				onChange={vi.fn()}
			/>,
		);
		expect(screen.getByLabelText("评价").textContent).toContain("优秀");
		expect(screen.getByLabelText("评价").textContent).not.toContain("一等奖");
		expect(screen.getByLabelText("奖项等级").textContent).toContain("一等奖");
	});
	it("金额输入与服务端字段错误正确关联", () => {
		const change = vi.fn();
		render(
			<DynamicFields
				fields={[amount]}
				data={input}
				catalog={catalog}
				disabled={false}
				errors={{ "attributes.project_funding": "最多两位小数" }}
				onChange={change}
			/>,
		);
		const field = screen.getByLabelText("项目经费（元）");
		expect(field.getAttribute("aria-invalid")).toBe("true");
		fireEvent.change(field, { target: { value: "0.01" } });
		expect(change.mock.calls[0][0].attributes.project_funding).toBe("0.01");
		expect(screen.getByRole("alert").textContent).toBe("最多两位小数");
	});
});
