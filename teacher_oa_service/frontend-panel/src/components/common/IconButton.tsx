import type { ButtonHTMLAttributes, Ref } from "react";

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label"> & {
	label: string;
	ref?: Ref<HTMLButtonElement>;
};
export function IconButton({
	label,
	children,
	className = "",
	...props
}: Props) {
	return (
		<button
			type="button"
			aria-label={label}
			title={label}
			className={`icon-button ${className}`}
			{...props}
		>
			{children}
		</button>
	);
}
