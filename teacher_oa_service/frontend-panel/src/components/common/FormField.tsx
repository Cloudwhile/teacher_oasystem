import type { ReactNode } from "react";
export function FormField({
	id,
	label,
	required = false,
	error,
	children,
}: {
	id: string;
	label: string;
	required?: boolean;
	error?: string;
	children: ReactNode;
}) {
	return (
		<div className={`form-field ${error ? "invalid" : ""}`}>
			<label htmlFor={id}>
				{label}
				{required && (
					<span className="required-mark" aria-hidden="true">
						{" "}
						*
					</span>
				)}
			</label>
			{children}
			{error && (
				<p id={`${id}-error`} className="field-error" role="alert">
					{error}
				</p>
			)}
		</div>
	);
}
