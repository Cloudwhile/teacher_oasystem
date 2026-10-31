import type { ButtonHTMLAttributes } from "react";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
	variant?: "primary" | "secondary";
};

export function ActionButton({
	children,
	variant = "secondary",
	type = "button",
	className = "",
	...props
}: Props) {
	return (
		<button
			type={type}
			className={`action-button action-button-${variant} ${className}`}
			{...props}
		>
			{children}
		</button>
	);
}
