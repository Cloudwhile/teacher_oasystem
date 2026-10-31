import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";
import { HttpError } from "@/services/httpClient";
import { type AuthResponse, authApi } from "./api";

type AuthStatus = "loading" | "ready" | "anonymous" | "error" | "forbidden";
type AuthContextValue = {
	status: AuthStatus;
	session: AuthResponse | null;
	refresh: () => Promise<void>;
	login: (username: string, password: string) => Promise<void>;
	changePassword: (current: string, password: string) => Promise<void>;
	logout: () => Promise<void>;
};
const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
	const [status, setStatus] = useState<AuthStatus>("loading");
	const [session, setSession] = useState<AuthResponse | null>(null);
	const applyError = useCallback((error: unknown) => {
		setSession(null);
		setStatus(
			error instanceof HttpError && error.status === 401
				? "anonymous"
				: error instanceof HttpError && error.status === 403
					? "forbidden"
					: "error",
		);
	}, []);
	const refresh = useCallback(async () => {
		setStatus("loading");
		try {
			setSession(await authApi.me());
			setStatus("ready");
		} catch (error) {
			applyError(error);
		}
	}, [applyError]);
	useEffect(() => {
		const controller = new AbortController();
		authApi
			.me(controller.signal)
			.then((value) => {
				if (!controller.signal.aborted) {
					setSession(value);
					setStatus("ready");
				}
			})
			.catch((error: unknown) => {
				if (!controller.signal.aborted) applyError(error);
			});
		return () => controller.abort();
	}, [applyError]);
	const login = async (username: string, password: string) => {
		setSession(await authApi.login({ username, password }));
		setStatus("ready");
	};
	const logout = async () => {
		if (session) {
			try {
				await authApi.logout(session.csrf_token);
			} catch (error) {
				if (!(error instanceof HttpError && error.status === 401)) throw error;
			}
		}
		setSession(null);
		setStatus("anonymous");
	};
	const changePassword = async (current: string, password: string) => {
		if (!session) return;
		setSession(
			await authApi.changePassword(current, password, session.csrf_token),
		);
	};
	return (
		<AuthContext.Provider
			value={{ status, session, refresh, login, logout, changePassword }}
		>
			{children}
		</AuthContext.Provider>
	);
}

export function useAuth() {
	const context = useContext(AuthContext);
	if (!context) throw new Error("AuthProvider is missing");
	return context;
}
