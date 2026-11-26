// Runs in every website before its own scripts. Kept tiny on purpose.
(() => {
  if (window.top !== window || window.__lurt) return;
  Object.defineProperty(window, '__lurt', { value: 1 });
  const ipc = window.ipc;
  const send = (m) => { try { ipc.postMessage(JSON.stringify(m)); } catch (_) {} };

  // Browser shortcuts that WebView2 does not handle on its own.
  const KEYS = new Set([
    'ctrl+t', 'ctrl+w', 'ctrl+l', 'ctrl+tab', 'ctrl+shift+tab', 'ctrl+d', 'ctrl+h',
    'ctrl+shift+t', 'ctrl+comma', 'ctrl+shift+b', 'ctrl+shift+o', 'alt+d', 'f6',
    'ctrl+1', 'ctrl+2', 'ctrl+3', 'ctrl+4', 'ctrl+5', 'ctrl+6', 'ctrl+7', 'ctrl+8', 'ctrl+9',
  ]);
  addEventListener('keydown', (e) => {
    let k = e.key.toLowerCase();
    if (k === ',') k = 'comma';
    const c = (e.ctrlKey ? 'ctrl+' : '') + (e.altKey ? 'alt+' : '') + (e.shiftKey ? 'shift+' : '') + k;
    if (KEYS.has(c)) {
      e.preventDefault();
      e.stopImmediatePropagation();
      send({ t: 'key', k: c });
    }
  }, true);

  // Lets the browser close its side panel when you click into the page.
  addEventListener('pointerdown', () => send({ t: 'down' }), { capture: true, passive: true });

  const favicon = () => {
    const l = document.querySelector('link[rel~="icon"], link[rel="apple-touch-icon"]');
    send({ t: 'fav', u: l && l.href ? l.href : location.origin + '/favicon.ico' });
  };
  const ready = () => { send({ t: 'dom' }); favicon(); };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', ready, { once: true });
  else ready();
})();
