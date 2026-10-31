import { Tabs } from "@base-ui/react/tabs";
import type { ReactNode } from "react";

export function LoginMethods({
	account,
	wecom,
	wecomEnabled,
	busy,
}: {
	account: ReactNode;
	wecom: ReactNode;
	wecomEnabled: boolean;
	busy: boolean;
}) {
	return (
		<Tabs.Root defaultValue="account">
			<Tabs.List className="login-tabs" aria-label="登录方式">
				<Tabs.Tab className="login-tab" value="account">
					账号登录
				</Tabs.Tab>
				<Tabs.Tab
					className="login-tab"
					value="wecom"
					disabled={!wecomEnabled || busy}
				>
					企业微信登录
				</Tabs.Tab>
			</Tabs.List>
			<Tabs.Panel className="login-method-panel" value="account">
				{account}
			</Tabs.Panel>
			<Tabs.Panel className="login-method-panel" value="wecom">
				{wecom}
			</Tabs.Panel>
		</Tabs.Root>
	);
}
