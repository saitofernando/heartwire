#!/usr/bin/env node
// SPDX-License-Identifier: MIT
// Collect original notices from locked packages; never synthesize copyrights.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = "x86_64-pc-windows-msvc";
const args = new Set(process.argv.slice(2));
for (const arg of args) {
  if (!["--check", "--refresh-upstream", "--help"].includes(arg)) throw new Error(`Unknown argument: ${arg}`);
}
if (args.has("--help")) {
  console.log("node scripts/third-party-notices.mjs [--check | --refresh-upstream]\n--check: compare outputs without writing; --refresh-upstream: cache omitted licenses from exact upstream commits.");
  process.exit(0);
}
if (args.has("--check") && args.has("--refresh-upstream")) throw new Error("--check cannot refresh upstream files.");

const normalize = (value) => value.replace(/^\uFEFF/, "").replace(/\r\n?/g, "\n");
const read = (name) => normalize(fs.readFileSync(name, "utf8"));
const hash = (value) => crypto.createHash("sha256").update(value).digest("hex");
const slash = (value) => value.split(path.sep).join("/");
const order = (a, b) => a < b ? -1 : a > b ? 1 : 0;
const json = (value) => `${JSON.stringify(value, null, 2)}\n`;
const expression = (value) => typeof value === "string" ? value.replace(/\s*\/\s*/g, " OR ") : "NOASSERTION";
const publicUrl = (value) => {
  if (!value) return null;
  try {
    const parsed = new URL(value.replace(/^git\+/, ""));
    if (!["https:", "http:"].includes(parsed.protocol) || parsed.username || parsed.password) return null;
    parsed.search = "";
    parsed.hash = "";
    return parsed.toString();
  } catch { return null; }
};
const licenseName = /^(?:unlicense|licen[sc]e|copying|notice|copyright|patents?)(?:[-._]|$)/i;
const ignoredDirs = new Set([".git", "node_modules", "target", ".github"]);
function licenseFiles(directory, relative = "", depth = 0) {
  if (depth > 12) return [];
  const found = [];
  const inLicenseDirectory = relative.split(path.sep).some((part) => /^licen[sc]es?$/i.test(part));
  for (const item of fs.readdirSync(path.join(directory, relative), { withFileTypes: true }).sort((a, b) => order(a.name, b.name))) {
    const name = path.join(relative, item.name);
    if (item.isDirectory() && !ignoredDirs.has(item.name)) found.push(...licenseFiles(directory, name, depth + 1));
    else if (item.isFile() && (licenseName.test(item.name) || (inLicenseDirectory && /(?:\.(?:txt|md|rst|html|license)|^[^.]+)$/i.test(item.name)))) {
      // SDK binary names / image fixtures are never copied into notice files.
      if (!/\.(?:png|jpe?g|gif|ico|pdf|dll|lib|exe|wasm|gz|zip|woff2?)$/i.test(item.name)) found.push(name);
    }
  }
  return found;
}

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--locked", "--format-version", "1", "--filter-platform", target], {
  cwd: root, encoding: "utf8", maxBuffer: 128 * 1024 * 1024,
  stdio: ["ignore", "pipe", "pipe"],
}));
const packages = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
const workspace = new Set(metadata.workspace_members);
const scopes = new Map();
const queue = [...workspace].map((id) => [id, "runtime"]);
for (let cursor = 0; cursor < queue.length; cursor++) {
  const [id, scope] = queue[cursor];
  if (!scopes.has(id)) scopes.set(id, new Set());
  if (scopes.get(id).has(scope)) continue;
  scopes.get(id).add(scope);
  for (const dep of nodes.get(id)?.deps ?? []) {
    for (const kind of dep.dep_kinds) {
      if (kind.kind === "dev" && !workspace.has(id)) continue;
      const isProcMacro = packages.get(dep.pkg)?.targets.some((target) => target.kind.includes("proc-macro"));
      const childScope = scope === "development" || kind.kind === "dev" ? "development"
        : scope === "build" || kind.kind === "build" || isProcMacro ? "build" : "runtime";
      queue.push([dep.pkg, childScope]);
    }
  }
}

const supplementPath = path.join(root, "licenses/supplemental.json");
const supplements = fs.existsSync(supplementPath) ? JSON.parse(read(supplementPath)) : { schemaVersion: 1, supplements: [], components: [] };
if (args.has("--refresh-upstream")) {
  for (const pkg of metadata.packages.filter((pkg) => pkg.source && scopes.has(pkg.id)).sort((a, b) => order(`${a.name}@${a.version}`, `${b.name}@${b.version}`))) {
    const id = `cargo:${pkg.name}@${pkg.version}`;
    const directory = path.dirname(pkg.manifest_path);
    if (licenseFiles(directory).length || pkg.license_file || supplements.supplements.some((item) => item.appliesTo.includes(id))) continue;
    const vcsPath = path.join(directory, ".cargo_vcs_info.json");
    // Monorepos may point at a subfolder (…/tree/main/crates/x); licenses live at the root.
    const repository = publicUrl(pkg.repository)?.replace(/\.git$/, "").match(/^https:\/\/github\.com\/[^/]+\/[^/]+/)?.[0];
    if (!fs.existsSync(vcsPath) || !repository?.startsWith("https://github.com/")) continue;
    const commit = JSON.parse(read(vcsPath)).git?.sha1;
    if (!/^[a-f0-9]{40}$/.test(commit ?? "")) continue;
    const documents = [];
    for (const filename of ["LICENSE", "LICENSE.md", "LICENSE.txt", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE-APACHE-2.0", "COPYING", "NOTICE", "LICENSES/MIT.txt", "LICENSES/Apache-2.0.txt"]) {
      const origin = `${repository.replace("https://github.com/", "https://raw.githubusercontent.com/")}/${commit}/${filename}`;
      const response = await fetch(origin);
      if (!response.ok) continue;
      const content = normalize(await response.text());
      if (!content.trim() || /^\s*</.test(content)) throw new Error(`Unexpected license response: ${origin}`);
      const relative = `licenses/upstream/${pkg.name}-${pkg.version}/${filename}`;
      fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
      fs.writeFileSync(path.join(root, relative), content);
      documents.push({ path: relative, origin, sha256: hash(content) });
    }
    if (documents.length) supplements.supplements.push({ appliesTo: [id], reason: "License omitted from the published crate; original files at the commit recorded in .cargo_vcs_info.json.", documents });
    else console.warn(`No upstream license found for ${id}; review required.`);
  }
  supplements.supplements.sort((a, b) => order(a.appliesTo[0], b.appliesTo[0]));
  fs.mkdirSync(path.dirname(supplementPath), { recursive: true });
  fs.writeFileSync(supplementPath, json(supplements));
}

const records = [];
const notices = new Map();
const issues = [];
function document(file, label, origin) {
  const text = read(file);
  if (!text.trim()) throw new Error(`Empty license document: ${label}`);
  const digest = hash(text);
  return { label, origin, sha256: digest, text };
}
function supplemented(id) {
  return supplements.supplements.filter((entry) => entry.appliesTo.includes(id)).flatMap((entry) => entry.documents.map((doc) => {
    const resolved = path.resolve(root, doc.path);
    if (!resolved.startsWith(`${path.join(root, "licenses")}${path.sep}`)) throw new Error("Supplement must remain inside licenses/.");
    const value = document(resolved, doc.path, publicUrl(doc.origin));
    if (value.sha256 !== doc.sha256) throw new Error(`Supplement hash mismatch: ${doc.path}`);
    return value;
  }));
}
function addRecord(record, documents) {
  const unique = [...new Map(documents.map((doc) => [`${doc.label}:${doc.sha256}`, doc])).values()].sort((a, b) => order(a.label, b.label));
  record.licenseFiles = unique.map(({ label, origin, sha256 }) => ({ path: label, origin, sha256 }));
  if (!unique.length && record.category !== "external-system-runtime") issues.push({ id: record.id, reason: "No original license/notice text located. Review before distribution." });
  if (record.licenseExpression === "NOASSERTION") issues.push({ id: record.id, reason: "Package declares no license expression." });
  records.push(record);
  notices.set(record.id, unique);
}

for (const pkg of metadata.packages.filter((pkg) => pkg.source && scopes.has(pkg.id))) {
  const id = `cargo:${pkg.name}@${pkg.version}`;
  if (!/^registry\+https:\/\/(?:github\.com\/rust-lang\/crates\.io-index|index\.crates\.io\/?)/.test(pkg.source)) {
    throw new Error(`Unsupported Cargo source for ${id}; add a verified source mapping before distribution.`);
  }
  const directory = path.dirname(pkg.manifest_path);
  const files = licenseFiles(directory);
  if (pkg.license_file) {
    const absolute = path.resolve(directory, pkg.license_file);
    const relative = path.relative(directory, absolute);
    if (relative.startsWith("..") || path.isAbsolute(relative)) throw new Error(`License file escapes package source: ${id}`);
    if (!files.includes(relative)) files.push(relative);
  }
  const source = `https://crates.io/api/v1/crates/${encodeURIComponent(pkg.name)}/${encodeURIComponent(pkg.version)}/download`;
  const documents = files.map((file) => document(path.join(directory, file), `${id}/${slash(file)}`, source));
  documents.push(...supplemented(id));
  addRecord({ id, ecosystem: "cargo", name: pkg.name, version: pkg.version, licenseExpression: expression(pkg.license), declaredLicense: pkg.license,
    scopes: [...scopes.get(pkg.id)].sort(order), category: scopes.get(pkg.id).has("runtime") ? "rust-runtime" : "rust-build-development",
    source, repository: publicUrl(pkg.repository) }, documents);
}

const lock = JSON.parse(read(path.join(root, "package-lock.json")));
const platformMatches = (values, actual) => !values || (!values.includes(`!${actual}`) && (!values.some((value) => !value.startsWith("!")) || values.includes(actual)));
const npm = new Map();
for (const [location, pkg] of Object.entries(lock.packages ?? {}).sort(([a], [b]) => order(a, b))) {
  if (!location || !platformMatches(pkg.os, "win32") || !platformMatches(pkg.cpu, "x64")) continue;
  const directory = path.join(root, location);
  const manifestPath = path.join(directory, "package.json");
  if (!fs.existsSync(manifestPath)) throw new Error(`Locked Windows npm package is missing: ${location}. Run npm ci with development dependencies on Windows x64.`);
  const installed = JSON.parse(read(manifestPath));
  if (installed.version !== pkg.version) throw new Error(`npm package version drift: ${location}. Run npm ci.`);
  const name = installed.name;
  const id = `npm:${name}@${pkg.version}`;
  if (!npm.has(id)) npm.set(id, { id, directory, installed, pkg, locations: [], scopes: new Set() });
  const entry = npm.get(id);
  entry.locations.push(location);
  entry.scopes.add(pkg.dev ? "build-development" : "miniapp-source");
}
for (const { id, directory, installed, pkg, locations, scopes: npmScopes } of npm.values()) {
  const source = publicUrl(pkg.resolved) ?? `https://registry.npmjs.org/${encodeURIComponent(installed.name)}/-/${installed.name.split("/").at(-1)}-${pkg.version}.tgz`;
  const documents = licenseFiles(directory).map((file) => document(path.join(directory, file), `${id}/${slash(file)}`, source));
  // Platform-specific build-tool packages omit licenses; their publisher ships
  // the original license with the matching parent package at the same version.
  const parentName = installed.name.startsWith("@esbuild/") ? "esbuild" : installed.name.startsWith("@rollup/rollup-") ? "rollup" : null;
  if (!documents.length && parentName) {
    const parentDir = path.join(root, "node_modules", parentName);
    const parent = JSON.parse(read(path.join(parentDir, "package.json")));
    if (parent.version !== pkg.version) throw new Error(`License parent version mismatch for ${id}`);
    for (const file of licenseFiles(parentDir)) documents.push(document(path.join(parentDir, file), `npm:${parentName}@${parent.version}/${slash(file)}`, `https://registry.npmjs.org/${encodeURIComponent(parentName)}/-/${parentName.split("/").at(-1)}-${parent.version}.tgz`));
  }
  documents.push(...supplemented(id));
  const declared = typeof (pkg.license ?? installed.license) === "object" ? (pkg.license ?? installed.license).type : pkg.license ?? installed.license;
  addRecord({ id, ecosystem: "npm", name: installed.name, version: pkg.version, licenseExpression: expression(declared), declaredLicense: declared ?? null,
    scopes: [...npmScopes].sort(order), category: npmScopes.has("miniapp-source") ? "miniapp-source" : "npm-build-development", source,
    repository: publicUrl(typeof installed.repository === "string" ? installed.repository : installed.repository?.url), lockfileLocations: locations.sort(order) }, documents);
}

const opus = metadata.packages.find((pkg) => pkg.name === "audiopus_sys" && scopes.has(pkg.id));
if (opus) {
  const directory = path.join(path.dirname(opus.manifest_path), "opus");
  if (!fs.existsSync(path.join(directory, "COPYING"))) throw new Error("Vendored libopus COPYING is missing; review native linking before distribution.");
  addRecord({ id: `native:libopus-in-audiopus_sys@${opus.version}`, ecosystem: "bundled-native", name: "libopus", version: `vendored snapshot in audiopus_sys ${opus.version}`,
    licenseExpression: "BSD-3-Clause", declaredLicense: "BSD-3-Clause; patent grants referenced in COPYING and LICENSE_PLEASE_READ.txt", scopes: ["runtime"], category: "bundled-native-codec",
    source: `https://crates.io/api/v1/crates/audiopus_sys/${opus.version}/download`, repository: "https://github.com/xiph/opus", includedBy: `cargo:audiopus_sys@${opus.version}` },
  licenseFiles(directory).map((file) => document(path.join(directory, file), `cargo:audiopus_sys@${opus.version}/opus/${slash(file)}`, `https://crates.io/api/v1/crates/audiopus_sys/${opus.version}/download`)));
}
for (const component of supplements.components) {
  if (!records.some((record) => component.includedBy?.includes(record.id))) continue;
  addRecord({ ...component, includedBy: [...component.includedBy].sort(order) }, supplemented(component.id));
}
records.sort((a, b) => order(a.id, b.id));
issues.sort((a, b) => order(a.id, b.id));
const inputs = ["Cargo.lock", "package-lock.json", "Cargo.toml", "crates/bridge-core/Cargo.toml", "crates/heartwire/Cargo.toml", "package.json", "licenses/supplemental.json"];
const inventory = { schemaVersion: 1, target, scope: "Conservative locked Windows x64 package inventory: runtime, build and development packages are labeled separately; original package notices plus identified bundled native components. External system runtimes are labeled and are not relicensed.",
  inputs: Object.fromEntries(inputs.filter((file) => fs.existsSync(path.join(root, file))).map((file) => [file, { sha256: hash(read(path.join(root, file))) }])),
  summary: { packages: records.length, cargo: records.filter((r) => r.ecosystem === "cargo").length, npm: records.filter((r) => r.ecosystem === "npm").length, issues: issues.length }, issues, dependencies: records };

const header = `AVISOS DE TERCEIROS / THIRD-PARTY NOTICES\n\nGerado por scripts/third-party-notices.mjs para ${target}.\nGenerated for ${target} from locked Cargo and npm packages.\n\nEscopo: inventário conservador de código-fonte de dependências de execução,\ncompilação e desenvolvimento, identificadas separadamente. A presença de um\npacote de ferramenta não significa que seu executável esteja no aplicativo.\nInclui avisos originais disponíveis nas árvores de fontes e complementos\nverificados nos commits originais e a libopus. Drivers externos, como o\nOpenGL da placa de vídeo, são componentes separados do sistema.\n\nScope: conservative source-package inventory covering runtime, build and\ndevelopment dependencies, labeled separately. Build-tool packages do not\nimply their executables are shipped. Original license/notice files retain\ntheir copyright and terms; translations are not substituted for originals.\nThis file does not relicense third-party code or select a different license\nfor composite SPDX expressions. See dependencies.json for sources and hashes.\n\nVB-CABLE is not distributed. Windows, Rust/Node/CMake toolchains and system\nruntimes are not relicensed by this project. This inventory covers the locked\npackage source trees and the explicitly identified bundled native components;\nit is not an inventory of every tool installed on the build machine.\n\n${issues.length ? `REVIEW REQUIRED: ${issues.length} missing-license/source issue(s); see dependencies.json.\n\n` : "No missing original license documents identified within this inventory.\n\n"}`;
let aggregate = header;
for (const record of records) {
  aggregate += `${"=".repeat(78)}\n${record.id}\nLicense expression: ${record.licenseExpression}\nCategory: ${record.category}\nScopes: ${record.scopes.join(", ")}\nSource: ${record.source ?? "See package source"}\n`;
  if (record.distribution) aggregate += `${record.distribution}\n`;
  aggregate += "\n";
  for (const doc of notices.get(record.id)) {
    aggregate += `--- ${doc.label} ---\nOriginal source: ${doc.origin ?? record.source}\nSHA-256 (UTF-8/LF): ${doc.sha256}\n\n${doc.text}${doc.text.endsWith("\n") ? "" : "\n"}\n`;
  }
}
const mpl = records.filter((record) => record.licenseExpression.includes("MPL-2.0"));
const mplTable = mpl.map((record) => `| ${record.name} ${record.version} | ${record.scopes.join(", ")} | [Fonte / Source](${record.source}) |`).join("\n");
const guide = `# Avisos de terceiros / Third-party notices

O projeto tem sua própria [licença](LICENSE). As dependências mantêm suas
licenças, direitos autorais e avisos originais; este documento não as
relicencia. O inventário atual cobre **${inventory.summary.cargo} pacotes Cargo,
${inventory.summary.npm} pacotes npm e os componentes nativos identificados**
para \`${target}\`, incluindo ferramentas de compilação e testes
rotuladas separadamente. Uma ferramenta inventariada pode estar ausente do
executável distribuído.

- [Textos integrais e avisos para acompanhar a distribuição](licenses/THIRD_PARTY_NOTICES.txt).
- [Inventário, expressões de licença, versões, fontes, escopos e hashes](licenses/dependencies.json).
- [Proveniência dos textos omitidos pelos pacotes e complementos verificados](licenses/supplemental.json).

O libopus embutido em \`audiopus_sys\` tem sua própria licença BSD-3-Clause
e referências de patentes, preservadas nos avisos. Isso é separado da
licença ISC do wrapper. Drivers de vídeo e fontes do Windows usados em tempo
de execução são componentes do sistema, com termos próprios, e não são
distribuídos. VB-CABLE e seus binários não são distribuídos por este projeto.

Os componentes abaixo declaram MPL-2.0 e conservam essa licença. As fontes
exatas correspondentes às versões resolvidas estão disponíveis nos arquivos
de código-fonte indicados; os avisos completos acompanham a distribuição.
As demais dependências também têm links de fonte no inventário JSON.

| Componente / Component | Escopo / Scope | Código-fonte correspondente / Corresponding source |
| --- | --- | --- |
${mplTable}

Para atualizar os avisos em uma máquina Windows x64 com Node e Cargo:

\`\`\`powershell
npm ci
cargo fetch --locked
node scripts/third-party-notices.mjs
node scripts/third-party-notices.mjs --check
\`\`\`

O gerador usa apenas módulos nativos do Node. Os arquivos são ordenados e
determinísticos, sem data de geração nem caminhos absolutos da máquina.
\`--check\` verifica conteúdo, hashes dos locks/manifests e complementos;
retorna erro se os arquivos estiverem desatualizados ou houver licença
original pendente. A opção de manutenção \`--refresh-upstream\` consulta
apenas os commits públicos registrados pelos próprios crates para recuperar
licenças omitidas e grava sua proveniência. Um SDK nativo atualizado exige
revisar e registrar os novos textos originais em \`licenses/supplemental.json\`.
Os textos originais mantêm seu idioma, inclusive avisos que proíbem tradução.

## English

The [project license](LICENSE) does not replace third-party licenses.
The [complete notices](licenses/THIRD_PARTY_NOTICES.txt) reproduce original
license and notice texts; the [inventory](licenses/dependencies.json)
identifies locked versions, source archives, license expressions, file hashes
and runtime/build/development scopes. Source archives for MPL-2.0 components
are linked in the table above. Build tools are included conservatively and
are not necessarily shipped with the application.

Bundled libopus has separate notices from its Rust wrapper. Graphics drivers
and Windows fonts used at run time are system components with their own terms
and are not redistributed; VB-CABLE is not redistributed either.
The inventory covers locked package source trees and explicitly identified
native components, rather than all software installed on a build machine.

Regenerate using the commands above and include the project LICENSE and the
complete notices with binary releases. The generator needs no npm production
dependency, normalizes line endings, preserves original copyright notices,
and fails its check when generated files drift or original notices are missing.
Verified upstream supplements are checked into \`licenses/upstream/\` with
public source URLs and SHA-256 hashes in \`licenses/supplemental.json\`.
`;
const outputs = new Map([["licenses/dependencies.json", json(inventory)], ["licenses/THIRD_PARTY_NOTICES.txt", aggregate], ["THIRD_PARTY_NOTICES.md", guide]]);
const changed = [];
for (const [relative, content] of outputs) {
  const destination = path.join(root, relative);
  if (!fs.existsSync(destination) || read(destination) !== content) changed.push(relative);
  if (!args.has("--check")) { fs.mkdirSync(path.dirname(destination), { recursive: true }); fs.writeFileSync(destination, content); }
}
console.log(`${args.has("--check") ? "Checked" : "Generated"}: ${inventory.summary.cargo} Cargo, ${inventory.summary.npm} npm, ${records.length} total components; ${issues.length} issue(s).`);
if (issues.length) for (const issue of issues) console.error(`${issue.id}: ${issue.reason}`);
if (args.has("--check") && changed.length) console.error(`Outdated generated notices: ${changed.join(", ")}. Run node scripts/third-party-notices.mjs.`);
if (issues.length || (args.has("--check") && changed.length)) process.exitCode = 1;
