import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const source = path.join(root, "generated/api.raw.ts");
const destination = path.join(root, "src/types/generated");
const raw = fs.readFileSync(source, "utf8");
fs.mkdirSync(destination, { recursive: true });
for (const name of fs.readdirSync(destination)) {
	if (/^(paths|schemas|operations)-\d+\.ts$/.test(name))
		fs.unlinkSync(path.join(destination, name));
}
const imports = [];
const banner =
	"/** Generated from the Rust OpenAPI contract. Do not edit. */\n";

function pieces(body, indent) {
	const pattern = new RegExp(
		`^${" ".repeat(indent)}(?:"[^"\\n]+"|[A-Za-z_$][\\w$]*)\\??:`,
		"gm",
	);
	const starts = [...body.matchAll(pattern)].map((match) => match.index);
	return starts.map((start, index) =>
		body.slice(start, starts[index + 1] ?? body.length).trimEnd(),
	);
}

function emit(kind, body, indent) {
	const groups = [];
	let group = [];
	let lines = 0;
	for (const member of pieces(body, indent)) {
		const size = member.split("\n").length;
		if (group.length && lines + size > 450) {
			groups.push(group);
			group = [];
			lines = 0;
		}
		group.push(member);
		lines += size;
	}
	if (group.length) groups.push(group);
	return groups
		.map((members, index) => {
			const name = `${kind[0].toUpperCase()}${kind.slice(1)}Part${index + 1}`;
			const filename = `${kind}-${String(index + 1).padStart(2, "0")}`;
			const content = members
				.join("\n")
				.split("\n")
				.map((line) => line.slice(Math.max(0, indent - 4)))
				.join("\n");
			const references = ["components", "operations", "paths", "$defs"].filter(
				(ref) => content.includes(`${ref}[`),
			);
			const header = references.length
				? `import type { ${references.join(", ")} } from "../api.generated";\n`
				: "";
			const output = `${banner}${header}export interface ${name} {\n${content}\n}\n`;
			if (output.split("\n").length > 950)
				throw new Error(`Generated API member too large: ${name}`);
			fs.writeFileSync(path.join(destination, `${filename}.ts`), output);
			imports.push(`import type { ${name} } from "./generated/${filename}";`);
			return name;
		})
		.join(" & ");
}

const index = raw.replace(
	/export interface (paths|components|operations) \{([\s\S]*?)^\}/gm,
	(_match, name, body) => {
		if (name === "components") {
			return `export interface components {${body.replace(/^ {4}schemas: \{([\s\S]*?)^ {4}\};/m, (_schemas, members) => `    schemas: ${emit("schemas", members, 8)};`)}\n}`;
		}
		return `export type ${name} = ${emit(name, body, 4)};`;
	},
);
if (!imports.length) throw new Error("Unsupported OpenAPI TypeScript output");
fs.writeFileSync(
	path.join(root, "src/types/api.generated.ts"),
	`${banner}${imports.join("\n")}\n${index}`,
);
fs.unlinkSync(source);
