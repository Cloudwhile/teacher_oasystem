import { useRef } from "react";
import { ActionButton } from "@/components/common/ActionButton";
import { errorMessage } from "@/features/achievements/api";
import { loginPageApi } from "@/features/auth/loginPageApi";

export function LoginImageField({
	label,
	value,
	csrf,
	disabled,
	onChange,
	onBusy,
	onError,
}: {
	label: string;
	value: string;
	csrf: string;
	disabled: boolean;
	onChange: (url: string) => void;
	onBusy: (busy: boolean) => void;
	onError: (error: string) => void;
}) {
	const input = useRef<HTMLInputElement>(null);
	const pending = useRef(false);
	const upload = async (file?: File) => {
		if (!file || disabled || pending.current) return;
		if (
			!["image/png", "image/jpeg"].includes(file.type) ||
			file.size > 8 * 1024 * 1024
		) {
			onError("请选择8MB以内的PNG或JPG图片");
			if (input.current) input.current.value = "";
			return;
		}
		pending.current = true;
		onBusy(true);
		onError("");
		try {
			onChange(await loginPageApi.upload(file, csrf));
		} catch (error) {
			onError(errorMessage(error, "图片上传失败，请重试"));
		} finally {
			pending.current = false;
			onBusy(false);
			if (input.current) input.current.value = "";
		}
	};
	return (
		<div className="login-image-field">
			<span>{label}</span>
			{value ? (
				<img src={value} alt={`${label}预览`} />
			) : (
				<div className="image-placeholder">未设置{label}</div>
			)}
			<input
				ref={input}
				className="visually-hidden"
				type="file"
				accept="image/png,image/jpeg"
				aria-label={`选择${label}`}
				tabIndex={-1}
				disabled={disabled}
				onChange={(event) => void upload(event.target.files?.[0])}
			/>
			<div className="row-actions">
				<ActionButton
					disabled={disabled}
					onClick={() => input.current?.click()}
				>
					上传{label}
				</ActionButton>
				{value && (
					<ActionButton disabled={disabled} onClick={() => onChange("")}>
						移除
					</ActionButton>
				)}
			</div>
		</div>
	);
}
