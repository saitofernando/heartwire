"use strict";

// Media Session describes the existing live audio element. It does not grant
// background execution or change WebRTC buffering, capture, or playback cadence.
// WebKit can omit MediaStream Now Playing info without metadata/action handlers:
// https://github.com/WebKit/WebKit/blob/main/Source/WebCore/html/MediaElementSession.cpp
(function () {
  function create({ player, canControl, onStop, onPlayError = () => {},
    getTitle = () => "Áudio do PC", onPlaybackChange = () => {} }) {
    let media = null;
    try { media = navigator.mediaSession || null; } catch {}
    let source = null;
    let tracks = [];
    let hasPlayed = false;
    let playing = false;
    let destroyed = false;
    let syncing = false;
    let revision = 0;
    let publishedState = "none";
    let declaredState = "none";
    let ownsState = false;
    let metadata = null;
    let metadataTitle = null;
    const actions = new Set();

    function eligible() {
      try {
        return !destroyed && !!canControl() && !!player.srcObject &&
          player.srcObject.active !== false &&
          player.srcObject.getAudioTracks().some((track) => track.readyState === "live");
      } catch { return false; }
    }
    function publishState(state) {
      if (publishedState === state) return;
      publishedState = state;
      try { onPlaybackChange(state); } catch {}
    }
    function release() {
      revision++;
      if (!media) return;
      // An idle helper never clears another component's metadata or handlers.
      for (const action of actions) {
        try { media.setActionHandler(action, null); } catch {}
      }
      actions.clear();
      try { if (metadata && media.metadata === metadata) media.metadata = null; } catch {}
      try { if (ownsState && media.playbackState === declaredState) media.playbackState = "none"; } catch {}
      metadata = null;
      metadataTitle = null;
      ownsState = false;
    }
    function watchSource() {
      if (source === player.srcObject) return;
      for (const track of tracks) track.removeEventListener("ended", sync);
      source = player.srcObject;
      try { tracks = source?.getAudioTracks() || []; } catch { tracks = []; }
      for (const track of tracks) track.addEventListener("ended", sync);
      hasPlayed = false;
      playing = false;
      release();
    }
    function setAction(action, handler) {
      if (actions.has(action)) return;
      const originalSource = source;
      const originalRevision = revision;
      const guarded = () => {
        if (originalSource === player.srcObject && originalRevision === revision) return handler();
      };
      try { media.setActionHandler(action, guarded); actions.add(action); } catch {}
    }
    async function play() {
      if (!eligible()) { sync(); return; }
      const originalSource = player.srcObject;
      const originalRevision = revision;
      try {
        await player.play();
      } catch (error) {
        if (eligible() && originalSource === player.srcObject && originalRevision === revision) {
          try { onPlayError(error); } catch {}
        }
      }
      // A late result never reconnects a peer, restarts capture, or touches a
      // replacement stream. stop() remains the owner's explicit operation.
      sync();
    }
    function pause() {
      if (!eligible()) { sync(); return; }
      try { player.pause(); } catch {}
      sync();
    }
    function stop() {
      if (!eligible()) { sync(); return; }
      // The owner invalidates its peer before clear() notifies its UI; a UI
      // render must not republish metadata for the session being disconnected.
      try { onStop(); } catch {}
      finally { clear(); }
    }
    function sync() {
      if (destroyed || syncing) return;
      syncing = true;
      try {
        watchSource();
        if (!eligible()) {
          release();
          publishState("none");
          return;
        }
        // Also recognize playback already in progress when the owner attaches
        // a stream or the connection recovers after a brief network change.
        if (!player.paused && !player.ended && !player.error && player.readyState >= 2) {
          hasPlayed = true;
          playing = true;
        }
        const state = playing && !player.paused && !player.ended && !player.error ? "playing" : "paused";
        if (media && hasPlayed) {
          let title = "Áudio do PC";
          try { title = getTitle(); } catch {}
          if (metadataTitle !== title && typeof window.MediaMetadata === "function") {
            try {
              const next = new window.MediaMetadata({ title, artist: "HeartWire", artwork: [
                { src: new URL("/icons/icon-192.png", location.href).href, sizes: "192x192", type: "image/png" },
                { src: new URL("/icons/icon-512.png", location.href).href, sizes: "512x512", type: "image/png" },
              ] });
              media.metadata = next;
              metadata = next;
              metadataTitle = title;
            } catch { /* Artwork/metadata are optional on older WebKit builds. */ }
          }
          setAction("play", play);
          setAction("pause", pause);
          setAction("stop", stop);
          try { media.playbackState = state; declaredState = state; ownsState = true; } catch {}
          // This is an unseekable live stream: do not invent duration/position
          // or register track skipping, seeking, camera, or microphone actions.
        }
        publishState(state);
      } finally { syncing = false; }
    }
    function clear() {
      playing = false;
      hasPlayed = false;
      release();
      publishState("none");
    }
    function onPlaying() {
      watchSource();
      if (eligible()) { hasPlayed = true; playing = true; }
      sync();
    }
    function onNotPlaying() { playing = false; sync(); }
    const listeners = [["playing", onPlaying], ["pause", onNotPlaying],
      ["ended", onNotPlaying], ["emptied", onNotPlaying], ["error", onNotPlaying]];
    for (const [event, handler] of listeners) player.addEventListener(event, handler);
    function destroy() {
      if (destroyed) return;
      destroyed = true;
      clear();
      for (const [event, handler] of listeners) player.removeEventListener(event, handler);
      for (const track of tracks) track.removeEventListener("ended", sync);
      tracks = [];
    }
    return { sync, clear, destroy, supported: !!media };
  }
  window.HeartWireMediaSession = Object.freeze({ create });
})();
