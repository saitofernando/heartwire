import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const skip = new Set(['node_modules', 'target', 'artifacts', '.git']);
const files = [];
function walk(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory() && !skip.has(entry.name)) walk(path);
    else if (entry.isFile() && extname(path).toLowerCase() === '.md') files.push(path);
  }
}
walk(root);
const problems = [];
for (const file of files) {
  const text = readFileSync(file, 'utf8');
  if (text.includes('\ufffd')) problems.push(`${relative(root, file)}: caractere de substituição / invalid Unicode`);
  // Links in code fences are examples rather than navigable documentation.
  const prose = text.replace(/^```[^\n]*\n[\s\S]*?^```\s*$/gm, '');
  const pattern = /!?\[[^\]]*\]\((?:<([^>]+)>|([^\s)]+))(?:\s+"[^"]*")?\)/g;
  for (const match of prose.matchAll(pattern)) {
    const link = match[1] || match[2];
    if (/^(?:[a-z][a-z0-9+.-]*:|#)/i.test(link)) continue;
    const path = decodeURIComponent(link.split('#')[0].split('?')[0]);
    if (path && !existsSync(resolve(dirname(file), path))) problems.push(`${relative(root, file)}: link quebrado / broken link ${link}`);
  }
  for (const match of prose.matchAll(/<(?:img|a)\b[^>]*\b(?:src|href)="([^"]+)"/gi)) {
    const link = match[1];
    if (/^(?:[a-z][a-z0-9+.-]*:|#)/i.test(link)) continue;
    const path = decodeURIComponent(link.split('#')[0].split('?')[0]);
    if (path && !existsSync(resolve(dirname(file), path))) problems.push(`${relative(root, file)}: recurso HTML ausente / missing HTML asset ${link}`);
  }
}
const pairs = [
  ['README.md', 'README.en.md'], ['CHANGELOG.md', 'CHANGELOG.en.md'],
  ['CONTRIBUTING.md', 'CONTRIBUTING.en.md'], ['CREDITS.md', 'CREDITS.en.md'],
  ['VALIDACAO.md', 'VALIDACAO.en.md'],
  ['docs/pt-BR/README.md', 'docs/en/README.md'],
  ['docs/pt-BR/uso.md', 'docs/en/usage.md'],
  ['docs/pt-BR/solucao-de-problemas.md', 'docs/en/troubleshooting.md'],
  ['docs/pt-BR/seguranca.md', 'docs/en/security.md'],
  ['docs/pt-BR/desenvolvimento.md', 'docs/en/development.md'],
  ['docs/pt-BR/arquitetura.md', 'docs/en/architecture.md'],
  ['docs/pt-BR/versoes-e-lancamentos.md', 'docs/en/versioning-and-releases.md'],
];
for (const pair of pairs) for (const file of pair) {
  if (!existsSync(resolve(root, file))) {
    problems.push(`Tradução ausente / Missing translation: ${file}`);
    continue;
  }
  const counterpart = pair.find(candidate => candidate !== file);
  const navigation = relative(dirname(resolve(root, file)), resolve(root, counterpart)).replaceAll('\\', '/');
  if (!readFileSync(resolve(root, file), 'utf8').includes(`](${navigation})`)) {
    problems.push(`${file}: navegação para tradução ausente / missing translation link ${navigation}`);
  }
}
for (const file of ['LICENSE', 'NOTICE', 'licenses/THIRD_PARTY_NOTICES.txt', 'licenses/dependencies.json']) {
  if (!existsSync(resolve(root, file)) || statSync(resolve(root, file)).size === 0) problems.push(`Arquivo obrigatório / Required file: ${file}`);
}
const license = resolve(root, 'LICENSE');
if (existsSync(license) && !readFileSync(license, 'utf8').includes('Copyright (c) 2026 saitofernando')) {
  problems.push('LICENSE: autoria do HeartWire ausente / missing HeartWire copyright');
}
const version = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')).version;
for (const file of ['README.md', 'README.en.md']) {
  if (existsSync(resolve(root, file)) && !readFileSync(resolve(root, file), 'utf8').includes(`: ${version}**`)) {
    problems.push(`${file}: versão desatualizada / outdated version`);
  }
}
if (problems.length) {
  console.error(problems.join('\n'));
  process.exitCode = 1;
} else {
  console.log(`Documentação verificada / Documentation checked: ${files.length} arquivos, ${pairs.length} pares PT-BR/EN`);
}
