import type { ReactNode } from "react";

export function InputControl({
	children,
	leadingIcon,
	trailing,
}: {
	children: ReactNode;
	leadingIcon?: ReactNode;
	trailing?: ReactNode;
}) {
	return (
		<div className="input-control">
			{leadingIcon && (
				<span className="input-leading" aria-hidden="true">
					{leadingIcon}
				</span>
			)}
			{children}
			{trailing && <span className="input-trailing">{trailing}</span>}
		</div>
	);
}
