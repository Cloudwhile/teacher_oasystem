import { toast } from "sonner";

export function notifySuccess(message: string, description?: string) {
	toast.success(message, description ? { description } : undefined);
}

export function notifyInfo(message: string, description?: string) {
	toast(message, description ? { description } : undefined);
}
