import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { extname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { connectCdp } from '../../scripts/cdp.mjs';

// Windows browser smoke, separate from portable `npm run test:pwa` unit tests.
// Downloads are denied in this isolated browser: no profile is installed.
const root = fileURLToPath(new URL('../', import.meta.url));
const profileRequests = [];
const assets = new Set(['/app.js', '/media-session.js', '/styles.css', '/icons/heartwire.svg', '/icons/apple-touch-icon.png']);
const fixture = createServer(async (request, response) => {
  const url = new URL(request.url, 'http://127.0.0.1');
  if (url.pathname === '/status') {
    response.setHeader('Content-Type', 'application/json');
    response.end(JSON.stringify({ running: true, mode: 'listen', connectedPeers: 0 }));
    return;
  }
  if (url.pathname === '/install/heartwire.mobileconfig') {
    profileRequests.push(url.searchParams.get('lang'));
    response.writeHead(200, {
      'Content-Type': 'application/x-apple-aspen-config',
      'Content-Disposition': 'attachment; filename="heartwire-ui-test.mobileconfig"',
      'Cache-Control': 'no-store',
    });
    response.end('<?xml version="1.0"?><plist version="1.0"><dict><key>PayloadDisplayName</key><string>HeartWire UI test</string></dict></plist>');
    return;
  }
  const file = url.pathname === '/' ? resolve(root, 'index.html') : assets.has(url.pathname) ? resolve(root, '.' + url.pathname) : null;
  if (!file) { response.writeHead(404); response.end(); return; }
  try {
    const content = await readFile(file);
    response.setHeader('Content-Type', ({ '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png' })[extname(file)]);
    response.end(content);
  } catch { response.writeHead(404); response.end(); }
});
await new Promise(resolveListen => fixture.listen(1452, '127.0.0.1', resolveListen));
const profile = resolve(root, '../artifacts/edge-webclip-ui-smoke');
await mkdir(profile, { recursive: true });
const edge = spawn('C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe', [
  '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
  '--remote-debugging-port=9342', `--user-data-dir=${profile}`, 'about:blank',
], { windowsHide: true, stdio: 'ignore' });
let browser;
try {
  browser = await connectCdp(9342);
  const { command, evaluate, waitFor } = browser;
  await command('Browser.setDownloadBehavior', { behavior: 'deny' });
  await command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
  await command('Emulation.setUserAgentOverride', { userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 27_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Mobile/15E148 Safari/604.1' });
  await command('Storage.clearDataForOrigin', { origin: 'http://127.0.0.1:1452', storageTypes: 'all' });
  await command('Page.navigate', { url: 'http://127.0.0.1:1452/?mode=mic&lang=en' });
  await waitFor(`document.getElementById('connect') && !document.getElementById('connect').disabled`);
  assert.equal(await evaluate(`document.getElementById('install-banner').hidden`), false);
  await evaluate(`document.getElementById('show-install-guide').click()`);
  assert.equal(await evaluate(`document.getElementById('settings').hidden`), false);
  assert.equal(await evaluate(`document.getElementById('webclip-instructions').open`), true);
  assert.equal(profileRequests.length, 0, 'Opening guidance must not download a profile');
  await evaluate(`document.getElementById('webclip-instructions').open=false`);
  assert.equal(await evaluate(`document.getElementById('install-webclip').textContent`), 'Install with icon');
  assert.equal(await evaluate(`document.getElementById('install-webclip').href`), 'http://127.0.0.1:1452/install/heartwire.mobileconfig?lang=en');
  assert.equal(await evaluate(`document.getElementById('webclip-instructions').open`), false);
  assert.match(await evaluate(`document.getElementById('install-note').textContent`), /Share, then Add to Home Screen/);
  assert.match(await evaluate(`document.getElementById('webclip-disclaimer').textContent`), /Finish installation in iPhone Settings/);
  assert.equal(await evaluate(`document.getElementById('install-state').hidden`), true);
  await evaluate(`document.getElementById('language-select').value='pt-BR';document.getElementById('language-select').dispatchEvent(new Event('change'));document.getElementById('webclip-instructions').open=false`);
  const installScreenshot = await command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
  await writeFile(resolve(root, '../artifacts/webclip-before-install.png'), Buffer.from(installScreenshot.data, 'base64'));
  await evaluate(`document.getElementById('language-select').value='en';document.getElementById('language-select').dispatchEvent(new Event('change'))`);
  assert.equal(profileRequests.length, 0, 'Showing settings must not download a configuration profile');
  await evaluate(`document.getElementById('install-webclip').click()`);
  assert.equal(await evaluate(`document.getElementById('webclip-instructions').open`), true);
  for (let attempt = 0; attempt < 50 && profileRequests.length < 1; attempt++) await new Promise(wait => setTimeout(wait, 40));
  assert.deepEqual(profileRequests, ['en'], 'Only the explicit English link requests a profile');
  assert.match(await evaluate(`document.getElementById('install-state').textContent`), /Installation requested/);
  assert.equal(await evaluate(`document.getElementById('install-state').classList.contains('confirmed')`), false, 'A denied or cancelled download must never count as installation');
  assert.match(await evaluate(`document.getElementById('webclip-instructions').textContent`), /Profile Downloaded → Install/);
  assert.match(await evaluate(`document.getElementById('webclip-instructions').textContent`), /does not configure VPN, certificates, or remote management/);
  assert.match(await evaluate(`document.getElementById('webclip-instructions').textContent`), /8 minutes/);

  await evaluate(`document.getElementById('language-select').value='pt-BR';document.getElementById('language-select').dispatchEvent(new Event('change'))`);
  assert.equal(await evaluate(`document.getElementById('install-webclip').textContent`), 'Baixar perfil novamente');
  assert.equal(await evaluate(`document.getElementById('install-webclip').href`), 'http://127.0.0.1:1452/install/heartwire.mobileconfig?lang=pt-BR');
  assert.equal(profileRequests.length, 1, 'Changing language must not trigger a profile download');
  assert.match(await evaluate(`document.getElementById('webclip-instructions').textContent`), /Remova o atalho antigo só depois que o novo funcionar/);
  assert.equal(await evaluate('mode'), 'listen', 'Installation UI keeps the live server audio mode');
  await command('Page.reload');
  await waitFor(`document.getElementById('connect') && !document.getElementById('connect').disabled`);
  await evaluate(`document.getElementById('mobile-tab-settings').click()`);
  assert.match(await evaluate(`document.getElementById('install-state').textContent`), /Instalação solicitada/);
  assert.equal(await evaluate(`document.getElementById('webclip-instructions').open`), true, 'Returning to the page retains the pending instructions');
  const screenshot = await command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
  await writeFile(resolve(root, '../artifacts/webclip-settings.png'), Buffer.from(screenshot.data, 'base64'));

  // The Home Screen context may have separate storage; its launch mode is the
  // evidence. No profile API, elapsed timer, or download callback is simulated.
  await command('Storage.clearDataForOrigin', { origin: 'http://127.0.0.1:1452', storageTypes: 'all' });
  const standalone = await command('Page.addScriptToEvaluateOnNewDocument', { source: `Object.defineProperty(navigator, 'standalone', {value:true,configurable:true})` });
  await command('Page.navigate', { url: 'http://127.0.0.1:1452/?lang=pt-BR' });
  await waitFor(`document.getElementById('connect') && !document.getElementById('connect').disabled`);
  await evaluate(`document.getElementById('mobile-tab-settings').click()`);
  assert.equal(await evaluate(`document.getElementById('install-state').textContent`), 'Aberto pela Tela de Início');
  assert.equal(await evaluate(`document.getElementById('install-state').classList.contains('confirmed')`), true);
  assert.equal(await evaluate(`document.getElementById('install-banner').hidden`), true);
  assert.equal(await evaluate(`document.getElementById('install-webclip').hidden`), true);
  assert.equal(profileRequests.length, 1, 'Recognition must not download another profile');
  const installedScreenshot = await command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
  await writeFile(resolve(root, '../artifacts/webclip-installed.png'), Buffer.from(installedScreenshot.data, 'base64'));
  await evaluate(`document.getElementById('language-select').value='en';document.getElementById('language-select').dispatchEvent(new Event('change'))`);
  assert.equal(await evaluate(`document.getElementById('install-state').textContent`), 'Opened from the Home Screen');
  await evaluate(`document.getElementById('show-install-options').click()`);
  assert.equal(await evaluate(`document.getElementById('install-webclip').hidden`), false);
  assert.equal(await evaluate(`document.getElementById('webclip-instructions').open`), true);
  assert.equal(profileRequests.length, 1, 'Showing repair instructions must not download anything');
  await command('Page.removeScriptToEvaluateOnNewDocument', { identifier: standalone.identifier });
  assert.deepEqual(browser.exceptions, []);
  console.log('WebClip UI OK: iPhone notice, explicit download, honest pending state/reload, standalone recognition without storage, reinstall, PT-BR/EN and no automatic downloads');
} finally {
  if (browser) await browser.command('Browser.close').catch(() => {});
  browser?.close();
  edge.kill();
  fixture.close();
}
