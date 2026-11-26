// Lurt privacy script: runs before a page's own scripts, in every frame.
(() => {
  if (window.__lurtPrivacy) return;
  Object.defineProperty(window, '__lurtPrivacy', { value: 1 });
  const C = __CFG__;
  const getter = (proto, key, fn) => {
    try { Object.defineProperty(proto, key, { get: fn, configurable: true, enumerable: true }); } catch (_) {}
  };

  // Global Privacy Control: tells sites not to sell or share your data.
  if (C.gpc) getter(Navigator.prototype, 'globalPrivacyControl', () => true);
  if (!C.fp) return;

  // Fingerprinting: sites read tiny details of your hardware to recognise you without cookies.
  // Make the usual ones generic, and add noise to canvas and audio reads that is fixed per
  // site and session, so a site sees the same values every time but can't link you to others.
  let seed = C.seed >>> 0;
  for (const ch of location.hostname) seed = Math.imul(seed ^ ch.charCodeAt(0), 16777619) >>> 0;
  const flip = (i) => (Math.imul((i + seed) | 0, 2654435761) >>> 27) === 0; // ~1 in 32

  getter(Navigator.prototype, 'hardwareConcurrency', () => 4);
  getter(Navigator.prototype, 'deviceMemory', () => 8);

  const noisy = (data) => {
    for (let i = 0; i < data.length; i += 4) if (data[i + 3] && flip(i)) data[i] ^= 1;
    return data;
  };
  const C2D = CanvasRenderingContext2D.prototype;
  const getImageData = C2D.getImageData;
  C2D.getImageData = function (...a) {
    const img = getImageData.apply(this, a);
    noisy(img.data);
    return img;
  };
  // toDataURL / toBlob: export a noisy copy of 2D canvases (WebGL canvases are left alone).
  const copy = (cv) => {
    let ctx = null;
    try { ctx = cv.getContext('2d'); } catch (_) {}
    if (!ctx || !cv.width || !cv.height) return cv;
    const c = document.createElement('canvas');
    c.width = cv.width; c.height = cv.height;
    const img = getImageData.call(ctx, 0, 0, cv.width, cv.height);
    noisy(img.data);
    c.getContext('2d').putImageData(img, 0, 0);
    return c;
  };
  const HC = HTMLCanvasElement.prototype;
  const toDataURL = HC.toDataURL, toBlob = HC.toBlob;
  HC.toDataURL = function (...a) { return toDataURL.apply(copy(this), a); };
  HC.toBlob = function (...a) { return toBlob.apply(copy(this), a); };

  // WebGL: hide the exact graphics card.
  const gl = (proto) => {
    if (!proto) return;
    const gp = proto.getParameter;
    proto.getParameter = function (p) {
      if (p === 0x9245) return 'Google Inc.';
      if (p === 0x9246) return 'ANGLE (Generic, Direct3D11)';
      return gp.call(this, p);
    };
  };
  gl(window.WebGLRenderingContext && WebGLRenderingContext.prototype);
  gl(window.WebGL2RenderingContext && WebGL2RenderingContext.prototype);

  // Audio: the same tiny fixed noise on rendered samples.
  if (window.AudioBuffer) {
    const gcd = AudioBuffer.prototype.getChannelData;
    const done = new WeakSet();
    AudioBuffer.prototype.getChannelData = function (...a) {
      const d = gcd.apply(this, a);
      if (!done.has(d)) { done.add(d); for (let i = 0; i < d.length; i += 97) if (flip(i)) d[i] += 1e-7; }
      return d;
    };
  }
})();
