"use strict";
// Portuguese is the default. The language choice is local to this browser;
// installing the Home Screen app does not send settings to a remote service.
const translations = {
  "Áudio local, baixa latência": "Local audio, low latency",
  "Conexão": "Connection", "Ajustes": "Settings", "Ajuda": "Help", "Idioma": "Language",
  "Seções": "Sections", "Modo de áudio": "Audio mode", "Fone": "Headphones", "Microfone": "Microphone",
  "Tela de Início": "Home Screen", "Instalar HeartWire": "Install HeartWire",
  "No Safari, toque em Compartilhar e Adicionar à Tela de Início.": "In Safari, tap Share, then Add to Home Screen.",
  "Instalar com ícone": "Install with icon", "Como instalar": "How to install",
  "Adicionar à Tela de Início": "Add to Home Screen",
  "Abra o HeartWire pelo ícone, sem as barras do Safari.": "Open HeartWire from its icon, without the Safari bars.",
  "Baixa um perfil com o ícone. Você conclui a instalação nos Ajustes do iPhone.": "Downloads a profile with the icon. Finish installation in iPhone Settings.",
  "Concluir instalação": "Finish installation",
  "Abra Ajustes → Perfil Baixado e instale. Depois, abra pelo ícone.": "Open Settings → Profile Downloaded and install. Then open the icon.",
  "Ver passos": "View steps",
  "Instalação solicitada. Conclua nos Ajustes e abra pelo ícone.": "Installation requested. Finish in Settings, then open the icon.",
  "O download do perfil não conclui a instalação.": "Downloading the profile does not finish installation.",
  "Aberto pela Tela de Início": "Opened from the Home Screen",
  "Você está usando o HeartWire em tela cheia.": "You are using HeartWire in its own full-screen window.",
  "HeartWire instalado": "HeartWire installed",
  "Abra o HeartWire pelo ícone para usar a janela do app.": "Open HeartWire from its icon to use the app window.",
  "Reinstalar ou corrigir o ícone": "Reinstall or fix the icon",
  "Baixar perfil novamente": "Download profile again",
  "No Safari, toque em Permitir para baixar o perfil.": "In Safari, tap Allow to download the profile.",
  "Logo depois, abra Ajustes → Perfil Baixado → Instalar.": "Then open Settings → Profile Downloaded → Install.",
  "Volte à Tela de Início e abra o novo ícone HeartWire para confirmar.": "Return to the Home Screen and open the new HeartWire icon to confirm.",
  "O perfil aparece em Geral → VPN e Gerenciamento de Dispositivo. Contém apenas o atalho e o ícone; não configura VPN, certificados ou gerenciamento remoto.": "The profile appears in General → VPN & Device Management. It only contains the shortcut and icon; it does not configure VPN, certificates, or remote management.",
  "O iPhone pode mostrar que o perfil não é assinado. Instale em até 8 minutos após baixar; se ele sumir, baixe novamente.": "iPhone may show that the profile is unsigned. Install within 8 minutes of downloading; if it disappears, download it again.",
  "A confirmação aparece ao abrir pelo ícone. O Safari não consegue consultar os perfis instalados.": "Confirmation appears when you open the icon. Safari cannot check installed profiles.",
  "Remova o atalho antigo só depois que o novo funcionar.": "Remove the old shortcut only after the new one works.",
  "O PC precisa estar ligado e com o HeartWire iniciado.": "The PC must be on and HeartWire must be running.",
  "Código aberto · MIT": "Open source · MIT",
  "Com a tela bloqueada, o iOS pode interromper o áudio.": "iOS may interrupt audio while the screen is locked.",
  "Liberar som no Safari": "Enable sound in Safari",
  "Retomar áudio": "Resume audio", "Áudio pausado": "Audio paused",
  "Troque o modo no aplicativo do PC.": "Change the mode in the PC app.",
  "Áudio do PC": "PC audio", "Microfone do iPhone": "iPhone microphone",
  "Microfone ativo": "Microphone active", "Recebendo áudio": "Receiving audio",
  "Conectando…": "Connecting…", "Desconectado": "Disconnected", "Desconectado.": "Disconnected.",
  "Conectar": "Connect", "Desconectar": "Disconnect", "Cancelar": "Cancel",
  "Permita o microfone ao conectar.": "Allow microphone access when connecting.",
  "Toque em Conectar para ouvir o PC.": "Tap Connect to listen to the PC.",
  "Conexão cancelada": "Connection cancelled",
  "A busca de conexão demorou demais. Verifique o Wi-Fi e tente novamente.": "Connection discovery timed out. Check Wi-Fi and try again.",
  "O microfone precisa do endereço https:// e da confiança no certificado local. Abra o link HTTPS gerado no PC e verifique as permissões do Safari.": "The microphone needs HTTPS and a trusted local certificate. Open the HTTPS link from the PC and check Safari permissions.",
  "Permita o microfone quando o Safari solicitar.": "Allow microphone access when Safari asks.",
  "Preparando o áudio do seu computador…": "Preparing PC audio…",
  "Não foi possível conectar.": "Could not connect.",
  "A conexão demorou demais. Confira a rede Wi-Fi e a permissão do Firewall do Windows.": "The connection timed out. Check Wi-Fi and the Windows Firewall permission.",
  "Microfone enviado para o PC.": "Sending microphone audio to the PC.",
  "Reproduza som no PC e ajuste o volume do iPhone.": "Play audio on the PC and adjust the iPhone volume.",
  "A conexão caiu. Toque em Conectar para tentar novamente.": "The connection dropped. Tap Connect to try again.",
  "O Wi-Fi oscilou. Tentando recuperar a conexão…": "Wi-Fi was interrupted. Reconnecting…",
  "Conexão perdida. Confira o Wi-Fi e conecte novamente.": "Connection lost. Check Wi-Fi and connect again.",
  "Toque em Liberar som no Safari para permitir a reprodução.": "Tap Enable sound in Safari to allow playback.",
  "O microfone foi interrompido. Conecte novamente para continuar.": "The microphone stopped. Connect again to continue.",
  "O acesso ao microfone foi negado. Permita o microfone nos ajustes do Safari e tente novamente.": "Microphone access was denied. Allow it in Safari settings and try again.",
  "O navegador não encontrou um microfone disponível.": "The browser could not find an available microphone.",
  "Não foi possível falar com o PC. Confira o Wi-Fi e se a conexão continua iniciada no aplicativo.": "Could not reach the PC. Check Wi-Fi and that HeartWire is running.",
  "Não foi possível conectar. Tente novamente.": "Could not connect. Try again.",
  "Reproduza algo no PC e ajuste o volume aqui no iPhone.": "Play audio on the PC and adjust the iPhone volume.",
  "O Safari bloqueou o áudio. Confira a saída de som do iPhone ou desconecte e tente novamente.": "Safari blocked audio. Check the iPhone audio output or disconnect and try again.",
  "Este navegador não suporta WebRTC. Abra o link no Safari atualizado ou em outro navegador moderno.": "This browser does not support WebRTC. Open the link in an up-to-date Safari or another modern browser.",
  "Não foi possível ler o modo selecionado no PC.": "Could not read the PC’s audio mode.",
  "Não foi possível consultar o PC. Você pode tentar Conectar ou atualizar esta página.": "Could not reach the PC. Try Connect or reload this page.",
  "Sem rede. Reconecte o Wi-Fi para continuar.": "Offline. Reconnect to Wi-Fi to continue.",
  "Origem da conexão não permitida.": "Connection origin is not allowed.",
  "O servidor está parando.": "The server is stopping.",
  "Mude o modo no aplicativo do PC e reinicie a transmissão.": "Change the mode in the PC app and restart the connection.",
  "Era esperada uma oferta WebRTC.": "A WebRTC offer was expected.",
  "Um microfone já está conectado. Desconecte-o antes de usar outro.": "A microphone is already connected. Disconnect it before using another.",
  "Limite de oito aparelhos atingido.": "The limit of eight devices has been reached.",
  "<b>Rede:</b> PC e iPhone no mesmo Wi-Fi. Permita o HeartWire no Firewall do Windows.": "<b>Network:</b> Keep the PC and iPhone on the same Wi-Fi. Allow HeartWire through Windows Firewall.",
  "<b>Fone:</b> escolha AirPods ou fones na saída de áudio do iPhone e reproduza som no PC.": "<b>Headphones:</b> Select AirPods or headphones as the iPhone audio output and play audio on the PC.",
  "<b>Microfone:</b> escolha CABLE Input no HeartWire e CABLE Output como microfone no aplicativo do PC. Requer VB-CABLE.": "<b>Microphone:</b> Select CABLE Input in HeartWire and CABLE Output as the PC app’s microphone. Requires VB-CABLE.",
  "<b>Certificado local:</b> no Safari, Mostrar detalhes → visitar este site. O microfone requer HTTPS e sua permissão.": "<b>Local certificate:</b> In Safari, Show Details → visit this website. Microphone access requires HTTPS and your permission.",
  "<b>Tela de Início:</b> no Safari, Compartilhar → Adicionar à Tela de Início. O ícone abre o HeartWire sem as barras do navegador.": "<b>Home Screen:</b> In Safari, Share → Add to Home Screen. The icon opens HeartWire without the browser bars.",
  "<b>Tela Bloqueada:</b> no modo Fone, o HeartWire solicita controles de reprodução ao iOS. Pausar mantém a conexão; Parar desconecta. O iOS pode suspender o áudio.": "<b>Lock Screen:</b> In Headphones mode, HeartWire requests playback controls from iOS. Pause keeps the connection; Stop disconnects. iOS may suspend audio.",
  "<b>Cache offline:</b> depende de o navegador confiar no certificado HTTPS. Se ele não permitir, a conexão e o atalho continuam disponíveis quando o PC estiver acessível.": "<b>Offline cache:</b> Requires a certificate trusted by your browser. If unavailable, the live connection and Home Screen shortcut still work while the PC is reachable.",
  "<b>Modo:</b> pare a conexão no PC antes de trocar entre Fone e Microfone.": "<b>Mode:</b> Stop the connection on the PC before switching between Headphones and Microphone.",
};
let language = "pt-BR";
try { language = localStorage.getItem("heartwire.language") === "en" ? "en" : "pt-BR"; } catch {}
const languageHint = new URLSearchParams(location.search).get("lang");
if (languageHint === "en" || languageHint === "pt-BR") {
  language = languageHint;
  try { localStorage.setItem("heartwire.language", language); } catch {}
}
function t(text) {
  if (language !== "en") return text;
  return translations[text] || text.replace(/^Não foi possível negociar a conexão \(HTTP (\d+)\)\.$/, "Could not negotiate the connection (HTTP $1).");
}
const $ = (id) => document.getElementById(id);
const player = $("player");
const connect = $("connect");
const card = $("connection");
const buttons = { listen: $("listen"), mic: $("mic") };
let mode = new URLSearchParams(location.search).get("mode") === "mic" ? "mic" : "listen";
let serverMode = null;
let current = null;
let lastToken = 0;
let ready = false;
let blockedAudio = false;
const supported = typeof RTCPeerConnection !== "undefined";
const micSupported = window.isSecureContext && !!navigator.mediaDevices?.getUserMedia;
const playIcon = '<svg class="icon" viewBox="0 0 24 24" fill="currentColor"><path d="m8 4 12 8-12 8z"/></svg>';
const stopIcon = '<svg class="icon" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="2"/></svg>';
let statusText = $("status").textContent;
let errorText = "";
let selectedTab = "connection";
let installPrompt = null;
const standaloneDisplay = matchMedia("(display-mode: standalone)");
const fullscreenDisplay = matchMedia("(display-mode: fullscreen)");
const isIos = /iPad|iPhone|iPod/.test(navigator.userAgent) || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1);
const installRequestKey = "heartwire.install.requestedAt";
let installRequestedAt = 0;
let installRequestStored = true;
let browserInstallConfirmed = false;
let showInstallOptions = false;
let mediaPlaybackState = "none";
const mediaControls = window.HeartWireMediaSession?.create({
  player,
  canControl: () => mode === "listen" && current && active(current) && current.pc.connectionState === "connected",
  getTitle: () => t("Áudio do PC"),
  onStop: () => { if (current && mode === "listen") stop(current); },
  onPlayError: () => {
    if (!current || mode !== "listen") return;
    blockedAudio = true;
    status("Toque em Liberar som no Safari para permitir a reprodução.");
    render();
  },
  onPlaybackChange: (state) => {
    mediaPlaybackState = state;
    if (state === "playing" && blockedAudio) {
      blockedAudio = false;
      status("Reproduza som no PC e ajuste o volume do iPhone.");
    }
    render();
  },
});
function showError(error) { errorText = error instanceof Error ? error.message : String(error); $("error").textContent = t(errorText); $("error").hidden = false; }
function status(text) { statusText = text; $("status").textContent = t(text); }
function active(session) { return current === session && session.token === lastToken; }
function render() {
  mediaControls?.sync();
  const connected = current?.pc.connectionState === "connected";
  const paused = connected && mode === "listen" && mediaPlaybackState === "paused";
  card.classList.toggle("connected", !!connected);
  card.classList.toggle("connecting", !!current && !connected);
  for (const name of ["listen", "mic"]) {
    buttons[name].setAttribute("aria-pressed", String(mode === name));
    buttons[name].disabled = !!current || (!!serverMode && serverMode !== name);
    buttons[name].title = serverMode && serverMode !== name ? t("Troque o modo no aplicativo do PC.") : "";
  }
  $("headline").textContent = t(mode === "mic" ? "Microfone do iPhone" : "Áudio do PC");
  const title = t(connected ? (mode === "mic" ? "Microfone ativo" : paused ? "Áudio pausado" : "Recebendo áudio") : current ? "Conectando…" : "Desconectado");
  if ($("connection-title").textContent !== title) $("connection-title").textContent = title;
  connect.innerHTML = `${current ? stopIcon : playIcon}<span>${t(connected ? "Desconectar" : current ? "Cancelar" : "Conectar")}</span>`;
  $("unblock-audio").textContent = t(blockedAudio ? "Liberar som no Safari" : "Retomar áudio");
  $("unblock-audio").hidden = !current || mode !== "listen" || (!blockedAudio && !paused);
  connect.disabled = !ready || !supported;
}
function selectMode(next) {
  if (current || (serverMode && serverMode !== next)) return;
  mode = next;
  const url = new URL(location.href); url.searchParams.set("mode", mode); history.replaceState(null, "", url);
  status(mode === "mic" ? "Permita o microfone ao conectar." : "Toque em Conectar para ouvir o PC."); render();
}
buttons.listen.addEventListener("click", () => selectMode("listen"));
buttons.mic.addEventListener("click", () => selectMode("mic"));

// Keep help content scoped to its tab.
function selectMobileTab(tab) {
  selectedTab = tab;
  for (const name of ["connection", "settings", "help"]) {
    const selected = tab === name;
    const id = `mobile-tab-${name}`;
    $(name === "connection" ? "mobile-connection" : name).hidden = !selected;
    $(id).setAttribute("aria-selected", String(selected));
    $(id).tabIndex = selected ? 0 : -1;
  }
  const content = $("help-content");
  content.replaceChildren();
  if (tab !== "help") return;
  const paragraphs = [
    "<b>Rede:</b> PC e iPhone no mesmo Wi-Fi. Permita o HeartWire no Firewall do Windows.",
    "<b>Fone:</b> escolha AirPods ou fones na saída de áudio do iPhone e reproduza som no PC.",
    "<b>Microfone:</b> escolha CABLE Input no HeartWire e CABLE Output como microfone no aplicativo do PC. Requer VB-CABLE.",
    "<b>Certificado local:</b> no Safari, Mostrar detalhes → visitar este site. O microfone requer HTTPS e sua permissão.",
    "<b>Tela de Início:</b> no Safari, Compartilhar → Adicionar à Tela de Início. O ícone abre o HeartWire sem as barras do navegador.",
    "<b>Tela Bloqueada:</b> no modo Fone, o HeartWire solicita controles de reprodução ao iOS. Pausar mantém a conexão; Parar desconecta. O iOS pode suspender o áudio.",
    "<b>Cache offline:</b> depende de o navegador confiar no certificado HTTPS. Se ele não permitir, a conexão e o atalho continuam disponíveis quando o PC estiver acessível.",
    "<b>Modo:</b> pare a conexão no PC antes de trocar entre Fone e Microfone.",
  ];
  for (const text of paragraphs) {
    const paragraph = document.createElement("p");
    paragraph.innerHTML = t(text);
    content.append(paragraph);
  }
}
for (const name of ["connection", "settings", "help"]) {
  const id = `mobile-tab-${name}`;
  $(id).addEventListener("click", () => selectMobileTab(name));
  $(id).addEventListener("keydown", (event) => {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const tabs = ["connection", "settings", "help"];
    const index = tabs.indexOf(name);
    const next = event.key === "Home" ? 0 : event.key === "End" ? 2 : (index + (event.key === "ArrowRight" ? 1 : -1) + 3) % 3;
    selectMobileTab(tabs[next]);
    $(`mobile-tab-${tabs[next]}`).focus();
  });
}

function applyLanguage() {
  document.documentElement.lang = language;
  document.querySelectorAll("[data-i18n]").forEach((element) => { element.textContent = t(element.dataset.i18n); });
  document.querySelector("nav").setAttribute("aria-label", t("Seções"));
  document.querySelector(".modes").setAttribute("aria-label", t("Modo de áudio"));
  $("language-select").value = language;
  $("install-webclip").setAttribute("href", `/install/heartwire.mobileconfig?lang=${encodeURIComponent(language)}`);
  $("status").textContent = t(statusText);
  $("error").textContent = t(errorText);
  renderInstall();
  selectMobileTab(selectedTab);
  render();
}
$("language-select").addEventListener("change", () => {
  language = $("language-select").value === "en" ? "en" : "pt-BR";
  try { localStorage.setItem("heartwire.language", language); } catch {}
  const url = new URL(location.href); url.searchParams.delete("lang"); history.replaceState(null, "", url);
  applyLanguage();
});
// A web page cannot inspect iOS profiles. A requested download stays pending;
// only the current launch mode or a browser appinstalled event confirms use.
// Safari and the Home Screen app can have separate storage containers.
function renderInstall() {
  if (installRequestStored) {
    try { installRequestedAt = Number(localStorage.getItem(installRequestKey)) || 0; } catch { /* Keep the in-memory request if storage is unavailable. */ }
  }
  const standalone = navigator.standalone === true || standaloneDisplay.matches || fullscreenDisplay.matches;
  const confirmed = standalone || browserInstallConfirmed;
  if (confirmed) {
    installRequestedAt = 0;
    try { localStorage.removeItem(installRequestKey); } catch {}
  }
  const pending = installRequestedAt > 0;
  $("install-banner").hidden = !isIos || confirmed;
  $("install-banner-title").textContent = t(pending ? "Concluir instalação" : "Adicionar à Tela de Início");
  $("install-banner-note").textContent = t(pending ? "Abra Ajustes → Perfil Baixado e instale. Depois, abra pelo ícone." : "Abra o HeartWire pelo ícone, sem as barras do Safari.");
  $("show-install-guide").textContent = t(pending ? "Ver passos" : "Como instalar");
  $("install-note").textContent = t(standalone ? "Você está usando o HeartWire em tela cheia." : browserInstallConfirmed ? "Abra o HeartWire pelo ícone para usar a janela do app." : pending ? "O download do perfil não conclui a instalação." : "No Safari, toque em Compartilhar e Adicionar à Tela de Início.");
  const state = $("install-state");
  state.hidden = !confirmed && !pending;
  state.classList.toggle("confirmed", confirmed);
  const message = confirmed ? standalone ? "Aberto pela Tela de Início" : "HeartWire instalado" : pending ? "Instalação solicitada. Conclua nos Ajustes e abra pelo ícone." : "";
  if (state.textContent !== t(message)) state.textContent = t(message);
  $("install").hidden = !installPrompt || confirmed;
  $("show-install-options").hidden = !confirmed || showInstallOptions;
  $("webclip-disclaimer").hidden = confirmed && !showInstallOptions;
  $("install-webclip").hidden = confirmed && !showInstallOptions;
  $("install-webclip").textContent = t(pending || showInstallOptions ? "Baixar perfil novamente" : "Instalar com ícone");
  $("install-confirmation-note").hidden = confirmed;
  if (pending || showInstallOptions) $("webclip-instructions").open = true;
}
$("show-install-guide").addEventListener("click", () => {
  selectMobileTab("settings");
  $("webclip-instructions").open = true;
  $("install-title").focus({ preventScroll: true });
  $("install-title").scrollIntoView({ block: "start" });
});
$("show-install-options").addEventListener("click", () => { showInstallOptions = true; renderInstall(); });
$("install-webclip").addEventListener("click", () => {
  installRequestedAt = Date.now();
  try { localStorage.setItem(installRequestKey, String(installRequestedAt)); installRequestStored = true; } catch { installRequestStored = false; }
  $("webclip-instructions").open = true;
  renderInstall();
});
for (const query of [standaloneDisplay, fullscreenDisplay]) {
  if (query.addEventListener) query.addEventListener("change", renderInstall);
  else if (query.addListener) query.addListener(renderInstall);
}
window.addEventListener("pageshow", renderInstall);
window.addEventListener("storage", (event) => { if (event.key === installRequestKey || event.key === null) renderInstall(); });
document.addEventListener("visibilitychange", () => { if (document.visibilityState === "visible") renderInstall(); });
window.addEventListener("beforeinstallprompt", (event) => {
  event.preventDefault(); installPrompt = event; renderInstall();
});
$("install").addEventListener("click", async () => {
  if (!installPrompt) return;
  const prompt = installPrompt; installPrompt = null; $("install").hidden = true;
  try { await prompt.prompt(); await prompt.userChoice; } catch { /* Safari uses Share → Add to Home Screen instead. */ }
});
window.addEventListener("appinstalled", () => {
  installPrompt = null; browserInstallConfirmed = true; renderInstall();
});

function stop(session, message = "Desconectado.") {
  if (!session) return;
  const wasCurrent = current === session;
  if (wasCurrent) { current = null; lastToken++; blockedAudio = false; }
  if (wasCurrent) mediaControls?.clear();
  session.abort.abort();
  clearInterval(session.statsTimer); clearTimeout(session.connectTimer); clearTimeout(session.disconnectTimer);
  session.stream?.getTracks().forEach((track) => track.stop());
  session.stream = null;
  if (session.wakeLock) void session.wakeLock.release().catch(() => {});
  session.wakeLock = null;
  session.pc.close();
  if (wasCurrent) {
    player.pause(); player.srcObject = null;
    $("stats").textContent = ""; status(message); render();
  }
}
function waitForIce(pc, signal) {
  if (pc.iceGatheringState === "complete") return Promise.resolve();
  return new Promise((resolve, reject) => {
    const cleanup = () => { clearTimeout(timer); pc.removeEventListener("icegatheringstatechange", check); signal.removeEventListener("abort", abort); };
    const check = () => { if (pc.iceGatheringState === "complete") { cleanup(); resolve(); } };
    const abort = () => { cleanup(); reject(new DOMException("Conexão cancelada", "AbortError")); };
    const timer = setTimeout(() => { cleanup(); reject(new Error("A busca de conexão demorou demais. Verifique o Wi-Fi e tente novamente.")); }, 12000);
    pc.addEventListener("icegatheringstatechange", check);
    signal.addEventListener("abort", abort, { once: true });
    if (signal.aborted) abort(); else check();
  });
}
async function requestWakeLock(session) {
  if (!active(session) || document.visibilityState !== "visible" || !navigator.wakeLock || session.wakeLock || session.wakePending) return;
  session.wakePending = true;
  try {
    const lock = await navigator.wakeLock.request("screen");
    if (!active(session)) { await lock.release(); return; }
    session.wakeLock = lock;
    lock.addEventListener("release", () => { if (session.wakeLock === lock) session.wakeLock = null; });
  } catch { /* Safari may not permit wake lock; the visible screen reminder remains. */ }
  finally { session.wakePending = false; }
}
function startStats(session) {
  let previous = null;
  let pending = false;
  session.statsTimer = setInterval(async () => {
    if (!active(session) || pending || session.pc.connectionState !== "connected") return;
    pending = true;
    try {
      const report = await session.pc.getStats();
      if (!active(session)) return;
      const type = mode === "mic" ? "outbound-rtp" : "inbound-rtp";
      report.forEach((entry) => {
        if (entry.type !== type || (entry.kind || entry.mediaType) !== "audio") return;
        if (previous) {
          const seconds = (entry.timestamp - previous.timestamp) / 1000;
          if (seconds <= 0) return;
          if (mode === "mic") {
            const rate = Math.round((entry.packetsSent - previous.packetsSent) / seconds);
            $("stats").textContent = language === "en" ? `${rate} packets/s · microphone active` : `${rate} pacotes/s · microfone ativo`;
          } else if (entry.jitterBufferDelay !== undefined && previous.jitterBufferDelay !== undefined) {
            const count = entry.jitterBufferEmittedCount - previous.jitterBufferEmittedCount;
            const delay = entry.jitterBufferDelay - previous.jitterBufferDelay;
            const lost = Math.max(0, Math.round(((entry.packetsLost || 0) - (previous.packetsLost || 0)) / seconds));
            if (count > 0) $("stats").textContent = language === "en" ? `buffer ${Math.round(delay / count * 1000)} ms · ${lost} lost/s` : `buffer ${Math.round(delay / count * 1000)} ms · ${lost} perdas/s`;
          }
        }
        previous = entry;
      });
    } catch { /* Stats are optional diagnostics and must not interrupt audio. */ }
    finally { pending = false; }
  }, 1000);
}
async function start() {
  $("error").hidden = true;
  if (mode === "mic" && !micSupported) { showError("O microfone precisa do endereço https:// e da confiança no certificado local. Abra o link HTTPS gerado no PC e verifique as permissões do Safari."); return; }
  const session = { token: ++lastToken, pc: new RTCPeerConnection({ iceServers: [] }), stream: null, abort: new AbortController(), wakeLock: null, wakePending: false, statsTimer: null, connectTimer: null, disconnectTimer: null };
  current = session; blockedAudio = false;
  status(mode === "mic" ? "Permita o microfone quando o Safari solicitar." : "Preparando o áudio do seu computador…");
  render();
  // iOS: call play() synchronously from the button gesture before signaling.
  if (mode === "listen") void player.play().catch(() => {});
  session.connectTimer = setTimeout(() => { if (active(session) && session.pc.connectionState !== "connected") { stop(session, "Não foi possível conectar."); showError("A conexão demorou demais. Confira a rede Wi-Fi e a permissão do Firewall do Windows."); } }, 25000);
  session.pc.addEventListener("connectionstatechange", () => {
    if (!active(session)) return;
    const state = session.pc.connectionState;
    if (state === "connected") {
      clearTimeout(session.connectTimer); clearTimeout(session.disconnectTimer);
      status(mode === "mic" ? "Microfone enviado para o PC." : "Reproduza som no PC e ajuste o volume do iPhone.");
      if (!session.statsTimer) startStats(session);
      void requestWakeLock(session); render();
    } else if (state === "failed" || state === "closed") {
      stop(session, "A conexão caiu. Toque em Conectar para tentar novamente.");
    } else if (state === "disconnected") {
      status("O Wi-Fi oscilou. Tentando recuperar a conexão…");
      render();
      clearTimeout(session.disconnectTimer);
      session.disconnectTimer = setTimeout(() => { if (active(session) && session.pc.connectionState === "disconnected") stop(session, "Conexão perdida. Confira o Wi-Fi e conecte novamente."); }, 5000);
    }
  });
  try {
    if (mode === "listen") {
      const transceiver = session.pc.addTransceiver("audio", { direction: "recvonly" });
      // Keep Safari's playout buffer at its lowest supported target.
      try { if ("playoutDelayHint" in transceiver.receiver) transceiver.receiver.playoutDelayHint = 0; } catch {}
      try { if ("jitterBufferTarget" in transceiver.receiver) transceiver.receiver.jitterBufferTarget = 0; } catch {}
      session.pc.addEventListener("track", (event) => {
        if (!active(session)) return;
        player.srcObject = event.streams[0] || new MediaStream([event.track]);
        void player.play().catch(() => { if (!active(session)) return; blockedAudio = true; status("Toque em Liberar som no Safari para permitir a reprodução."); render(); });
      });
    } else {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: false, noiseSuppression: true, autoGainControl: true }, video: false });
      if (!active(session)) { stream.getTracks().forEach((track) => track.stop()); return; }
      session.stream = stream;
      for (const track of stream.getAudioTracks()) {
        session.pc.addTransceiver(track, { direction: "sendonly" });
        track.addEventListener("ended", () => { if (active(session)) stop(session, "O microfone foi interrompido. Conecte novamente para continuar."); });
      }
    }
    if (!active(session)) return;
    await session.pc.setLocalDescription(await session.pc.createOffer());
    await waitForIce(session.pc, session.abort.signal);
    if (!active(session)) return;
    const response = await fetch(mode === "mic" ? "/mic-offer" : "/offer", {
      method: "POST", headers: { "Content-Type": "application/json" }, signal: session.abort.signal,
      body: JSON.stringify({ type: session.pc.localDescription.type, sdp: session.pc.localDescription.sdp }),
    });
    if (!response.ok) {
      let message = `Não foi possível negociar a conexão (HTTP ${response.status}).`;
      try { const body = await response.json(); if (body.error) message = body.error; } catch {}
      throw new Error(message);
    }
    const answer = await response.json();
    if (!active(session)) return;
    await session.pc.setRemoteDescription(answer);
    if (active(session)) void requestWakeLock(session);
  } catch (error) {
    if (!active(session)) return;
    const message = error.name === "NotAllowedError" ? "O acesso ao microfone foi negado. Permita o microfone nos ajustes do Safari e tente novamente." : error.name === "NotFoundError" ? "O navegador não encontrou um microfone disponível." : error instanceof TypeError ? "Não foi possível falar com o PC. Confira o Wi-Fi e se a conexão continua iniciada no aplicativo." : error.message || String(error);
    stop(session, "Não foi possível conectar. Tente novamente."); showError(message);
  }
}
connect.addEventListener("click", () => {
  if (current) stop(current);
  else void start();
});
$("unblock-audio").addEventListener("click", () => {
  if (!current) return;
  const session = current;
  void player.play().then(() => { if (!active(session)) return; blockedAudio = false; status("Reproduza algo no PC e ajuste o volume aqui no iPhone."); render(); }).catch(() => showError("O Safari bloqueou o áudio. Confira a saída de som do iPhone ou desconecte e tente novamente."));
});
document.addEventListener("visibilitychange", () => {
  if (document.visibilityState !== "visible") return;
  if (current) void requestWakeLock(current); else void initialize();
});
window.addEventListener("pagehide", () => { if (current) stop(current); });
window.addEventListener("offline", () => {
  ready = false;
  if (current) stop(current, "Sem rede. Reconecte o Wi-Fi para continuar.");
  else { status("Sem rede. Reconecte o Wi-Fi para continuar."); render(); }
});
window.addEventListener("online", () => { if (!current) void initialize(); });
let initializing = false;
async function initialize() {
  if (initializing || current) return;
  initializing = true;
  render();
  if (!supported) { initializing = false; showError("Este navegador não suporta WebRTC. Abra o link no Safari atualizado ou em outro navegador moderno."); return; }
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 6000);
  try {
    const response = await fetch("/status", { cache: "no-store", signal: controller.signal });
    if (!response.ok) throw new Error("Não foi possível ler o modo selecionado no PC.");
    const state = await response.json();
    if (state.mode === "listen" || state.mode === "mic") { serverMode = state.mode; selectMode(serverMode); }
    $("error").hidden = true;
  } catch (error) { showError("Não foi possível consultar o PC. Você pode tentar Conectar ou atualizar esta página."); }
  finally { clearTimeout(timeout); initializing = false; ready = navigator.onLine; render(); }
}
applyLanguage();
// A self-signed certificate can prevent SW installation even after visiting the
// page. The live miniapp and iPhone Home Screen metadata work independently.
if ("serviceWorker" in navigator && window.isSecureContext) {
  void navigator.serviceWorker.register("/sw.js", { scope: "/", updateViaCache: "none" }).catch(() => {});
}
void initialize();
