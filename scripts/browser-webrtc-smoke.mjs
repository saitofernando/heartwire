import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { connectCdp } from './cdp.mjs';

// Exercise the real mobile page against the packaged app, including the HTTP/2
// Origin/:authority request that previously failed on iPhone.
export async function browserWebrtcSmoke(url, onConnected = async () => {}) {
  const edge = spawn('C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe', [
    '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
    '--ignore-certificate-errors', '--remote-debugging-port=9339',
    `--user-data-dir=${resolve('artifacts/edge-webrtc-smoke')}`, 'about:blank',
  ], { windowsHide: true, stdio: 'ignore' });
  let client;
  try {
    client = await connectCdp(9339, { timeout: 20000 });
    await client.command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
    // Keep the real browser implementation, and retain the registered callbacks
    // to exercise remote controls without claiming a physical iOS lock-screen test.
    await client.command('Page.addScriptToEvaluateOnNewDocument', { source: `
      window.__mediaActions = {};
      if (navigator.mediaSession) {
        const register = navigator.mediaSession.setActionHandler.bind(navigator.mediaSession);
        navigator.mediaSession.setActionHandler = (action, handler) => {
          register(action, handler);
          window.__mediaActions[action] = handler;
        };
      }
    ` });
    await client.command('Page.navigate', { url });
    await client.waitFor(`document.getElementById('connect') && !document.getElementById('connect').disabled`);
    await client.evaluate(`document.getElementById('connect').click()`);
    await client.waitFor(`typeof current !== 'undefined' && current?.pc.connectionState === 'connected'`, 20000);
    const protocol = await client.evaluate(`performance.getEntriesByType('resource').find(entry => new URL(entry.name).pathname === '/offer')?.nextHopProtocol`);
    assert.equal(protocol, 'h2', 'Browser signaling must use HTTP/2');
    await client.waitFor(`current.pc.getStats().then(report => Array.from(report.values()).some(entry => entry.type === 'inbound-rtp' && entry.kind === 'audio' && entry.packetsReceived >= 50))`, 10000);
    await client.waitFor(`navigator.mediaSession?.playbackState === 'playing' && navigator.mediaSession.metadata?.artist === 'HeartWire'`);
    const token = await client.evaluate('current.token');
    await client.evaluate(`window.__mediaActions.pause()`);
    await client.waitFor(`player.paused && navigator.mediaSession.playbackState === 'paused'`);
    assert.equal(await client.evaluate('current.token'), token, 'Pause preserves the WebRTC session');
    assert.equal(await client.evaluate('current.pc.connectionState'), 'connected');
    assert.equal(await client.evaluate(`document.getElementById('unblock-audio').hidden`), false, 'Paused playback can be resumed from the page');
    await client.evaluate(`
      const originalPlay = player.play.bind(player);
      player.play = () => {
        player.play = originalPlay;
        return Promise.reject(new DOMException('Autoplay denied in test', 'NotAllowedError'));
      };
      window.__mediaActions.play();
    `);
    await client.waitFor(`blockedAudio && !document.getElementById('unblock-audio').hidden`);
    await client.evaluate(`window.__mediaActions.play()`);
    await client.waitFor(`!player.paused && navigator.mediaSession.playbackState === 'playing'`);
    assert.equal(await client.evaluate(`document.getElementById('unblock-audio').hidden`), true, 'Successful remote playback clears the autoplay warning');
    await client.evaluate(`document.getElementById('language-select').value='en';document.getElementById('language-select').dispatchEvent(new Event('change'))`);
    await client.waitFor(`navigator.mediaSession.metadata?.title === 'PC audio'`);
    assert.equal(await client.evaluate('current.token'), token, 'Language changes preserve the stream');
    await client.evaluate(`document.getElementById('language-select').value='pt-BR';document.getElementById('language-select').dispatchEvent(new Event('change'))`);
    await client.waitFor(`navigator.mediaSession.metadata?.title === 'Áudio do PC'`);
    await mkdir('artifacts', { recursive: true });
    const screenshot = await client.command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
    await writeFile('artifacts/mobile-connected.png', Buffer.from(screenshot.data, 'base64'));
    await onConnected();
    assert.deepEqual(client.exceptions, []);
    await client.evaluate(`window.__savedRemotePlay = window.__mediaActions.play; window.__mediaActions.stop()`);
    await client.waitFor(`current === null && navigator.mediaSession.playbackState === 'none' && navigator.mediaSession.metadata === null`);
    await client.evaluate(`window.__savedRemotePlay()`);
    assert.equal(await client.evaluate('current'), null, 'A stale remote play action cannot reconnect');
    assert.equal(await client.evaluate('player.srcObject'), null);
    console.log('Miniapp real: HTTP/2, WebRTC, áudio, Media Session, pausa/retomada na mesma sessão, idiomas e parada OK (sem simular bloqueio físico do iOS)');
  } finally {
    if (client) await client.command('Browser.close').catch(() => {});
    client?.close();
    edge.kill();
  }
}
