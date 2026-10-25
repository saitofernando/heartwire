"use strict";
let language = "pt-BR";
try { language = localStorage.getItem("heartwire.language") || language; } catch {}
if (language === "en") {
  document.documentElement.lang = "en";
  for (const [id, text] of Object.entries({
    subtitle: "Local audio, low latency", "offline-title": "PC unavailable",
    "offline-message": "Connect to the PC’s Wi-Fi and start HeartWire from the Windows tray.",
    "offline-address": "If the PC’s address changed, scan the new QR code.", retry: "Try again",
  })) document.getElementById(id).textContent = text;
}
