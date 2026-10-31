import { useState } from "react";
import { Eye, EyeOff, LockKeyhole } from "reicon-react";
import { FormField } from "./FormField";
import { IconButton } from "./IconButton";
import { InputControl } from "./InputControl";
export function PasswordField({
	id,
	label,
	value,
	onChange,
	disabled = false,
	autoComplete = "new-password",
	required = true,
	minLength,
}: {
	id: string;
	label: string;
	value: string;
	onChange: (value: string) => void;
	disabled?: boolean;
	autoComplete?: string;
	required?: boolean;
	minLength?: number;
}) {
	const [visible, setVisible] = useState(false);
	return (
		<FormField id={id} label={label} required={required}>
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
				/>
			</InputControl>
		</FormField>
	);
}
