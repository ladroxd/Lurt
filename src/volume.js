// Lurt's own volume for a tab, on top of the site's volume controls. Runs in every frame
// before the page's scripts. At 100% it changes nothing you can hear.
(() => {
  const media = window.HTMLMediaElement;
  if (!media || window.__lurtVolume) return;
  let factor = 1;

  // Audio and video elements. The site keeps reading and writing its own volume; the
  // element actually plays at that volume times the tab's.
  const prop = Object.getOwnPropertyDescriptor(media.prototype, 'volume');
  const wanted = new WeakMap(); // element -> the volume the site asked for
  const seen = new Set(); // WeakRefs, including players that are not in the document
  const known = new WeakSet();
  const apply = (el) => {
    if (!known.has(el)) {
      if (factor === 1) return;
      known.add(el);
      seen.add(new WeakRef(el));
    }
    if (!wanted.has(el)) wanted.set(el, prop.get.call(el));
    prop.set.call(el, wanted.get(el) * factor);
  };
  Object.defineProperty(media.prototype, 'volume', {
    ...prop,
    get() { return wanted.has(this) ? wanted.get(this) : prop.get.call(this); },
    set(v) {
      prop.set.call(this, v); // throws for values outside 0..1, like the original
      if (factor === 1 && !known.has(this)) return;
      wanted.set(this, Number(v));
      apply(this);
    },
  });
  const play = media.prototype.play;
  media.prototype.play = function (...a) { apply(this); return play.apply(this, a); };
  // Autoplay never calls play().
  document.addEventListener('play', (e) => { if (e.target instanceof media) apply(e.target); }, true);

  // Web Audio (games, some players): everything headed for the speakers passes a gain node.
  const Context = window.AudioContext;
  const gains = new Set(); // WeakRefs
  if (Context && window.AudioNode) {
    const byContext = new WeakMap();
    const { connect, disconnect } = AudioNode.prototype;
    const speakers = (target) => target instanceof AudioDestinationNode && target.context instanceof Context;
    const gainFor = (ctx) => {
      let g = byContext.get(ctx);
      if (!g) {
        g = ctx.createGain();
        g.gain.value = factor;
        connect.call(g, ctx.destination);
        byContext.set(ctx, g);
        gains.add(new WeakRef(g));
      }
      return g;
    };
    AudioNode.prototype.connect = function (target, ...a) {
      if (!speakers(target)) return connect.call(this, target, ...a);
      connect.call(this, gainFor(target.context), ...a);
      return target;
    };
    AudioNode.prototype.disconnect = function (target, ...a) {
      if (arguments.length && speakers(target)) return disconnect.call(this, gainFor(target.context), ...a);
      return disconnect.apply(this, arguments);
    };
  }

  const set = (f) => {
    f = Math.min(1, Math.max(0, Number(f)));
    if (!Number.isFinite(f) || f === factor) return;
    factor = f;
    for (const el of document.querySelectorAll('audio, video')) apply(el);
    for (const ref of seen) { const el = ref.deref(); if (el) apply(el); else seen.delete(ref); }
    for (const ref of gains) { const g = ref.deref(); if (g) g.gain.value = f; else gains.delete(ref); }
    for (let i = 0; i < window.frames.length; i++) window.frames[i].postMessage({ lurtVolume: f }, '*');
  };

  // Frames get the tab's volume from the frame around them; the top one gets it from Lurt.
  addEventListener('message', (e) => {
    const m = e.data;
    if (!m || typeof m !== 'object') return;
    if (window !== window.top && e.source === window.parent && typeof m.lurtVolume === 'number') {
      e.stopImmediatePropagation();
      set(m.lurtVolume);
    } else if (m.lurtVolumeAsk === true && e.source) {
      e.stopImmediatePropagation();
      if (factor !== 1) e.source.postMessage({ lurtVolume: factor }, '*');
    }
  }, true);
  if (window === window.top) {
    Object.defineProperty(window, '__lurtVolume', { value: set });
    try { window.ipc.postMessage(JSON.stringify({ t: 'volumeAsk' })); } catch (_) {}
  } else {
    window.parent.postMessage({ lurtVolumeAsk: true }, '*');
  }
})();
