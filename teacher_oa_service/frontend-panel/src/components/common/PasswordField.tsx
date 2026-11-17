import { type ReactNode, useState } from "react";
import { Eye, EyeOff, LockKeyhole } from "reicon-react";
import { FormField, useFormFieldAria } from "./FormField";
import { IconButton } from "./IconButton";
import { InputControl } from "./InputControl";

function LinkedPasswordInput({
	id,
	visible,
	autoComplete,
	value,
	onChange,
	disabled,
	required,
	minLength,
}: {
	id: string;
	visible: boolean;
	autoComplete: string;
	value: string;
	onChange: (value: string) => void;
	disabled: boolean;
	required: boolean;
	minLength?: number;
}) {
	const aria = useFormFieldAria();
	return (
		<input
			id={id}
			type={visible ? "text" : "password"}
			autoComplete={autoComplete}
			value={value}
			onChange={(e) => onChange(e.target.value)}
			disabled={disabled}
			required={required}
			minLength={minLength}
			maxLength={128}
			aria-invalid={aria?.invalid ? "true" : "false"}
			aria-describedby={aria?.describedBy}
		/>
	);
}

export function PasswordField({
	id,
	label,
	value,
	onChange,
	disabled = false,
	autoComplete = "new-password",
	required = true,
	minLength,
	error,
	hint,
}: {
	id: string;
	label: string;
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
	autoComplete?: string;
	required?: boolean;
	minLength?: number;
	error?: string;
	hint?: ReactNode;
}) {
	const [visible, setVisible] = useState(false);
	return (
		<FormField
			id={id}
			label={label}
			required={required}
			error={error}
			hint={hint}
		>
			<InputControl
				leadingIcon={<LockKeyhole />}
				trailing={
					<IconButton
						className="password-toggle"
						label={visible ? `隐藏${label}` : `显示${label}`}
						aria-controls={id}
						aria-pressed={visible}
						disabled={disabled}
						onClick={() => setVisible(!visible)}
					>
						{visible ? (
							<EyeOff aria-hidden="true" />
						) : (
							<Eye aria-hidden="true" />
						)}
					</IconButton>
				}
			>
				<LinkedPasswordInput
					id={id}
					visible={visible}
					autoComplete={autoComplete}
					value={value}
					onChange={onChange}
					disabled={disabled}
					required={required}
					minLength={minLength}
				/>
			</InputControl>
		</FormField>
	);
}
