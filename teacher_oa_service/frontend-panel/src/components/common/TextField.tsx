import type { InputHTMLAttributes, ReactNode } from "react";
import { FormField, useFormFieldAria } from "./FormField";
import { InputControl } from "./InputControl";

type Props = Omit<InputHTMLAttributes<HTMLInputElement>, "id"> & {
	id: string;
	label: string;
	icon?: ReactNode;
	error?: string;
	hint?: ReactNode;
};

function LinkedInput({
	id,
	...props
}: InputHTMLAttributes<HTMLInputElement> & { id: string }) {
	const aria = useFormFieldAria();
	const describedBy =
		[props["aria-describedby"], aria?.describedBy]
			.filter(Boolean)
			.filter((val, idx, arr) => arr.indexOf(val) === idx)
			.join(" ") || undefined;
	return (
		<input
			id={id}
			{...props}
			aria-invalid={props["aria-invalid"] ?? (aria?.invalid ? "true" : "false")}
			aria-describedby={describedBy}
		/>
	);
}

export function TextField({ id, label, icon, error, hint, ...props }: Props) {
	return (
		<FormField
			id={id}
			label={label}
			required={props.required}
			error={error}
			hint={hint}
		>
			<InputControl leadingIcon={icon}>
				<LinkedInput id={id} {...props} />
			</InputControl>
		</FormField>
	);
}
