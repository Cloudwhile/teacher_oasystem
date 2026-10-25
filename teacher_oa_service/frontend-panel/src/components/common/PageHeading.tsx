import type { ReactNode } from "react";
export function PageHeading({
	title,
	eyebrow,
	children,
}: {
	title: string;
	eyebrow?: string;
	children?: ReactNode;
}) {
	return (
		<div className="page-heading">
			<div>
				{eyebrow && <p className="eyebrow">{eyebrow}</p>}
				<h1>{title}</h1>
			</div>
			<div className="heading-actions">{children}</div>
		</div>
	);
}
