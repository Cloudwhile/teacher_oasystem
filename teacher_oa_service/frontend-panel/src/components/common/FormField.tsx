import {
	cloneElement,
	createContext,
	isValidElement,
	type ReactElement,
	type ReactNode,
	useContext,
	useMemo,
} from "react";

type FormFieldAria = {
	id: string;
	invalid: boolean;
	describedBy?: string;
};

const FormFieldContext = createContext<FormFieldAria | null>(null);

export function useFormFieldAria() {
	return useContext(FormFieldContext);
}

export function FormField({
	id,
	label,
	labelAdornment,
	required = false,
	error,
	hint,
	children,
}: {
	id: string;
	label: string;
	labelAdornment?: ReactNode;
	required?: boolean;
	error?: string;
	hint?: ReactNode;
	children: ReactNode;
}) {
	const invalid = Boolean(error);
	const hintId = hint ? `${id}-hint` : undefined;
	const errorId = invalid ? `${id}-error` : undefined;
	const describedBy = [hintId, errorId].filter(Boolean).join(" ") || undefined;

	const aria = useMemo(
		() => ({ id, invalid, describedBy }),
		[id, invalid, describedBy],
	);

	let wiredChildren = children;
	if (
		isValidElement(children) &&
		typeof children.type === "string" &&
		(children.type === "input" ||
			children.type === "select" ||
			children.type === "textarea")
	) {
		const element = children as ReactElement<{
			"aria-invalid"?: boolean | "true" | "false";
			"aria-describedby"?: string;
		}>;
		const existingDescribedBy = element.props["aria-describedby"];
		const mergedDescribedBy =
			[existingDescribedBy, describedBy]
				.filter(Boolean)
				.filter((val, idx, arr) => arr.indexOf(val) === idx)
				.join(" ") || undefined;
		wiredChildren = cloneElement(element, {
			"aria-invalid":
				element.props["aria-invalid"] ?? (invalid ? "true" : "false"),
			"aria-describedby": mergedDescribedBy,
		});
	}

	const labelNode = (
		<label htmlFor={id}>
			{label}
			{required && (
				<span className="required-mark" aria-hidden="true">
					{" "}
					*
				</span>
			)}
		</label>
	);

	return (
		<FormFieldContext.Provider value={aria}>
			<div className={`form-field${invalid ? " invalid" : ""}`}>
				{labelAdornment ? (
					<div className="participant-field-head">
						{labelNode}
						{labelAdornment}
					</div>
				) : (
					labelNode
				)}
				{wiredChildren}
				{hint && (
					<p id={hintId} className="field-hint">
						{hint}
					</p>
				)}
				{error && (
					<p id={errorId} className="field-error" role="alert">
						{error}
					</p>
				)}
			</div>
		</FormFieldContext.Provider>
	);
}
