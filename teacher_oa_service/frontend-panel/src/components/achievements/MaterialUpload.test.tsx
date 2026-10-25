// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { achievementApi } from "@/features/achievements/api";
import { HttpError } from "@/services/httpClient";
import { MaterialUpload } from "./MaterialUpload";

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});
it("上传失败后恢复可操作状态并显示错误", async () => {
	vi.spyOn(achievementApi, "upload").mockRejectedValue(
		new HttpError(503, { code: "unavailable", msg: "材料存储暂时不可用" }),
	);
	const onBusy = vi.fn();
	const onAdd = vi.fn();
	render(
		<MaterialUpload
			files={[]}
			csrf="test"
			disabled={false}
			onBusy={onBusy}
			onAdd={onAdd}
			onRemove={vi.fn()}
		/>,
	);
	fireEvent.change(screen.getByLabelText("选择证明材料"), {
		target: {
			files: [new File(["proof"], "proof.png", { type: "image/png" })],
		},
	});
	expect((await screen.findByRole("alert")).textContent).toContain(
		"材料存储暂时不可用",
	);
	expect(onBusy.mock.calls.map((call) => call[0])).toEqual([true, false]);
	expect(onAdd).not.toHaveBeenCalled();
	expect(
		(screen.getByRole("button", { name: "上传证明材料" }) as HTMLButtonElement)
			.disabled,
	).toBe(false);
});

it("一份材料失败不会丢弃后续上传，且标明失败文件名", async () => {
	const upload = vi
		.spyOn(achievementApi, "upload")
		.mockRejectedValueOnce(new HttpError(400, { msg: "图片内容无效" }))
		.mockResolvedValueOnce({
			id: 2,
			filename: "good.png",
			size_bytes: 4,
			mime_type: "image/png",
			sha256: "test",
			uploaded_by: 91001,
			upload_status: "ready",
			created_at: "2026-10-25T00:00:00Z",
		});
	const onAdd = vi.fn();
	render(
		<MaterialUpload
			files={[]}
			csrf="test"
			disabled={false}
			onAdd={onAdd}
			onRemove={vi.fn()}
			onBusy={vi.fn()}
		/>,
	);
	fireEvent.change(screen.getByLabelText("选择证明材料"), {
		target: {
			files: [new File(["bad"], "bad.png"), new File(["good"], "good.png")],
		},
	});
	expect((await screen.findByRole("alert")).textContent).toContain(
		"bad.png：图片内容无效",
	);
	expect(upload).toHaveBeenCalledTimes(2);
	expect(onAdd).toHaveBeenCalledWith(expect.objectContaining({ id: 2 }));
});

it("会话失效时停止剩余上传并触发重新认证", async () => {
	const upload = vi
		.spyOn(achievementApi, "upload")
		.mockRejectedValue(new HttpError(401, { msg: "请先登录" }));
	const expired = vi.fn();
	render(
		<MaterialUpload
			files={[]}
			csrf="test"
			disabled={false}
			onAdd={vi.fn()}
			onRemove={vi.fn()}
			onBusy={vi.fn()}
			onSessionExpired={expired}
		/>,
	);
	fireEvent.change(screen.getByLabelText("选择证明材料"), {
		target: { files: [new File(["a"], "one.png"), new File(["b"], "two.png")] },
	});
	await screen.findByRole("alert");
	expect(expired).toHaveBeenCalledOnce();
	expect(upload).toHaveBeenCalledOnce();
});
