import type { InputHTMLAttributes, ReactNode } from "react";
import { FormField } from "./FormField";
import { InputControl } from "./InputControl";

type Props = Omit<InputHTMLAttributes<HTMLInputElement>, "id"> & {
	id: string;
	label: string;
	icon?: ReactNode;
};

export function TextField({ id, label, icon, ...props }: Props) {
	return (
		<FormField id={id} label={label} required={props.required}>
			<InputControl leadingIcon={icon}>
				<input id={id} {...props} />
			</InputControl>
		</FormField>
	);
}
