// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import {
	type AchievementDetail,
	achievementApi,
} from "@/features/achievements/api";
import { HttpError } from "@/services/httpClient";
import { ReviewActions } from "./ReviewActions";

vi.mock("@/features/auth/AuthProvider", () => ({
	useAuth: () => ({ session: { csrf_token: "csrf" }, refresh: vi.fn() }),
}));
const detail = {
	record: { id: 12, version: 2 },
	can_review: true,
	can_archive: false,
} as AchievementDetail;
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

it("空白退回意见不能提交，填写后经确认弹窗携带当前版本和意见", async () => {
	const send = vi.spyOn(achievementApi, "review").mockResolvedValue(detail);
	const complete = vi.fn();
	render(
		<ReviewActions detail={detail} onComplete={complete} onBusy={vi.fn()} />,
	);
	fireEvent.click(screen.getByRole("button", { name: "退回补正" }));
	expect(screen.getByRole("alert").textContent).toContain("请填写退回意见");
	expect(send).not.toHaveBeenCalled();
	fireEvent.change(screen.getByLabelText("办理意见"), {
		target: { value: "补充证明日期" },
	});
	fireEvent.click(screen.getByRole("button", { name: "退回补正" }));
	fireEvent.click(await screen.findByRole("button", { name: "确认退回" }));
	await waitFor(() => expect(complete).toHaveBeenCalledOnce());
	expect(send).toHaveBeenCalledWith(12, "return", 2, "补充证明日期", "csrf");
});

it("版本冲突后保留意见、锁定旧版本并提供重新加载入口", async () => {
	const send = vi
		.spyOn(achievementApi, "review")
		.mockRejectedValue(new HttpError(409, { msg: "记录已变化，请重新加载" }));
	const onReload = vi.fn();
	render(
		<ReviewActions
			detail={detail}
			onReload={onReload}
			onComplete={vi.fn()}
			onBusy={vi.fn()}
		/>,
	);
	fireEvent.change(screen.getByLabelText("办理意见"), {
		target: { value: "已核验" },
	});
	fireEvent.click(screen.getByRole("button", { name: "审核通过" }));
	fireEvent.click(await screen.findByRole("button", { name: "确认通过" }));
	expect((await screen.findByRole("alert")).textContent).toContain(
		"请重新加载",
	);
	expect((screen.getByLabelText("办理意见") as HTMLTextAreaElement).value).toBe(
		"已核验",
	);
	expect(
		(screen.getByRole("button", { name: "审核通过" }) as HTMLButtonElement)
			.disabled,
	).toBe(true);
	fireEvent.click(screen.getByRole("button", { name: "退回补正" }));
	expect(send).toHaveBeenCalledOnce();
	fireEvent.click(screen.getByRole("button", { name: "重新加载最新记录" }));
	expect(onReload).toHaveBeenCalledOnce();
});

it("归档操作只在允许归档时出现，且须二次确认才发送", async () => {
	const send = vi.spyOn(achievementApi, "review").mockResolvedValue(detail);
	const complete = vi.fn();
	const { rerender } = render(
		<ReviewActions
			detail={{ ...detail, can_review: false, can_archive: true }}
			onComplete={complete}
			onBusy={vi.fn()}
		/>,
	);
	expect(screen.queryByRole("button", { name: "审核通过" })).toBeNull();
	fireEvent.click(screen.getByRole("button", { name: "归档成果" }));
	expect(send).not.toHaveBeenCalled();
	fireEvent.click(await screen.findByRole("button", { name: "取消" }));
	expect(send).not.toHaveBeenCalled();
	fireEvent.click(screen.getByRole("button", { name: "归档成果" }));
	fireEvent.click(await screen.findByRole("button", { name: "确认归档" }));
	await waitFor(() => expect(complete).toHaveBeenCalledOnce());
	expect(send).toHaveBeenCalledWith(12, "archive", 2, "", "csrf");

	rerender(
		<ReviewActions
			detail={{ ...detail, can_review: false }}
			onComplete={vi.fn()}
			onBusy={vi.fn()}
		/>,
	);
	expect(screen.queryByRole("button")).toBeNull();
});
