import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
import assert from 'node:assert/strict';
import { connectCdp } from './cdp.mjs';

// Miniapp only: the Windows panel is native Rust and has its own cargo tests.
const root = process.cwd();
await mkdir('artifacts', { recursive: true });
const server = createServer(async (req, res) => {
  try {
    const path = new URL(req.url, 'http://localhost').pathname;
    if (path === '/status') { res.setHeader('content-type', 'application/json'); res.end(JSON.stringify({running:true,mode:'mic',connectedPeers:0,urls:['https://192.168.1.10:8080'],port:8080,lastError:null})); return; }
    const assets = new Set(['/app.js', '/media-session.js', '/styles.css', '/manifest.webmanifest', '/sw.js']);
    const file = path === '/' ? resolve(root, 'web/index.html') : assets.has(path) || path.startsWith('/icons/') ? resolve(root, 'web', '.' + path) : null;
    if (!file) throw new Error('not found');
    const content = await readFile(file);
    res.setHeader('content-type', ({'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml'})[extname(file)] || 'application/octet-stream');
    res.end(content);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(r => server.listen(1432, '127.0.0.1', r));
const edge = spawn('C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe', ['--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--remote-debugging-port=9337', `--user-data-dir=${resolve(root,'artifacts/edge-ui-smoke')}`, 'about:blank'], { windowsHide:true, stdio:'ignore' });
let browser;
try {
  browser = await connectCdp(9337);
  const { command: cdp, evaluate, waitFor: wait, exceptions } = browser;
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true});
  const mobileInjection=await cdp('Page.addScriptToEvaluateOnNewDocument',{source:`
    window.__stoppedTracks=0;
    window.RTCPeerConnection=class extends EventTarget {constructor(){super();this.connectionState='new';this.iceGatheringState='complete';} close(){this.connectionState='closed';this.dispatchEvent(new Event('connectionstatechange'));}};
    navigator.mediaDevices.getUserMedia=()=>new Promise(resolve=>window.__grantMic=()=>resolve({getTracks:()=>[{stop:()=>window.__stoppedTracks++}],getAudioTracks:()=>[]}));
  `});
  await cdp('Page.navigate',{url:'http://127.0.0.1:1432/?mode=listen'});
  await wait(`document.getElementById('connect') && !document.getElementById('connect').disabled`);
  assert.equal(await evaluate(`document.getElementById('mic').getAttribute('aria-pressed')`),'true');
  assert.equal(await evaluate(`document.getElementById('listen').disabled`),true);
  assert.equal(await evaluate(`document.getElementById('help-content').textContent`),'','Help prose is absent from the initial document');
  await evaluate(`document.getElementById('mobile-tab-help').click()`);
  await wait(`document.getElementById('help-content').textContent.includes('Tela Bloqueada')`);
  assert.doesNotMatch(await evaluate(`document.getElementById('help-content').textContent`),/Leitor|Reader/);
  await evaluate(`document.getElementById('mobile-tab-connection').click()`);
  await wait(`document.getElementById('help-content').childElementCount===0`);
  const mobile=await cdp('Page.captureScreenshot',{format:'png',captureBeyondViewport:true});await writeFile('artifacts/mobile-mic.png',Buffer.from(mobile.data,'base64'));
  await evaluate(`document.getElementById('connect').click()`);
  await wait(`typeof window.__grantMic === 'function' && document.getElementById('connect').textContent.includes('Cancelar')`);
  await evaluate(`document.getElementById('connect').click();window.__grantMic()`);
  await wait(`window.__stoppedTracks===1`);
  assert.match(await evaluate(`document.getElementById('connect').textContent`),/Conectar/);
  assert.deepEqual(exceptions,[],'Miniapp has no uncaught browser exceptions');
  await cdp('Page.removeScriptToEvaluateOnNewDocument',{identifier:mobileInjection.identifier});
  console.log('PASS miniapp: server mode, incompatible mode lock, current help, mic pending permission cancellation stops late stream');
  console.log('PASS browser: zero uncaught exceptions, screenshot artifacts/mobile-mic.png');
  await cdp('Browser.close').catch(() => {});
} finally {
  if (browser) await browser.command('Browser.close').catch(() => {});
  browser?.close();
  edge.kill();
  server.close();
}
