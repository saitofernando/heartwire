import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import vm from 'node:vm';

const source = await readFile(new URL('../sw.js', import.meta.url), 'utf8');
const manifest = JSON.parse(await readFile(new URL('../manifest.webmanifest', import.meta.url), 'utf8'));
const origin = 'https://192.168.1.20:8081';

function worker() {
  const listeners = new Map();
  const writes = [];
  const precached = [];
  const offline = new Response('PC indisponível');
  let online = true;
  const context = {
    URL,
    self: {
      location: { origin },
      addEventListener: (type, handler) => listeners.set(type, handler),
      skipWaiting: async () => {}, clients: { claim: async () => {} },
    },
    caches: {
      open: async () => ({
        addAll: async (assets) => precached.push(...assets),
        put: async (request) => writes.push(request.url),
      }),
      match: async (request) => request === '/offline.html' ? offline : undefined,
      keys: async () => [], delete: async () => true,
    },
    fetch: async () => {
      if (!online) throw new TypeError('PC unavailable');
      return new Response('public asset');
    },
  };
  vm.runInNewContext(source, context);
  return {
    listeners, writes, precached,
    offline: () => { online = false; },
    request(path, { method = 'GET', mode = 'cors' } = {}) {
      let response;
      listeners.get('fetch')({
        request: { url: new URL(path, origin).href, method, mode },
        respondWith: (promise) => { response = promise; },
      });
      return response;
    },
  };
}

test('Home Screen app has a stable identity, no session URL, and valid PNG icons', async () => {
  assert.equal(manifest.name, 'HeartWire');
  assert.equal(manifest.start_url, '/');
  assert.equal(manifest.id, '/');
  assert.equal(manifest.scope, '/');
  assert.equal(manifest.display, 'standalone');
  for (const icon of manifest.icons) {
    const png = await readFile(new URL('..' + icon.src, import.meta.url));
    assert.equal(png.subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
    assert.equal(`${png.readUInt32BE(16)}x${png.readUInt32BE(20)}`, icon.sizes);
  }
});

test('worker never intercepts signaling, device status, private URLs, or other origins', () => {
  const sw = worker();
  for (const path of ['/status', '/offer', '/mic-offer', '/install/info', '/install/heartwire.mobileconfig', '/install/heartwire.mobileconfig?lang=en', '/preferences.json', '/app.js?token=private', 'https://example.org/app.js']) {
    assert.equal(sw.request(path), undefined, path);
  }
  assert.equal(sw.request('/offer', { method: 'POST' }), undefined);
  assert.equal(sw.request('/app.js', { method: 'POST' }), undefined);
  assert.deepEqual(sw.writes, []);
});

test('unreachable PC produces an offline page rather than a cached connected app', async () => {
  const sw = worker();
  sw.offline();
  const response = await sw.request('/?mode=mic', { mode: 'navigate' });
  assert.equal(await response.text(), 'PC indisponível');
  assert.deepEqual(sw.writes, []);
});

test('installation caches public files only, then refreshes static files from the network', async () => {
  const sw = worker();
  let installed;
  sw.listeners.get('install')({ waitUntil: (promise) => { installed = promise; } });
  await installed;
  assert.ok(sw.precached.includes('/offline.html'));
  assert.ok(sw.precached.includes('/media-session.js'));
  assert.ok(!sw.precached.includes('/'));
  assert.ok(sw.precached.every((path) => !path.includes('?') && !['/status', '/offer', '/mic-offer'].includes(path)));
  const response = await sw.request('/app.js');
  assert.equal(await response.text(), 'public asset');
  assert.deepEqual(sw.writes, [`${origin}/app.js`]);
});
