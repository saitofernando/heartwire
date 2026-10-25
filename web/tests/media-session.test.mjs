import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import vm from 'node:vm';

const source = await readFile(new URL('../media-session.js', import.meta.url), 'utf8');

class Player extends EventTarget {
  paused = true;
  ended = false;
  error = null;
  readyState = 0;
  srcObject = null;
  plays = 0;
  pauses = 0;
  playError = null;
  play() {
    this.plays++;
    if (this.playError) return Promise.reject(this.playError);
    this.paused = false;
    this.readyState = 4;
    this.dispatchEvent(new Event('playing'));
    return Promise.resolve();
  }
  pause() {
    this.pauses++;
    this.paused = true;
    this.dispatchEvent(new Event('pause'));
  }
}

function setup({ api = true, throws = false, renderOnChange = false } = {}) {
  const handlers = new Map();
  const registrations = [];
  const originalMetadata = { title: 'Other player' };
  const media = { metadata: originalMetadata, playbackState: 'playing',
    setActionHandler(action, handler) {
      if (throws) throw new Error('Unsupported action');
      registrations.push(action);
      handlers.set(action, handler);
    },
  };
  const player = new Player();
  const track = new EventTarget();
  track.readyState = 'live';
  const stream = { active: true, getAudioTracks: () => [track] };
  let connected = false;
  let mode = 'listen';
  let title = 'Áudio do PC';
  let stopped = 0;
  const errors = [];
  const states = [];
  const window = { MediaMetadata: class { constructor(values) { Object.assign(this, values); } } };
  const navigator = api ? { mediaSession: media } : {};
  vm.runInNewContext(source, { window, navigator, URL, location: { href: 'https://192.168.1.20:8081/' } });
  let control;
  control = window.HeartWireMediaSession.create({ player,
    canControl: () => connected && mode === 'listen',
    onStop: () => { stopped++; connected = false; player.pause(); player.srcObject = null; control.clear(); },
    onPlayError: (error) => errors.push(error), getTitle: () => title,
    onPlaybackChange: (state) => { states.push(state); if (renderOnChange) control?.sync(); },
  });
  return { media, player, track, stream, handlers, registrations, states, errors,
    originalMetadata, control, get stopped() { return stopped; },
    connect() { connected = true; player.srcObject = stream; control.sync(); },
    disconnect() { connected = false; control.sync(); },
    setMode(value) { mode = value; control.sync(); },
    setTitle(value) { title = value; control.sync(); },
  };
}

test('idle helper does not take media focus or overwrite another player', () => {
  const app = setup();
  app.control.sync();
  app.control.clear();
  app.control.destroy();
  assert.equal(app.media.metadata, app.originalMetadata);
  assert.equal(app.media.playbackState, 'playing');
  assert.deepEqual(app.registrations, []);
  assert.equal(app.player.plays, 0);
});

test('live playback publishes artwork and language without changing its stream', async () => {
  const app = setup();
  app.connect();
  assert.equal(app.media.metadata, app.originalMetadata, 'no lock-screen claim before playback');
  await app.player.play();
  assert.equal(app.media.metadata.title, 'Áudio do PC');
  assert.equal(app.media.metadata.artist, 'HeartWire');
  assert.equal(app.media.metadata.artwork[1].src, 'https://192.168.1.20:8081/icons/icon-512.png');
  assert.equal(app.media.playbackState, 'playing');
  app.setTitle('PC audio');
  assert.equal(app.media.metadata.title, 'PC audio');
  assert.equal(app.player.srcObject, app.stream);
  assert.deepEqual([...app.handlers.keys()].sort(), ['pause', 'play', 'stop']);
});

test('pause/play controls keep the peer and stop clears controls permanently', async () => {
  const app = setup({ renderOnChange: true });
  app.connect();
  await app.player.play();
  const stalePlay = app.handlers.get('play');
  app.handlers.get('pause')();
  assert.equal(app.player.paused, true);
  assert.equal(app.media.playbackState, 'paused');
  assert.equal(app.player.srcObject, app.stream);
  assert.equal(app.stopped, 0);
  await app.handlers.get('play')();
  assert.equal(app.player.paused, false);
  assert.equal(app.media.playbackState, 'playing');
  app.handlers.get('stop')();
  assert.equal(app.stopped, 1);
  assert.equal(app.media.metadata, null);
  assert.equal(app.media.playbackState, 'none');
  for (const handler of app.handlers.values()) assert.equal(handler, null);
  const plays = app.player.plays;
  stalePlay();
  app.setTitle('PC audio');
  assert.equal(app.player.plays, plays);
  assert.equal(app.media.metadata, null);
});

test('microphone mode, ended tracks and stale handlers cannot start playback', async () => {
  const app = setup();
  app.connect();
  await app.player.play();
  const stalePlay = app.handlers.get('play');
  app.setMode('mic');
  stalePlay();
  assert.equal(app.player.plays, 1);
  assert.equal(app.media.metadata, null);
  app.setMode('listen');
  const resumedPlay = app.handlers.get('play');
  app.track.readyState = 'ended';
  app.track.dispatchEvent(new Event('ended'));
  resumedPlay();
  assert.equal(app.player.plays, 1);
  assert.equal(app.media.metadata, null);
  assert.equal(app.media.playbackState, 'none');
});

test('unsupported APIs and rejected playback leave the audio owner in control', async () => {
  const unavailable = setup({ api: false });
  unavailable.connect();
  await unavailable.player.play();
  unavailable.player.pause();
  assert.equal(unavailable.control.supported, false);
  assert.deepEqual(unavailable.states, ['paused', 'playing', 'paused']);
  unavailable.control.destroy();
  const partial = setup({ throws: true });
  partial.connect();
  await partial.player.play();
  partial.control.sync();
  partial.disconnect();
  assert.equal(partial.media.metadata, null);
  const rejected = setup();
  rejected.connect();
  await rejected.player.play();
  rejected.handlers.get('pause')();
  rejected.player.playError = new DOMException('User gesture required', 'NotAllowedError');
  await rejected.handlers.get('play')();
  assert.equal(rejected.media.playbackState, 'paused');
  assert.equal(rejected.errors.length, 1);
  assert.equal(rejected.player.srcObject, rejected.stream);
});
