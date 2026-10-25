import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = file => readFileSync(resolve(root, file), 'utf8');
const json = file => JSON.parse(read(file));
const save = (file, value) => writeFileSync(resolve(root, file), JSON.stringify(value, null, 2) + '\n');
const command = process.argv[2] || 'check';
const pattern = /(\[workspace\.package\][\s\S]*?\nversion\s*=\s*)"([^"]+)"/;
const current = read('Cargo.toml').match(pattern)?.[2];
if (!current) throw new Error('Cargo.toml: workspace.package.version ausente / missing');

if (command === 'set') {
  const next = process.argv[3];
  if (!next || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/.test(next)) {
    throw new Error('Uso / Usage: npm run version:set -- 0.1.0');
  }
  const pkg = json('package.json');
  const lock = json('package-lock.json');
  pkg.version = next;
  lock.version = next;
  lock.packages[''].version = next;
  lock.packages[''].license = pkg.license;
  writeFileSync(resolve(root, 'Cargo.toml'), read('Cargo.toml').replace(pattern, (_, prefix) => `${prefix}"${next}"`));
  save('package.json', pkg);
  save('package-lock.json', lock);
  writeFileSync(resolve(root, 'web/sw.js'), read('web/sw.js').replace(/(const CACHE = "heartwire-shell-)[^"]+/, (_, prefix) => `${prefix}${next}`));
  for (const file of ['README.md', 'README.en.md']) {
    const text = read(file)
      .replace(/(\*\*(?:Versão|Version): )[^*]+(\*\*)/, (_, prefix, suffix) => `${prefix}${next}${suffix}`)
      .replace(/(https:\/\/img\.shields\.io\/badge\/HeartWire-)[^?\"]+(-79edbb)(?=\?)/, (_, prefix, suffix) => `${prefix}${next.replaceAll('-', '--')}${suffix}`)
      .replace(/HeartWire \d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?/g, `HeartWire ${next}`)
      .replace(/(https:\/\/github\.com\/saitofernando\/heartwire\/releases\/download\/v)[^/]+\/HeartWire_[^/]+_x64-setup\.exe/g, (_, prefix) => `${prefix}${next}/HeartWire_${next}_x64-setup.exe`)
      .replace(/(https:\/\/github\.com\/saitofernando\/heartwire\/releases\/tag\/v)[0-9A-Za-z.-]+/g, (_, prefix) => `${prefix}${next}`);
    writeFileSync(resolve(root, file), text);
  }
} else if (command !== 'check') {
  throw new Error('Comando inválido / Invalid command: ' + command);
}

// Metadata refreshes Cargo.lock after an intentional version change. Checks
// use --locked so CI cannot silently rewrite a stale lockfile.
const args = ['metadata', '--format-version', '1'];
if (command === 'check') args.push('--locked');
const metadata = JSON.parse(execFileSync('cargo', args, { cwd: root, encoding: 'utf8', windowsHide: true, maxBuffer: 128 * 1024 * 1024 }));
const expected = read('Cargo.toml').match(pattern)[2];
const lock = json('package-lock.json');
const versions = {
  'Cargo workspace': expected,
  'package.json': json('package.json').version,
  'package-lock.json': lock.version,
  'package-lock.json package': lock.packages[''].version,
  'PWA cache': read('web/sw.js').match(/const CACHE = "heartwire-shell-([^"]+)"/)?.[1],
};
for (const pkg of metadata.packages.filter(pkg => metadata.workspace_members.includes(pkg.id))) {
  versions[pkg.name] = pkg.version;
  if (pkg.license !== 'MIT') throw new Error(`${pkg.name}: license deve ser MIT / must be MIT`);
}
for (const file of ['README.md', 'README.en.md']) {
  const text = read(file);
  versions[file] = text.match(/\*\*(?:Versão|Version): ([^*]+)\*\*/)?.[1];
  versions[`${file} badge`] = text.match(/https:\/\/img\.shields\.io\/badge\/HeartWire-([^?\"]+)-79edbb\?/)?.[1]?.replaceAll('--', '-');
  versions[`${file} badge alt`] = text.match(/alt="HeartWire (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)"/)?.[1];
  const downloads = [...text.matchAll(/\/releases\/download\/v([^/]+)\/HeartWire_([^/]+)_x64-setup\.exe/g)];
  if (!downloads.length) throw new Error(`${file}: link de instalação ausente / missing installer link`);
  downloads.forEach((match, index) => {
    versions[`${file} download ${index + 1}`] = match[1];
    versions[`${file} installer ${index + 1}`] = match[2];
  });
  versions[`${file} release notes`] = text.match(/\/releases\/tag\/v([0-9A-Za-z.-]+)/)?.[1];
}
if (command === 'check') {
  for (const file of ['CHANGELOG.md', 'CHANGELOG.en.md']) {
    versions[file] = read(file).match(/^## (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)\s*[—-]/m)?.[1];
  }
}
for (const [name, version] of Object.entries(versions)) {
  if (version !== expected) throw new Error(`${name}: ${version}; esperado / expected ${expected}`);
}
if (json('package.json').license !== 'MIT') throw new Error('package.json: license deve ser MIT / must be MIT');
console.log(`Versões consistentes / Versions consistent: ${expected}`);
