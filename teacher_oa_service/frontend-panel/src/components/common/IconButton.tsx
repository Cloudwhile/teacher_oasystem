import type { ButtonHTMLAttributes } from "react";

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label"> & {
	label: string;
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
