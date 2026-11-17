import type { ButtonHTMLAttributes } from "react";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
	variant?: "primary" | "secondary" | "tertiary" | "danger-tertiary" | "danger";
	size?: "sm" | "md" | "lg";
};

export function ActionButton({
	children,
	variant = "secondary",
	size = "md",
	type = "button",
	className = "",
	...props
}: Props) {
	const sizeClass = size === "md" ? "" : ` action-button-${size}`;
	return (
		<button
			type={type}
			className={`action-button action-button-${variant}${sizeClass}${className ? ` ${className}` : ""}`}
			{...props}
		>
			{children}
		</button>
	);
}
