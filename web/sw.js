"use strict";

// Only the public shell belongs in this cache. Audio, SDP, peer counts, status,
// query strings and microphone permissions must always remain live.
const CACHE = "heartwire-shell-0.1.0";
const ASSETS = [
  "/offline.html", "/offline.js", "/styles.css", "/app.js", "/media-session.js", "/manifest.webmanifest",
  "/icons/heartwire.svg", "/icons/icon-192.png", "/icons/icon-512.png",
  "/icons/apple-touch-icon.png", "/icons/maskable-512.png",
];
const STATIC = new Set(ASSETS);

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(ASSETS)).then(() => self.skipWaiting()));
});
self.addEventListener("activate", (event) => {
  event.waitUntil(caches.keys().then((keys) => Promise.all(
    keys.filter((key) => key.startsWith("heartwire-shell-") && key !== CACHE).map((key) => caches.delete(key)),
  )).then(() => self.clients.claim()));
});
self.addEventListener("fetch", (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== "GET" || url.origin !== self.location.origin) return;
  // No cached online page: an unreachable PC must never look connected.
  if (request.mode === "navigate" && url.pathname === "/") {
    event.respondWith(fetch(request).catch(() => caches.match("/offline.html")));
    return;
  }
  if (!STATIC.has(url.pathname) || url.search) return;
  event.respondWith(fetch(request).then(async (response) => {
    if (response.ok && response.type !== "opaque") {
      // Storage limits/private browsing must not break a live connection.
      try {
        const cache = await caches.open(CACHE);
        await cache.put(request, response.clone());
      } catch {}
    }
    return response;
  }).catch(() => caches.match(request)));
});
