import {
	isRouteErrorResponse,
	Link,
	useLocation,
	useNavigate,
	useRouteError,
} from "react-router-dom";
import { AlertCircle, ArrowLeft, FileText, Refresh } from "reicon-react";
import { ActionButton } from "@/components/common/ActionButton";
import {
	StandaloneStatusShell,
	StatusStateCard,
	type StatusTone,
} from "@/components/common/StatusState";
import { appPaths } from "@/routes/routePaths";

function isChunkLoadFailure(error: unknown): boolean {
	if (!(error instanceof Error)) return false;
	return /Failed to fetch dynamically imported module|Importing a module script failed|ChunkLoadError|error loading dynamically imported module/i.test(
		error.message,
	);
}

function NotFoundOrErrorContent({
	inShell,
	kind,
}: {
	inShell: boolean;
	kind: "not-found" | "chunk" | "crash";
}) {
	const location = useLocation();
	const navigate = useNavigate();
	const isLoginPath = location.pathname.startsWith(appPaths.login);
	const homePath = isLoginPath ? appPaths.login : appPaths.dashboard;
	const homeLabel = isLoginPath ? "返回登录页" : "返回工作台";
	const canGoBack = typeof window !== "undefined" && window.history.length > 1;

	const meta: Record<
		"not-found" | "chunk" | "crash",
		{
			tone: StatusTone;
			code: string;
			title: string;
			description: string;
		}
	> = {
		"not-found": {
			tone: "neutral",
			code: "404 · 页面未找到",
			title: "未找到您访问的页面",
			description:
				"该链接可能已变更、输入有误，或对应记录已被移除。您可以返回上一页或回到工作台继续办理。",
		},
		chunk: {
			tone: "warning",
			code: "版本已更新",
			title: "检测到系统版本已更新",
			description: "当前浏览器缓存的页面资源已过期，刷新页面即可加载最新版本。",
		},
		crash: {
			tone: "danger",
			code: "页面异常",
			title: "当前页面加载出现异常",
			description:
				"页面渲染时遇到意外问题，您已保存的数据不受影响。请尝试刷新页面或返回首页。",
		},
	};

	const current = meta[kind];
	const actions =
		kind === "not-found" ? (
			<>
				{canGoBack && (
					<ActionButton
						variant="secondary"
						onClick={() => {
							navigate(-1);
						}}
					>
						返回上一页
					</ActionButton>
				)}
				<Link
					className="action-button action-button-primary"
					to={homePath}
					replace
				>
					<ArrowLeft aria-hidden="true" />
					{homeLabel}
				</Link>
			</>
		) : (
			<>
				<ActionButton
					variant="primary"
					onClick={() => {
						window.location.reload();
					}}
				>
					<Refresh aria-hidden="true" />
					刷新页面
				</ActionButton>
				{location.pathname !== homePath && (
					<Link
						className="action-button action-button-secondary"
						to={homePath}
						replace
					>
						<ArrowLeft aria-hidden="true" />
						{homeLabel}
					</Link>
				)}
			</>
		);

	const card = (
		<StatusStateCard
			framed={!inShell}
			role={kind === "not-found" ? undefined : "alert"}
			tone={current.tone}
			icon={kind === "not-found" ? <FileText /> : <AlertCircle />}
			code={current.code}
			title={current.title}
			description={current.description}
			actions={actions}
		/>
	);

	if (inShell) {
		return <div className="panel status-panel">{card}</div>;
	}
	return <StandaloneStatusShell>{card}</StandaloneStatusShell>;
}

export function ErrorPage({ inShell = false }: { inShell?: boolean }) {
	return <NotFoundOrErrorContent inShell={inShell} kind="not-found" />;
}

export function RouteErrorPage({ inShell = false }: { inShell?: boolean }) {
	const error = useRouteError();
	if (isRouteErrorResponse(error) && error.status === 404) {
		return <NotFoundOrErrorContent inShell={inShell} kind="not-found" />;
	}
	if (isChunkLoadFailure(error)) {
		return <NotFoundOrErrorContent inShell={inShell} kind="chunk" />;
	}
	return <NotFoundOrErrorContent inShell={inShell} kind="crash" />;
}
