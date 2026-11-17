import { Dialog } from "@base-ui/react/dialog";
import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useMemo,
	useRef,
	useState,
} from "react";
import { ActionButton } from "./ActionButton";

export type ConfirmOptions = {
	title: string;
	description?: ReactNode;
	confirmLabel?: string;
	cancelLabel?: string;
	tone?: "primary" | "danger";
};

export function ConfirmDialog({
	open,
	title,
	description,
	confirmLabel = "确认",
	cancelLabel = "取消",
	tone = "primary",
	busy = false,
	children,
	onConfirm,
	onCancel,
}: ConfirmOptions & {
	open: boolean;
	busy?: boolean;
	children?: ReactNode;
	onConfirm: () => void;
	onCancel: () => void;
}) {
	return (
		<Dialog.Root
			open={open}
			onOpenChange={(nextOpen) => {
				if (!nextOpen && !busy) onCancel();
			}}
		>
			<Dialog.Portal>
				<Dialog.Backdrop className="notice-backdrop" />
				<Dialog.Popup className="confirm-dialog" role="alertdialog">
					<Dialog.Title>{title}</Dialog.Title>
					{description && (
						<Dialog.Description className="confirm-dialog-description">
							{description}
						</Dialog.Description>
					)}
					{children}
					<div className="confirm-dialog-actions">
						<ActionButton
							variant="secondary"
							autoFocus={tone === "danger"}
							disabled={busy}
							onClick={onCancel}
						>
							{cancelLabel}
						</ActionButton>
						<ActionButton
							variant={tone === "danger" ? "danger" : "primary"}
							autoFocus={tone !== "danger"}
							disabled={busy}
							aria-busy={busy}
							onClick={onConfirm}
						>
							{confirmLabel}
						</ActionButton>
					</div>
				</Dialog.Popup>
			</Dialog.Portal>
		</Dialog.Root>
	);
}

type ConfirmFn = (options: ConfirmOptions) => Promise<boolean>;

const ConfirmContext = createContext<ConfirmFn | null>(null);

export function ConfirmProvider({ children }: { children: ReactNode }) {
	const existing = useContext(ConfirmContext);
	const [current, setCurrent] = useState<ConfirmOptions | null>(null);
	const resolverRef = useRef<((accepted: boolean) => void) | null>(null);

	const confirm = useCallback<ConfirmFn>((options) => {
		if (resolverRef.current) {
			resolverRef.current(false);
		}
		setCurrent(options);
		return new Promise<boolean>((resolve) => {
			resolverRef.current = resolve;
		});
	}, []);

	const settle = useCallback((accepted: boolean) => {
		const resolve = resolverRef.current;
		resolverRef.current = null;
		setCurrent(null);
		resolve?.(accepted);
	}, []);

	const value = useMemo(() => confirm, [confirm]);

	if (existing) return <>{children}</>;

	return (
		<ConfirmContext.Provider value={value}>
			{children}
			<ConfirmDialog
				open={current !== null}
				title={current?.title ?? ""}
				description={current?.description}
				confirmLabel={current?.confirmLabel}
				cancelLabel={current?.cancelLabel}
				tone={current?.tone}
				onConfirm={() => settle(true)}
				onCancel={() => settle(false)}
			/>
		</ConfirmContext.Provider>
	);
}

export function useConfirm(): ConfirmFn {
	const context = useContext(ConfirmContext);
	return useCallback<ConfirmFn>(
		async (options) => {
			if (context) return context(options);
			return window.confirm(
				typeof options.description === "string"
					? `${options.title}\n${options.description}`
					: options.title,
			);
		},
		[context],
	);
}
