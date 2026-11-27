'use strict';
// Filled in from Cargo.toml when Lurt is built.
const VERSION = '__LURT_VERSION__';
// Lurt browser UI. Talks to Rust with ipc.postMessage(JSON) and receives lurt.on(msg).

const $ = (s, r = document) => r.querySelector(s);
const send = (m) => window.ipc.postMessage(JSON.stringify(m));
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

// ---- icons -----------------------------------------------------------------
const P = {
  back: '<path d="M15 5l-7 7 7 7"/>',
  fwd: '<path d="M9 5l7 7-7 7"/>',
  reload: '<path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path d="M19.5 4.5v4h-4"/>',
  home: '<path d="M4.5 10.5L12 4l7.5 6.5V19a1 1 0 0 1-1 1H15v-5.5H9V20H5.5a1 1 0 0 1-1-1z"/>',
  plus: '<path d="M12 5.5v13M5.5 12h13"/>',
  user: '<circle cx="12" cy="8" r="4"/><path d="M4.5 20.5c1.2-3.8 4-5.5 7.5-5.5s6.3 1.7 7.5 5.5"/>',
  users: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c1-3.4 3.4-5 6.5-5s5.5 1.6 6.5 5M16 4.6a3.5 3.5 0 0 1 0 6.8M18 15.2c1.8.7 3 2.3 3.5 4.8"/>',
  logout: '<path d="M14 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-4M9 16l-4-4 4-4M5 12h10"/>',
  upload: '<path d="M12 15V4M7.5 8.5 12 4l4.5 4.5M4.5 15v3a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2v-3"/>',
  x: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
  star: '<path d="M12 3.8l2.5 5.1 5.6.8-4 3.9.9 5.6-5-2.6-5 2.6.9-5.6-4-3.9 5.6-.8z"/>',
  shield: '<path d="M12 3.5l7 2.8v5.2c0 4.3-3 7.9-7 9-4-1.1-7-4.7-7-9V6.3z"/><path d="M9 12l2 2 4-4"/>',
  download: '<path d="M12 4v11M7.5 10.5 12 15l4.5-4.5M5 19.5h14"/>',
  pause: '<path d="M9 6.5v11M15 6.5v11"/>',
  play: '<path d="M8.5 6v12l9.5-6z"/>',
  mask: '<path d="M3.5 9.5c0-1.4 1.1-2.5 2.5-2.5h12c1.4 0 2.5 1.1 2.5 2.5v1c0 3.3-2.2 6-5 6-1.6 0-2.4-1.5-3.5-1.5S10.1 16.5 8.5 16.5c-2.8 0-5-2.7-5-6z"/><circle cx="8.5" cy="11" r="1.4"/><circle cx="15.5" cy="11" r="1.4"/>',
  shieldOff: '<path d="M12 3.5l7 2.8v5.2c0 4.3-3 7.9-7 9-4-1.1-7-4.7-7-9V6.3z"/><path d="M4 4l16 16"/>',
  menu: '<path d="M5 9h14M5 15h9"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="M20 20l-4.2-4.2"/>',
  lock: '<rect x="5.5" y="10.5" width="13" height="9.5" rx="2.6"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5"/>',
  warn: '<path d="M12 4.5l8.5 15h-17z"/><path d="M12 10v4M12 16.8v.2"/>',
  globe: '<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17M12 3.5c2.4 2.4 3.4 5.4 3.4 8.5s-1 6.1-3.4 8.5c-2.4-2.4-3.4-5.4-3.4-8.5s1-6.1 3.4-8.5z"/>',
  clock: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
  trash: '<path d="M5 7h14M10 7V5h4v2M7 7l1 12.5h8L17 7"/>',
  folder: '<path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>',
  refresh: '<path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path d="M19.5 4.5v4h-4"/>',
  arrow: '<path d="M5 12h14M13 6l6 6-6 6"/>',
  sparkle: '<path d="M12 4l1.8 5.2L19 11l-5.2 1.8L12 18l-1.8-5.2L5 11l5.2-1.8z"/>',
  gear: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  palette: '<path d="M12 3.5a8.5 8.5 0 1 0 0 17c1.2 0 1.8-.8 1.8-1.7 0-1.3-1.1-1.6-1.1-2.8 0-1 .8-1.6 1.8-1.6h2.2a3.8 3.8 0 0 0 3.8-3.8c0-4-3.8-7.1-8.5-7.1z"/><circle cx="7.8" cy="11.2" r=".6"/><circle cx="10.4" cy="7.6" r=".6"/><circle cx="14.8" cy="7.8" r=".6"/>',
  grid: '<rect x="4.5" y="4.5" width="6" height="6" rx="1.6"/><rect x="13.5" y="4.5" width="6" height="6" rx="1.6"/><rect x="4.5" y="13.5" width="6" height="6" rx="1.6"/><rect x="13.5" y="13.5" width="6" height="6" rx="1.6"/>',
  database: '<ellipse cx="12" cy="6.5" rx="7" ry="2.8"/><path d="M5 6.5v11c0 1.5 3.1 2.8 7 2.8s7-1.3 7-2.8v-11M5 12c0 1.5 3.1 2.8 7 2.8s7-1.3 7-2.8"/>',
  bolt: '<path d="M13 3.5L5.5 13.5H12l-1 7 7.5-10H12z"/>',
  code: '<path d="M9 7l-5 5 5 5M15 7l5 5-5 5"/>',
  info: '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5M12 7.8v.2"/>',
  pin: '<path d="M9 4.5h6l-1 5 3 3H7l3-3zM12 12.5V20"/>',
  edit: '<path d="M14.5 5.5l4 4L9 19H5v-4z"/>',
  external: '<path d="M13.5 5.5h5v5M18.5 5.5L11 13M17 13.5V18a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h4.5"/>',
  download: '<path d="M12 4.5v11M7 11l5 5 5-5M5 19.5h14"/>',
  check: '<path d="M5 12.5l4.5 4.5L19 7.5"/>',
  sound: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4z"/><path d="M15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11"/>',
  soundOff: '<path d="M4 9.5h3.5L12 5.5v13l-4.5-4H4z"/><path d="M15.5 9.5l5 5M20.5 9.5l-5 5"/>',
  eye: '<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.8"/>',
};
// The Lurt logo (assets/lurt.svg): two clear-glass bars crossing into an L.
const LOGO = '<svg viewBox="0 0 100 100" aria-hidden="true"><defs> <linearGradient id="lg-tile" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1c2230"/><stop offset="1" stop-color="#0e1219"/></linearGradient> <radialGradient id="lg-glow" cx=".3" cy=".22" r=".85"><stop offset="0" stop-color="#fff" stop-opacity=".12"/><stop offset=".7" stop-color="#fff" stop-opacity="0"/></radialGradient> <linearGradient id="lg-body" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#fff" stop-opacity=".7"/></linearGradient> <linearGradient id="lg-rim" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".95"/><stop offset=".45" stop-color="#fff" stop-opacity=".15"/><stop offset="1" stop-color="#fff" stop-opacity=".45"/></linearGradient> </defs> <rect x="2" y="2" width="96" height="96" rx="26" fill="url(#lg-tile)"/> <rect x="2" y="2" width="96" height="96" rx="26" fill="url(#lg-glow)"/> <rect x="2.5" y="2.5" width="95" height="95" rx="25.5" fill="none" stroke="#fff" stroke-opacity=".14"/> <g transform="translate(18 18) scale(.64)"> <rect x="22" y="12" width="20" height="76" rx="10" fill="url(#lg-body)" fill-opacity=".22"/> <rect x="22.5" y="12.5" width="19" height="75" rx="9.5" fill="none" stroke="url(#lg-rim)" stroke-width="1.6"/> <rect x="22" y="68" width="62" height="20" rx="10" fill="url(#lg-body)" fill-opacity=".2"/> <rect x="22.5" y="68.5" width="61" height="19" rx="9.5" fill="none" stroke="url(#lg-rim)" stroke-width="1.6"/> </g></svg>';
const icon = (n) => `<svg class="i" viewBox="0 0 24 24" aria-hidden="true">${P[n]}</svg>`;
const WIN = {
  min: '<svg viewBox="0 0 10 10"><path d="M0 5.5h10"/></svg>',
  max: '<svg viewBox="0 0 10 10"><rect x=".5" y=".5" width="9" height="9" rx="1.6"/></svg>',
  restore: '<svg viewBox="0 0 10 10"><rect x=".5" y="2.5" width="7" height="7" rx="1.4"/><path d="M2.5 2.4V2A1.5 1.5 0 0 1 4 .5h4A1.5 1.5 0 0 1 9.5 2v4A1.5 1.5 0 0 1 8 7.5h-.4"/></svg>',
  close: '<svg viewBox="0 0 10 10"><path d="M.5.5l9 9M9.5.5l-9 9"/></svg>',
};

// ---- state -----------------------------------------------------------------
const S = {
  tabs: [], active: 0, back: false, fwd: false, star: false, allowed: false, max: false, fs: false,
  settings: null, backdrop: 'blur', shieldReady: false, modsDir: '',
  overlay: false, panel: null, lastPanel: 'bookmarks', rows: [], sel: 0, blank: true,
};
const cur = () => S.tabs.find((t) => t.id === S.active);
const ENGINES = { google: 'Google', duckduckgo: 'DuckDuckGo', bing: 'Bing', brave: 'Brave Search', startpage: 'Startpage', ecosia: 'Ecosia' };
const mq = matchMedia('(prefers-color-scheme: dark)');

const hostOf = (u) => { try { return new URL(u).hostname.replace(/^www\./, ''); } catch { return ''; } };
const pretty = (u) => {
  if (!u) return '';
  try {
    const x = new URL(u);
    if (x.protocol !== 'https:' && x.protocol !== 'http:') return u;
    const path = x.pathname === '/' ? '' : x.pathname;
    return x.hostname.replace(/^www\./, '') + (x.port ? ':' + x.port : '') + path + x.search + x.hash;
  } catch { return u; }
};
const hue = (s) => { let h = 0; for (const c of s) h = (h * 31 + c.charCodeAt(0)) >>> 0; return h % 360; };
const letter = (u, title) => {
  const host = hostOf(u) || title || '?';
  return `<span class="letter" style="background:hsl(${hue(host)} 55% 48%)">${esc(host[0] || '?')}</span>`;
};
const looksLikeUrl = (q) => /^[a-z]+:\/\//i.test(q) || /^(localhost|\d{1,3}(\.\d{1,3}){3})(:\d+)?(\/|$)/i.test(q) ||
  (!/\s/.test(q) && /^[\w-]+(\.[\w-]+)*\.[a-z]{2,}(:\d+)?(\/.*)?$/i.test(q));

// ---- messages from Rust ----------------------------------------------------
const H = {
  state(m) {
    Object.assign(S, m);
    brandFromTabs();
    renderTabs(); renderBar(); renderStart(); renderVol();
    // First real frame is on screen: let the native startup skeleton fade out.
    if (!S.painted) { S.painted = true; requestAnimationFrame(() => requestAnimationFrame(() => send({ t: 'painted' }))); }
  },
  settings(m) {
    S.settings = m.s; S.backdrop = m.backdrop; S.shieldReady = m.shield; S.modsDir = m.modsDir;
    applyLook();
    if (!document.activeElement?.closest('#settings, #side')) renderSettings();
    renderTiles();
  },
  list(m) {
    if (m.kind === 'top') { S.top = m.items; renderTiles(); }
    else if (S.panel === m.kind) renderList(m.kind, m.items);
  },
  suggest(m) { if (m.q === box.value.trim()) renderSugg(m.items); },
  overlay(m) {
    if (!S.overlay) return;
    if (!m.snap) { send({ t: 'overlay', shown: true }); return; }
    snap.onload = () => {
      snap.classList.add('on');
      send({ t: 'overlay', shown: true });
      requestAnimationFrame(() => requestAnimationFrame(() => S.overlay && snap.classList.add('melt')));
    };
    snap.src = m.snap;
  },
  focus() { addr.focus(); addr.select(); },
  panel(m) { if (m.p === 'settings') openSettings(); else if (m.p === 'downloads') openDownloadsPage(); else togglePanel(m.p); },
  mods(m) { $('#mods-css').textContent = m.css || ''; },
  toast(m) { toast(m.text); },
  sources(m) {
    S.sources = m.items;
    if (settingsOpen()) renderSettings();
  },
  imported(m) {
    S.importing = null;
    S.importMsg = m.text;
    toast(m.text);
    renderSettings();
  },
  blocked(m) {
    const t = S.tabs.find((x) => x.id === m.id);
    if (t) t.blocked = m.n;
    if (m.id === S.active) renderShield();
    if (m.id === S.active && (S.overlay === 'shield-open' || S.panel === 'trackers')) refreshTrackers();
  },
  preview(m) {
    if (!m.img) { S.previews.delete(m.id); renderWake(cur()); return; }
    shrinkPreview(m.img).then((small) => { S.previews.set(m.id, small); renderWake(cur()); });
  },
  pageDown() { if (S.panel) closePanel(); },
  shieldInfo(m) {
    if (m.id !== S.active) return;
    S.shieldInfo = m;
    const t = cur(); if (t) t.blocked = m.n;
    if (S.overlay === 'shield-open') renderPop();
    if (S.panel === 'trackers') renderTrackers();
  },
};
window.lurt = { on: (m) => { try { (H[m.t] || (() => {}))(m); } catch (e) { console.error(e); } } };

// ---- look ------------------------------------------------------------------
function applyLook() {
  const s = S.settings; if (!s) return;
  const root = document.documentElement;
  root.style.setProperty('--accent', s.accent);
  root.style.setProperty('--r', s.radius + 'px');
  root.style.setProperty('--tint', S.backdrop === 'none' ? 1 : (0.12 + (s.tint / 100) * 0.88).toFixed(3));
  root.dataset.mode = s.theme === 'system' ? (mq.matches ? 'dark' : 'light') : s.theme;
  root.dataset.backdrop = S.backdrop;
  root.toggleAttribute('data-compact', !!s.compact);
  root.toggleAttribute('data-calm', !!s.reduce_motion);
  root.toggleAttribute('data-noclock', !s.show_clock);
  syncBar();
  $('#ui-css').textContent = s.ui_css || '';
  requestAnimationFrame(sendInsets);
}
mq.addEventListener('change', applyLook);

// The page views are native windows laid over #viewport; tell Rust where it is.
let lastInsets = '';
function sendInsets() {
  const r = $('#viewport').getBoundingClientRect();
  const m = { t: 'insets', top: r.top, left: r.left, right: innerWidth - r.right, bottom: innerHeight - r.bottom };
  const key = [m.top, m.left, m.right, m.bottom].map((v) => v.toFixed(1)).join();
  if (key !== lastInsets) { lastInsets = key; send(m); }
}
new ResizeObserver(sendInsets).observe($('#viewport'));
addEventListener('resize', sendInsets);

// ---- tabs ------------------------------------------------------------------
const tabEls = new Map();
function favHtml(t) {
  if (t.loading) return '<i class="spin"></i>';
  if (!t.url) return icon('sparkle');
  if (t.fav) return `<img src="${esc(t.fav)}" alt="" onerror="this.replaceWith(Object.assign(document.createElement('span'),{innerHTML:${esc(JSON.stringify(letter(t.url, t.title)))}}).firstChild)">`;
  const host = /^https?:/i.test(t.url) ? hostOf(t.url) : '';
  return host ? iconImg(host, letter(t.url, t.title)) : letter(t.url, t.title);
}
function renderTabs() {
  const list = $('#tablist');
  const seen = new Set();
  S.tabs.forEach((t, i) => {
    seen.add(t.id);
    let el = tabEls.get(t.id);
    if (!el) {
      el = document.createElement('div');
      el.className = 'tab enter lens';
      el.dataset.id = t.id;
      el.innerHTML = `<span class="fav"></span><span class="title"></span><button class="snd"></button><button class="x" title="Close (Ctrl+W)">${icon('x')}</button>`;
      el.addEventListener('animationend', () => el.classList.remove('enter'), { once: true });
      tabEls.set(t.id, el);
    }
    // While a tab is being dragged the strip keeps the order under the pointer.
    if (list.children[i] !== el && (!tabDrag?.on || !el.parentNode)) list.insertBefore(el, list.children[i] || null);
    el.classList.toggle('active', t.id === S.active);
    el.classList.toggle('asleep', !!t.asleep);
    el.classList.toggle('private', !!t.private);
    const title = t.title || (t.url ? pretty(t.url) : 'New tab');
    if (el._title !== title) { el._title = title; el.title = title; el.querySelector('.title').textContent = title; }
    const fk = t.loading ? 'spin' : `${t.url}|${t.fav}|${!!t.private}|${S.icons?.[hostOf(t.url)] ?? ''}`;
    const quiet = t.muted || t.volume === 0;
    const sk = t.audible || t.muted || t.volume < 100 ? (quiet ? 'soundOff' : 'sound') : '';
    if (el._snd !== sk) {
      el._snd = sk;
      const b = el.querySelector('.snd');
      b.innerHTML = sk ? icon(sk) : '';
      b.title = quiet ? 'Unmute tab' : 'Mute tab';
      el.classList.toggle('sounds', !!sk);
    }
    if (el._fav !== fk) { el._fav = fk; el.querySelector('.fav').innerHTML = t.private && !t.url && !t.loading ? icon('mask') : favHtml(t); }
  });
  for (const [id, el] of tabEls) if (!seen.has(id)) { el.remove(); tabEls.delete(id); }
  requestAnimationFrame(() => {
    for (const el of tabEls.values()) el.classList.toggle('narrow', el.offsetWidth < 74);
  });
}
$('#tablist').addEventListener('mousedown', (e) => {
  const el = e.target.closest('.tab'); if (!el) return;
  const id = +el.dataset.id;
  if (e.button === 1) { e.preventDefault(); send({ t: 'close', id }); return; }
  if (e.button !== 0) return;
  if (e.target.closest('.x, .snd')) return;
  if (id !== S.active) send({ t: 'activate', id });
  tabDrag = { el, id, x: e.clientX, grab: e.clientX - el.getBoundingClientRect().left, on: false };
});
// Drag a tab along the strip: it follows the pointer and the others slide out of its way.
let tabDrag = null;
addEventListener('mousemove', (e) => {
  const d = tabDrag; if (!d) return;
  if (!(e.buttons & 1) || !d.el.isConnected) { endTabDrag(); return; }
  if (!d.on) {
    if (Math.abs(e.clientX - d.x) < 5) return;
    d.on = true;
    d.el.classList.add('dragging');
  }
  const list = $('#tablist');
  const box = list.getBoundingClientRect();
  const w = d.el.offsetWidth;
  const left = Math.max(box.left, Math.min(e.clientX - d.grab, box.right - w));
  // Layout positions (offsetLeft) ignore the slide animations still running.
  const mid = left - box.left + w / 2;
  const others = [...list.children].filter((x) => x !== d.el);
  const next = others.find((x) => mid < x.offsetLeft - list.offsetLeft + x.offsetWidth / 2) || null;
  if (d.el.nextElementSibling !== next) {
    const before = new Map(others.map((x) => [x, x.offsetLeft]));
    list.insertBefore(d.el, next);
    if (!S.settings?.reduce_motion) {
      for (const [x, was] of before) {
        const by = was - x.offsetLeft;
        if (by) x.animate([{ transform: `translateX(${by}px)` }, { transform: 'none' }], { duration: 180, easing: 'cubic-bezier(.2,.8,.2,1)' });
      }
    }
  }
  d.el.style.transform = `translateX(${left - box.left - (d.el.offsetLeft - list.offsetLeft)}px)`;
});
function endTabDrag() {
  const d = tabDrag; if (!d) return;
  tabDrag = null;
  if (!d.on) return;
  d.el.classList.remove('dragging');
  d.el.style.transform = '';
  const ids = [...$('#tablist').children].map((x) => +x.dataset.id);
  S.tabs.sort((a, b) => ids.indexOf(a.id) - ids.indexOf(b.id));
  renderTabs();
  send({ t: 'tabOrder', ids });
}
addEventListener('mouseup', endTabDrag);
addEventListener('blur', endTabDrag);
$('#tablist').addEventListener('click', (e) => {
  const snd = e.target.closest('.snd');
  if (snd) { toggleMute(+snd.closest('.tab').dataset.id); return; }
  const x = e.target.closest('.x'); if (!x) return;
  send({ t: 'close', id: +x.closest('.tab').dataset.id });
});
addEventListener('resize', () => requestAnimationFrame(() => {
  for (const el of tabEls.values()) el.classList.toggle('narrow', el.offsetWidth < 74);
}));

// ---- toolbar ---------------------------------------------------------------
const addr = $('#addr');
const startQ = $('#start-q');
// The search box in use: the address bar or the start page's. Suggestions follow it.
let box = addr;
const omni = $('#omni');
const snap = $('#snap');
$('#newtab').innerHTML = icon('plus');
$('#back').innerHTML = icon('back');
$('#fwd').innerHTML = icon('fwd');
$('#reload').innerHTML = icon('reload');
$('#home').innerHTML = icon('home');
$('#star').innerHTML = icon('star');
$('#menu').innerHTML = icon('menu');
$('#side-close').innerHTML = icon('x');
$('#start-search .ic').innerHTML = icon('search');
$('#shield').insertAdjacentHTML('afterbegin', icon('shield'));
for (const b of document.querySelectorAll('#win button')) b.innerHTML = WIN[b.dataset.a];

function renderBar() {
  const t = cur();
  $('#back').disabled = !S.back;
  $('#fwd').disabled = !S.fwd;
  if (document.activeElement !== addr) addr.value = t && t.url ? pretty(t.url) : '';
  const lock = $('#lock');
  const kind = !t || !t.url ? 'search' : t.url.startsWith('https:') ? 'lock' : t.url.startsWith('http:') ? 'warn' : 'globe';
  if (lock._k !== kind) { lock._k = kind; lock.innerHTML = icon(kind); lock.className = kind === 'warn' ? 'warn' : ''; }
  lock.title = kind === 'lock' ? 'Secure connection' : kind === 'warn' ? 'Not secure: this site does not use HTTPS' : '';
  $('#star').style.display = t && t.url ? '' : 'none';
  $('#star').classList.toggle('on', !!S.star);
  $('#win [data-a="max"]').innerHTML = S.max ? WIN.restore : WIN.max;
  $('#win [data-a="max"]').title = S.max ? 'Restore' : 'Maximize';
  document.documentElement.toggleAttribute('data-max', !!S.max);
  const waking = !!(t && t.url && !t.shown);
  document.body.classList.toggle('waking', waking);
  if (waking) renderWake(t);
  renderShield();
}
function renderShield() {
  if (S.overlay === 'shield-open' && pop._tab !== S.active) closeShield(false);
  if (S.panel === 'trackers') {
    const key = `${S.active}|${cur()?.url}|${S.allowed}|${S.settings?.adblock}`;
    if (key !== S.trkKey) { S.trkKey = key; S.shieldInfo = null; renderTrackers(); refreshTrackers(); }
  }
  const t = cur(); const b = $('#shield'); const n = $('#shield-n');
  const on = S.settings ? S.settings.adblock && !S.allowed : true;
  b.classList.toggle('off', !on);
  const k = on ? 'shield' : 'shieldOff';
  if (b._k !== k) { b._k = k; b.querySelector('svg').outerHTML = icon(k); }
  const count = on && t ? t.blocked : 0;
  n.textContent = count > 99 ? '99+' : count || '';
  n.classList.toggle('on', count > 0);
  b.title = !S.settings?.adblock ? 'Blocking is off (Settings → Privacy)'
    : S.allowed ? `Blocking paused on ${hostOf(t?.url || '')}`
    : `${count} blocked on this page`;
}
$('#newtab').onclick = () => send({ t: 'key', k: 'ctrl+t' });
$('#newtab').oncontextmenu = (e) => {
  e.preventDefault();
  showMenu(e.clientX, e.clientY, [
    { icon: 'plus', label: 'New tab', run: () => send({ t: 'key', k: 'ctrl+t' }) },
    { icon: 'mask', label: 'New private tab', run: () => send({ t: 'newPrivate' }) },
  ]);
};
$('#back').onclick = () => send({ t: 'nav', a: 'back' });
$('#fwd').onclick = () => send({ t: 'nav', a: 'forward' });
$('#reload').onclick = () => send({ t: 'nav', a: 'reload' });
$('#home').onclick = () => send({ t: 'nav', a: 'home' });
$('#star').onclick = () => (S.overlay === 'star-open' ? closeStar(true) : openStar());
$('#shield').onclick = () => (S.overlay === 'shield-open' ? closeShield() : openShield());

// ---- waking tab ------------------------------------------------------------
// The tab's last picture, shrunk to a tiny image so blurring it is nearly free.
S.previews = new Map();
function shrinkPreview(src) {
  return new Promise((done) => {
    const img = new Image();
    img.onload = () => {
      const w = 120; const h = Math.max(1, Math.round((img.height / img.width) * w));
      const c = document.createElement('canvas'); c.width = w; c.height = h;
      c.getContext('2d').drawImage(img, 0, 0, w, h);
      done(c.toDataURL('image/jpeg', 0.8));
    };
    img.onerror = () => done(null);
    img.src = src;
  });
}
function renderWake(t) {
  if (!t || !t.url || t.shown) return;
  const w = $('#wake'); const bg = w.querySelector('.wake-bg');
  const pic = S.previews.get(t.id);
  if (bg._pic !== pic) {
    bg._pic = pic;
    bg.style.backgroundImage = pic ? `url("${pic}")` : '';
    bg.classList.toggle('none', !pic);
  }
  const host = hostOf(t.url);
  // Per tab and site: a tab can go on to another site while it loads.
  if (w._id !== t.id + host) {
    w._id = t.id + host;
    w.querySelector('.orb-ic').innerHTML = `<img src="${esc(t.fav || `https://${host}/favicon.ico`)}" alt="" onerror="this.outerHTML=${esc(JSON.stringify(letter(t.url, t.title)))}">`;
    w.querySelector('.wake-t').textContent = siteName(t.url);
    wakeBrand(t);
  }
}

// The site's name for the loading screen: "youtube.com" -> "Youtube", "m.twitch.tv" -> "Twitch",
// "bbc.co.uk" -> "Bbc".
function siteName(u) {
  const h = hostOf(u);
  if (!h) return pretty(u);
  const parts = h.split('.');
  if (parts.length < 2 || /^\d+$/.test(parts[parts.length - 1])) return h;
  let i = parts.length - 2;
  if (parts.length >= 3 && parts[parts.length - 1].length === 2 && /^(co|com|org|net|gov|ac|edu)$/.test(parts[i])) i--;
  const n = parts[i];
  return n[0].toUpperCase() + n.slice(1);
}

// Loading screen tint: the color of the site's logo, faded and breathing. Well-known sites
// get their brand color at once; for the rest Rust downloads the favicon (pages can't read
// pixels of images from other sites) and its strongest color is picked here.
const BRANDS = {
  youtube: '#ff0033', youtu: '#ff0033', twitch: '#9146ff', netflix: '#e50914', spotify: '#1db954', facebook: '#1877f2',
  instagram: '#e1306c', reddit: '#ff4500', discord: '#5865f2', linkedin: '#0a66c2', tiktok: '#fe2c55', whatsapp: '#25d366',
  amazon: '#ff9900', google: '#4285f4', chatgpt: '#10a37f', openai: '#10a37f', claude: '#d97757', anthropic: '#d97757',
  pinterest: '#e60023', telegram: '#26a5e4', kick: '#53fc18', soundcloud: '#ff5500', twitter: '#1d9bf0', paypal: '#0070e0',
};
S.brand = new Map();
function wakeBrand(t) {
  const host = hostOf(t.url);
  let c = BRANDS[siteName(t.url).toLowerCase()];
  if (c === undefined && host) {
    if (!S.brand.has(host)) { S.brand.set(host, null); send({ t: 'favColor', host, url: t.fav || '' }); }
    c = S.brand.get(host);
  }
  applyBrand(c);
}
// Sites whose /favicon.ico can't be fetched (blocked, or declared only in the page): use the
// icon the page declares once the tab knows it, so the color is ready next time.
S.brandTried = new Set();
function brandFromTabs() {
  for (const t of S.tabs || []) {
    const host = hostOf(t.url);
    if (!host || !t.fav || !/^https?:/.test(t.fav) || S.brandTried.has(host)) continue;
    if (BRANDS[siteName(t.url).toLowerCase()] || S.brand.get(host)) continue;
    if (S.brand.get(host) === null) continue; // a lookup is still running
    S.brandTried.add(host); S.brand.set(host, null);
    send({ t: 'favColor', host, url: t.fav });
  }
}
function applyBrand(c) {
  const w = $('#wake');
  if (c) w.style.setProperty('--brand', c);
  w.classList.toggle('branded', !!c);
}
H.favData = (m) => iconColor(m.data, (c) => {
  S.brand.set(m.host, c || '');
  const t = cur();
  if (t && !t.shown && hostOf(t.url) === m.host) applyBrand(c);
});
function iconColor(src, done) {
  if (!src) { done(null); return; }
  const img = new Image();
  img.onerror = () => done(null);
  img.onload = () => {
    const cv = document.createElement('canvas'); cv.width = cv.height = 32;
    const g = cv.getContext('2d', { willReadFrequently: true });
    g.drawImage(img, 0, 0, 32, 32);
    let d; try { d = g.getImageData(0, 0, 32, 32).data; } catch { done(null); return; }
    // Bucket colorful pixels by hue, weighted by saturation; skip grays, white and black.
    const buckets = new Map();
    for (let i = 0; i < d.length; i += 4) {
      if (d[i + 3] < 140) continue;
      const r = d[i], gr = d[i + 1], b = d[i + 2];
      const max = Math.max(r, gr, b), min = Math.min(r, gr, b);
      const l = (max + min) / 510;
      const s = max === min ? 0 : (max - min) / (255 - Math.abs(max + min - 255));
      if (s < 0.25 || l < 0.12 || l > 0.92) continue;
      let h = max === r ? (gr - b) / (max - min) : max === gr ? 2 + (b - r) / (max - min) : 4 + (r - gr) / (max - min);
      h = ((h * 60) + 360) % 360;
      const k = Math.round(h / 24) % 15;
      const e = buckets.get(k) || { n: 0, r: 0, g: 0, b: 0 };
      e.n += s; e.r += r * s; e.g += gr * s; e.b += b * s;
      buckets.set(k, e);
    }
    let best = null;
    for (const e of buckets.values()) if (!best || e.n > best.n) best = e;
    if (!best || best.n < 6) { done(null); return; }
    done(`rgb(${Math.round(best.r / best.n)} ${Math.round(best.g / best.n)} ${Math.round(best.b / best.n)})`);
  };
  img.src = src;
}

// ---- shield popover ---------------------------------------------------------
const pop = $('#shield-pop');
const KINDS = { script: 'scripts', image: 'images', xhr: 'requests', stylesheet: 'styles', media: 'media',
  font: 'fonts', websocket: 'sockets', ping: 'pings', csp_report: 'reports', other: 'other' };
S.shieldInfo = null;
let popT;
function openShield() {
  if (settingsOpen()) return;
  S.shieldInfo = null; pop._tab = S.active;
  renderPop();
  openOverlay('shield-open');
  send({ t: 'shieldInfo' });
}
function blockedList(info) {
  return `<div class="blk">` + info.items.map((i) =>
    `<div class="item" title="${esc(i.h)}"><span class="fav">${letter('https://' + i.h, i.h)}</span><div class="txt"><span class="t">${esc(i.h)}</span>` +
    `<span class="u">${esc(i.k.map((k) => KINDS[k] || k).join(', '))}</span></div><em>${i.n}</em></div>`).join('') + `</div>`;
}
function refreshTrackers() { clearTimeout(popT); popT = setTimeout(() => send({ t: 'shieldInfo' }), 200); }
function closeShield(focusPage) {
  if (S.overlay === 'shield-open') closeOverlay(focusPage);
}
function placePop() {
  const r = $('#shield').getBoundingClientRect();
  pop.style.top = r.bottom + 8 + 'px';
  pop.style.left = Math.max(8, Math.min(r.right - 320 + 6, innerWidth - 328)) + 'px';
}
function renderPop() {
  const t = cur(); const info = S.shieldInfo;
  const host = hostOf(t?.url || '');
  const global = !!S.settings?.adblock;
  const on = global && !S.allowed;
  const n = on && t ? t.blocked : 0;
  pop.classList.toggle('off', !on);
  let html = `<div class="head">${icon(on ? 'shield' : 'shieldOff')}<div style="min-width:0"><b>${on ? 'Protected' : 'Not blocking'}</b>` +
    `<small>${esc(host || 'Start page')}</small></div></div>`;
  if (!global) {
    html += `<label class="set"><div class="lbl"><b>Block ads &amp; trackers</b><small>Off for every site</small></div>` +
      `<span class="switch"><input type="checkbox" data-a="global"><i></i></span></label>`;
  } else if (host) {
    html += `<label class="set"><div class="lbl"><b>Block on this site</b><small>${S.allowed ? 'Paused, ads will show here' : 'Turn off if the site breaks'}</small></div>` +
      `<span class="switch"><input type="checkbox" data-a="site" ${on ? 'checked' : ''}><i></i></span></label>`;
  }
  if (on && host) {
    html += `<div class="count"><strong>${n}</strong><span>${n === 1 ? 'request' : 'requests'} blocked on this page</span></div>`;
    if (!info) html += `<div class="empty">Loading details...</div>`;
    else if (!info.items.length) html += `<div class="empty">Nothing blocked yet. Trackers that load later will show up here.</div>`;
    else {
      html += blockedList(info);
    }
  } else if (!host && global) {
    html += `<div class="empty">Open a website to see what gets blocked.</div>`;
  }
  html += `<div class="foot"><button data-a="settings">More in Trackers</button></div>`;
  pop.innerHTML = html;
  placePop();
}
pop.addEventListener('change', (e) => {
  const a = e.target.dataset.a;
  if (a === 'site') send({ t: 'shield' });
  if (a === 'global') { S.settings.adblock = true; send({ t: 'settings', s: S.settings }); }
  // The page reloads with the new rule, so the snapshot behind us is stale.
  setTimeout(() => closeShield(false), 280);
});
pop.addEventListener('click', (e) => {
  if (e.target.closest('[data-a="settings"]')) { closeShield(false); if (S.panel !== 'trackers') togglePanel('trackers'); }
});
addEventListener('mousedown', (e) => {
  if (S.overlay === 'shield-open' && !pop.contains(e.target) && !$('#shield').contains(e.target)) closeShield(false);
}, true);
addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && S.overlay === 'shield-open') { e.preventDefault(); closeShield(true); }
});
addEventListener('resize', () => S.overlay === 'shield-open' && placePop());
$('#menu').onclick = () => togglePanel(S.panel || S.lastPanel);
$('#gear').innerHTML = icon('gear');
$('#gear').onclick = () => (settingsOpen() ? closeSettings() : openSettings());
for (const b of document.querySelectorAll('#win button')) b.onclick = () => send({ t: 'win', a: b.dataset.a });

let toastT;
function toast(text) {
  const c = $('#chip');
  c.textContent = text; c.classList.add('on');
  clearTimeout(toastT); toastT = setTimeout(() => c.classList.remove('on'), 2200);
}

// window dragging (double-click toggles maximize) and resizing
let lastDown = 0;
$('#top').addEventListener('mousedown', (e) => {
  if (e.button !== 0 || e.target.closest('button, .tab, input, #omni')) return;
  const now = performance.now();
  if (now - lastDown < 380) { lastDown = 0; send({ t: 'win', a: 'max' }); return; }
  lastDown = now;
  // Maximized: start the drag (which restores the window) only once the mouse moves,
  // so clicking the empty toolbar or double-clicking doesn't restore it by accident.
  if (S.max) { dragFrom = [e.screenX, e.screenY]; return; }
  send({ t: 'win', a: 'drag' });
});
let dragFrom = null;
addEventListener('mousemove', (e) => {
  if (!dragFrom) return;
  if (!(e.buttons & 1)) { dragFrom = null; return; }
  if (Math.hypot(e.screenX - dragFrom[0], e.screenY - dragFrom[1]) > 5) { dragFrom = null; send({ t: 'win', a: 'drag' }); }
});
addEventListener('mouseup', () => { dragFrom = null; });
for (const r of document.querySelectorAll('.rz')) {
  r.addEventListener('mousedown', (e) => { if (e.button === 0) send({ t: 'win', a: 'resize', d: r.dataset.d }); });
}

// liquid highlight that follows the pointer
addEventListener('pointermove', (e) => {
  const el = e.target.closest && e.target.closest('.lens'); if (!el) return;
  const r = el.getBoundingClientRect();
  el.style.setProperty('--mx', (e.clientX - r.left) + 'px');
  el.style.setProperty('--my', (e.clientY - r.top) + 'px');
}, { passive: true });

// ---- address bar & suggestions --------------------------------------------
const sugg = $('#sugg');
// Anything drawn over the page (address suggestions, shield popover) asks Rust to swap
// the live page for a blurred snapshot, since the page is a native view on top of us.
function openOverlay(cls = 'omni-open') {
  if (S.overlay) return;
  S.overlay = cls;
  document.body.classList.add(cls);
  send({ t: 'overlay', open: true });
}
function closeOverlay(focusPage) {
  if (!S.overlay) return;
  document.body.classList.remove(S.overlay);
  S.overlay = false;
  snap.classList.remove('on', 'melt');
  sugg.classList.remove('has');
  send({ t: 'overlay', open: false, focusPage: !!focusPage });
  setTimeout(() => { if (!S.overlay) snap.removeAttribute('src'); }, 250);
}
function placeSugg() {
  const r = (box === startQ ? $('#start-search') : omni).getBoundingClientRect();
  Object.assign(sugg.style, { left: r.left + 'px', top: r.bottom + 6 + 'px', width: r.width + 'px' });
}
function renderSugg(items) {
  const q = box.value.trim();
  if (!q) { S.rows = []; sugg.classList.remove('has'); return; }
  const engine = ENGINES[S.settings?.search] || 'your search engine';
  const first = looksLikeUrl(q)
    ? { kind: 'go', icon: 'arrow', t: q, u: 'Open address' }
    : { kind: 'search', icon: 'search', t: q, u: `Search with ${engine}` };
  S.rows = [first, ...items.map((i) => ({ kind: 'url', icon: 'globe', t: i.title || pretty(i.url), u: pretty(i.url), url: i.url }))];
  S.sel = Math.min(S.sel, S.rows.length - 1);
  sugg.innerHTML = S.rows.map((r, i) =>
    `<div class="row${i === S.sel ? ' sel' : ''}" data-i="${i}">${r.kind === 'url' ? `<span class="fav">${knownIcon(r.url, r.t)}</span>` : icon(r.icon)}` +
    `<div class="txt"><span class="t">${esc(r.t)}</span><span class="u">${esc(r.u)}</span></div></div>`).join('');
  placeSugg();
  sugg.classList.add('has');
}
function pick(i, newTab) {
  const r = S.rows[i] || { kind: 'search' };
  const input = box.value.trim();
  if (!input && r.kind !== 'url') return;
  if (r.kind === 'url') send({ t: 'open', url: r.url, newTab });
  else send({ t: 'go', input, newTab });
  box.blur();
}
let suggT;
function boxFocus(input) {
  box = input;
  S.sel = 0; S.rows = []; sugg.classList.remove('has');
  openOverlay();
}
addr.addEventListener('focus', () => {
  if (S.overlay === 'shield-open') closeShield();
  if (settingsOpen()) closeSettings();
  const t = cur();
  addr.value = t?.url || '';
  addr.select();
  omni.classList.add('focus');
  boxFocus(addr);
});
startQ.addEventListener('focus', () => {
  if (S.overlay === 'shield-open') closeShield();
  boxFocus(startQ);
});
startQ.addEventListener('blur', () => {
  if (!document.hasFocus() && document.activeElement === startQ) return;
  startQ.value = '';
  if (S.overlay === 'omni-open') closeOverlay(false);
});
// First click selects the whole address (like other browsers); a drag on that first
// click, and every click after it, selects text normally.
let keepSelection = false;
addr.addEventListener('mousedown', () => { keepSelection = document.activeElement !== addr; });
addr.addEventListener('mouseup', (e) => {
  if (keepSelection && addr.selectionStart === addr.selectionEnd) { e.preventDefault(); addr.select(); }
  keepSelection = false;
});
addr.addEventListener('blur', () => {
  // The window itself losing focus (e.g. while the page behind is swapped for its
  // snapshot) isn't leaving the address bar: keep it open with your selection.
  if (!document.hasFocus() && document.activeElement === addr) return;
  omni.classList.remove('focus');
  if (S.overlay === 'omni-open') closeOverlay(false);
  renderBar();
});
function boxInput() {
  openOverlay(); // reopen if something else closed it while the box kept focus
  S.sel = 0;
  const q = box.value.trim();
  renderSugg(S.rows.length > 1 ? S.rows.slice(1).map((r) => ({ url: r.url, title: r.t })) : []);
  clearTimeout(suggT);
  if (q) suggT = setTimeout(() => send({ t: 'suggest', q }), 35);
}
function boxKey(e) {
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    if (!S.rows.length) return;
    e.preventDefault();
    S.sel = (S.sel + (e.key === 'ArrowDown' ? 1 : -1) + S.rows.length) % S.rows.length;
    sugg.querySelectorAll('.row').forEach((r, i) => r.classList.toggle('sel', i === S.sel));
  } else if (e.key === 'Enter') {
    e.preventDefault();
    pick(S.sel, e.altKey);
  } else if (e.key === 'Escape') {
    e.preventDefault();
    // First Escape undoes what you typed, the next one leaves the box.
    const was = box === addr ? cur()?.url || '' : '';
    if (box.value !== was && box.value) { box.value = was; box.select(); renderSugg([]); return; }
    S.overlay && closeOverlay(true);
    box.blur();
  }
}
for (const input of [addr, startQ]) {
  input.addEventListener('input', boxInput);
  input.addEventListener('keydown', boxKey);
}
sugg.addEventListener('mousedown', (e) => e.preventDefault());
sugg.addEventListener('click', (e) => {
  const r = e.target.closest('.row'); if (r) pick(+r.dataset.i, e.ctrlKey);
});
addEventListener('resize', () => S.overlay && placeSugg());

// ---- start page ------------------------------------------------------------
function renderStart() {
  const t = cur();
  const blank = !t || !t.url;
  // What you typed on the start page belongs to the tab you typed it in.
  if (startQ._tab !== S.active || !blank) {
    startQ._tab = S.active; startQ.value = '';
    if (document.activeElement === startQ) startQ.blur();
  }
  document.body.classList.toggle('blank', blank);
  document.body.classList.toggle('private', !!(t && t.private));
  if (blank && !S.blank) send({ t: 'list', kind: 'top' });
  if (blank && !S.blankInit) { S.blankInit = true; send({ t: 'list', kind: 'top' }); }
  S.blank = blank;
  syncBar();
}
// Favourites (pinned by you) come first, then most-visited sites you haven't removed.
S.top = [];
function tileList() {
  const s = S.settings || {};
  const pins = (s.shortcuts || []).map((p) => ({ ...p, pinned: true }));
  const hidden = new Set(s.hidden_tiles || []);
  const taken = new Set(pins.map((p) => hostOf(p.url)));
  const rest = S.top.filter((i) => {
    const h = hostOf(i.url);
    if (!h || hidden.has(h) || taken.has(h)) return false;
    taken.add(h); return true;
  });
  return [...pins, ...rest.slice(0, Math.max(0, 8 - pins.length))];
}
function tileName(i) {
  const host = hostOf(i.url);
  if (i.pinned && i.title) return i.title;
  return (i.title || host).split(/[|\-–—:·]/)[0].trim() || host;
}
// Site icons: /favicon.ico, or the icon the site's pages declare (Discord and others have no
// /favicon.ico). Lurt remembers declared icons, and looks one up on the homepage the first
// time /favicon.ico is missing.
S.icons = {};
S.iconAsked = new Set();
function iconImg(host, fallback, attr = 'src') {
  const known = S.icons[host];
  if (known === '') return fallback;
  const src = known || `https://${host}/favicon.ico`;
  return `<img ${attr}="${esc(src)}" alt="" onerror="iconMiss(${esc(JSON.stringify(host))});this.outerHTML=${esc(JSON.stringify(fallback))}">`;
}
function iconMiss(host) {
  if (S.icons[host] !== undefined || S.iconAsked.has(host)) return;
  S.iconAsked.add(host);
  send({ t: 'findIcon', host });
}
let iconT = 0;
H.icons = (m) => { S.icons = m.map || {}; redrawIcons(); };
H.icon = (m) => {
  S.icons[m.host] = m.url;
  clearTimeout(iconT);
  iconT = setTimeout(redrawIcons, 200);
};
function redrawIcons() { renderTiles(); renderBmBar(); renderTabs(); }
// For long lists (history, suggestions): an icon Lurt already knows, else the letter,
// without asking every listed site for one.
function knownIcon(url, title) {
  const src = S.icons[hostOf(url)];
  const fallback = letter(url, title);
  return src ? `<img src="${esc(src)}" alt="" onerror="this.outerHTML=${esc(JSON.stringify(fallback))}">` : fallback;
}
function renderTiles() {
  S.tiles = tileList();
  $('#tiles').innerHTML = S.tiles.map((i, n) => {
    const host = hostOf(i.url);
    return `<button class="tile${i.pinned ? ' pinned' : ''}" data-i="${n}" data-url="${esc(i.url)}" title="${esc(i.title || i.url)}"><span class="ic glass">` +
      `${iconImg(host, letter(i.url, i.title))}</span><span>${esc(tileName(i))}</span></button>`;
  }).join('');
}
$('#tile-add').innerHTML = icon('plus');
$('#tile-add').onclick = () => editTile(null);
$('#tiles').addEventListener('click', (e) => {
  const b = e.target.closest('.tile[data-url]'); if (b) send({ t: 'open', url: b.dataset.url, newTab: e.ctrlKey });
});
$('#tiles').addEventListener('auxclick', (e) => {
  const b = e.target.closest('.tile[data-url]'); if (b && e.button === 1) send({ t: 'open', url: b.dataset.url, newTab: true });
});
$('#tiles').addEventListener('contextmenu', (e) => {
  const b = e.target.closest('.tile[data-i]'); if (!b) return;
  e.preventDefault();
  const t = S.tiles[+b.dataset.i]; if (!t) return;
  const items = [
    { icon: 'arrow', label: 'Open', run: () => send({ t: 'open', url: t.url }) },
    { icon: 'external', label: 'Open in new tab', run: () => send({ t: 'open', url: t.url, newTab: true }) },
    '-',
  ];
  if (t.pinned) {
    items.push({ icon: 'edit', label: 'Edit', run: () => editTile(t) });
    items.push({ icon: 'trash', label: 'Remove from favourites', danger: true, run: () => removePin(t.url) });
  } else {
    items.push({ icon: 'pin', label: 'Add to favourites', run: () => savePin(null, tileName(t), t.url) });
    items.push({ icon: 'trash', label: 'Remove from start page', danger: true, run: () => hideTile(t.url) });
  }
  showMenu(e.clientX, e.clientY, items);
});
function savePin(origUrl, title, url) {
  const list = [...(S.settings.shortcuts || [])];
  const i = origUrl ? list.findIndex((p) => p.url === origUrl) : -1;
  if (i >= 0) list[i] = { title, url }; else list.push({ title, url });
  // A pinned site shouldn't stay hidden.
  setSetting('hidden_tiles', (S.settings.hidden_tiles || []).filter((h) => h !== hostOf(url)));
  setSetting('shortcuts', list, { rerender: true });
}
function removePin(url) {
  setSetting('shortcuts', (S.settings.shortcuts || []).filter((p) => p.url !== url), { rerender: true });
  toast('Removed from favourites');
}
function hideTile(url) {
  const h = hostOf(url); if (!h) return;
  setSetting('hidden_tiles', [...new Set([...(S.settings.hidden_tiles || []), h])], { rerender: true });
  toast(`Removed ${h}. Bring it back in Settings → Start page`);
}

// ---- right-click menu --------------------------------------------------------
// Lives at the top level so it stacks above the bookmark folder menus (also top level).
const ctx = $('#ctx'); document.body.appendChild(ctx);
function showMenu(x, y, items) {
  ctx._items = items;
  ctx.innerHTML = items.map((it, i) => it === '-' ? '<hr>'
    : `<button data-i="${i}" class="${it.danger ? 'danger' : ''}">${icon(it.icon)}${esc(it.label)}</button>`).join('');
  document.body.classList.add('ctx-open');
  ctx.style.left = Math.max(6, Math.min(x, innerWidth - ctx.offsetWidth - 6)) + 'px';
  ctx.style.top = Math.max(6, Math.min(y, innerHeight - ctx.offsetHeight - 6)) + 'px';
}
function hideMenu() {
  if (!document.body.classList.contains('ctx-open')) return;
  document.body.classList.remove('ctx-open');
  bmSync();
}
ctx.addEventListener('click', (e) => {
  const b = e.target.closest('button[data-i]'); if (!b) return;
  const it = ctx._items[+b.dataset.i]; hideMenu(); it.run();
});
addEventListener('mousedown', (e) => { if (!ctx.contains(e.target)) hideMenu(); }, true);
// Close when you switch to another app, but not on the blink of focus while a website is
// swapped for its snapshot (Lurt hides the page and refocuses the toolbar), which used to
// close right-click menus on the bookmarks bar as soon as they opened.
addEventListener('blur', () => setTimeout(() => { if (!document.hasFocus()) hideMenu(); }, 150));

// ---- add / edit favourite dialog ------------------------------------------------
const dlg = $('#tile-dlg');
function editTile(t) {
  dlg._orig = t ? t.url : null;
  dlg._save = null;
  dlg.innerHTML = `<h3>${t ? 'Edit favourite' : 'Add a favourite'}</h3>` +
    `<label>Address<input class="field" id="td-url" value="${esc(t?.url || '')}" placeholder="youtube.com" spellcheck="false"></label>` +
    `<label>Name<input class="field" id="td-name" value="${esc(t ? tileName(t) : '')}" placeholder="Optional" spellcheck="false"></label>` +
    `<div class="err" id="td-err"></div>` +
    `<div class="btns"><button class="btn" data-d="cancel">Cancel</button><button class="btn primary" data-d="save">Save</button></div>`;
  document.body.classList.add('dlg-open');
  setTimeout(() => $('#td-url').focus(), 30);
}
function closeDlg() { document.body.classList.remove('dlg-open'); bmSync(); }
// A typed address as a full URL, or '' if it isn't one.
function normUrl(u) {
  u = (u || '').trim();
  if (!u) return '';
  if (/^[a-z]:[\\/]/i.test(u)) return 'file:///' + u.replace(/\\/g, '/');
  if (!/^[a-z][a-z0-9+.-]*:/i.test(u)) u = 'https://' + u;
  try {
    const x = new URL(u);
    if (/^https?:$/.test(x.protocol) && !x.hostname.includes('.') && x.hostname !== 'localhost') return '';
    return x.href;
  } catch { return ''; }
}
function saveDlg() {
  let u = $('#td-url').value.trim();
  if (!u) { $('#td-err').textContent = 'Type an address, like youtube.com'; return; }
  u = normUrl(u);
  if (!u) { $('#td-err').textContent = "That doesn't look like a web address"; return; }
  const name = $('#td-name').value.trim() || hostOf(u).split('.')[0].replace(/^./, (c) => c.toUpperCase());
  savePin(dlg._orig, name, u);
  closeDlg();
}
dlg.addEventListener('click', (e) => {
  const d = e.target.closest('[data-d]')?.dataset.d;
  if (d === 'cancel') closeDlg();
  if (d === 'save') (dlg._save || saveDlg)();
});
dlg.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') { e.preventDefault(); (dlg._save || saveDlg)(); }
  if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); closeDlg(); }
});
$('#scrim').addEventListener('mousedown', () => {
  if (document.body.classList.contains('dlg-open')) closeDlg(); else closeSettings();
});

function tickClock() {
  const d = new Date();
  $('#time').textContent = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  $('#date').textContent = d.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' });
}
tickClock();
setInterval(() => { if (S.blank) tickClock(); }, 10000);

// ---- bookmarks bar -----------------------------------------------------------
// Bookmarks and folders in your order. Folders open as menus (subfolders cascade),
// everything can be dragged to reorder or into folders, and links can be dropped on it.
const bar = $('#bmbar');
S.bm = { kids: new Map(), folders: new Map() };
S.bmMenus = [];
H.bar = (m) => {
  const kids = new Map(); const folders = new Map();
  const put = (p, x) => { if (!kids.has(p)) kids.set(p, []); kids.get(p).push(x); };
  for (const f of m.folders) { const x = { k: 'f', id: f.id, title: f.title, pos: f.pos, parent: f.parent }; folders.set(f.id, x); }
  for (const x of folders.values()) put(folders.has(x.parent) ? x.parent : 0, x);
  for (const b of m.items) put(folders.has(b.parent) ? b.parent : 0, { k: 'b', url: b.url, title: b.title, pos: b.pos, parent: b.parent });
  for (const l of kids.values()) l.sort((a, b) => a.pos - b.pos);
  S.bm = { kids, folders };
  if (settingsOpen() && S.setSec === 'bookmarks') renderSettings();
  renderBmBar();
  refreshBmMenus();
};
const bmKids = (p) => S.bm.kids.get(p) || [];
const keyOf = (x) => (x.k === 'f' ? 'f' + x.id : 'b' + x.url);
const elKey = (el) => (el.dataset.k === 'f' ? 'f' + el.dataset.f : 'b' + el.dataset.u);
function bmUrls(id) { const out = []; for (const x of bmKids(id)) x.k === 'b' ? out.push(x.url) : out.push(...bmUrls(x.id)); return out; }
function findMark(u) { for (const l of S.bm.kids.values()) for (const x of l) if (x.k === 'b' && x.url === u) return x; return null; }
const bmName = (x) => x.title || (x.k === 'f' ? 'Folder' : hostOf(x.url) || pretty(x.url));
function bmFav(x) {
  if (x.k === 'f') return icon('folder');
  const h = /^https?:/i.test(x.url) ? hostOf(x.url) : '';
  return h ? iconImg(h, letter(x.url, x.title), 'data-src') : letter(x.url, x.title);
}
function bmEl(x, row) {
  const key = x.k === 'f' ? `data-f="${x.id}"` : `data-u="${esc(x.url)}" title="${esc((x.title ? x.title + '\n' : '') + x.url)}"`;
  return `<button class="bm${x.k === 'f' ? ' fold' : ''}" draggable="true" data-k="${x.k}" ${key}>${bmFav(x)}<span>${esc(bmName(x))}</span>` +
    `${row && x.k === 'f' ? `<i class="chev">${icon('fwd')}</i>` : ''}</button>`;
}
// Favicons load only for what is actually on screen.
function loadFavs(root) {
  for (const i of root.querySelectorAll('img[data-src]')) if (i.offsetParent) { i.src = i.dataset.src; i.removeAttribute('data-src'); }
}

function syncBar() {
  const mode = S.settings?.bookmarks_bar || 'always';
  const on = mode === 'always' || (mode === 'start' && S.blank);
  if (document.body.classList.contains('bm-on') === on) return;
  document.body.classList.toggle('bm-on', on);
  if (on) renderBmBar(); else closeBmMenus();
}
function renderBmBar() {
  if (!document.body.classList.contains('bm-on')) return;
  const list = bmKids(0);
  bar.innerHTML = list.length
    ? list.map((x) => bmEl(x)).join('') + `<button class="bm more" id="bm-more" title="More bookmarks">${icon('fwd')}</button>`
    : '<span class="hint">Press <kbd>Ctrl D</kbd> or the star to add pages here, or drag links onto this bar</span>';
  fitBar();
}
// Whatever doesn't fit goes into the » menu at the end.
function fitBar() {
  const more = $('#bm-more'); bar._hidden = 0;
  if (!more || !bar.offsetParent) return;
  const chips = [...bar.querySelectorAll('.bm:not(.more)')];
  chips.forEach((c) => c.classList.remove('hide'));
  more.classList.remove('hide');
  const limit = bar.clientWidth - 10 - more.offsetWidth - 6;
  let cut = false;
  for (const c of chips) {
    if (!cut && c.offsetLeft + c.offsetWidth > limit) cut = true;
    if (cut) { c.classList.add('hide'); bar._hidden++; }
  }
  more.classList.toggle('hide', !bar._hidden);
  loadFavs(bar);
}
new ResizeObserver(() => fitBar()).observe(bar);
const overflowList = () => (bar._hidden ? bmKids(0).slice(-bar._hidden) : []);

// The page is a native view above us: menus and dialogs over it swap it for a snapshot.
function bmOverlay() { if (!S.blank && !S.overlay) openOverlay('bm-open'); }
function bmSync() {
  setTimeout(() => {
    const c = document.body.classList;
    const busy = S.bmMenus.length || c.contains('ctx-open') || c.contains('dlg-open');
    if (!busy && S.overlay === 'bm-open') closeOverlay(false);
  }, 0);
}

// No folder menus while a bookmark window or the bookmark card is open: they'd cover it.
const bmBusy = () => document.body.classList.contains('dlg-open') || S.overlay === 'star-open';
function openBmMenu(parent, anchor, level, list) {
  if (bmBusy()) return;
  closeBmMenus(level, true);
  const m = document.createElement('div');
  m.className = 'bm-menu glass';
  m.dataset.parent = parent; m._level = level; m._anchor = anchor; m._more = !!list;
  fillBmMenu(m, list || bmKids(parent));
  document.body.appendChild(m);
  const r = anchor.getBoundingClientRect();
  let x = level ? r.right + 2 : r.left; let y = level ? r.top - 5 : r.bottom + 4;
  if (level && x + m.offsetWidth > innerWidth - 6) x = r.left - m.offsetWidth - 2;
  if (!level && list) x = r.right - m.offsetWidth;
  x = Math.max(6, Math.min(x, innerWidth - m.offsetWidth - 6));
  y = Math.max(6, Math.min(y, innerHeight - m.offsetHeight - 6));
  Object.assign(m.style, { left: x + 'px', top: y + 'px' });
  S.bmMenus[level] = m;
  anchor.classList.add('open');
  loadFavs(m);
  bmOverlay();
}
function fillBmMenu(m, items) {
  m.innerHTML = items.length ? items.map((x) => bmEl(x, true)).join('') : '<div class="empty">Empty folder</div>';
}
function closeBmMenus(from = 0, keepOverlay) {
  while (S.bmMenus.length > from) { const m = S.bmMenus.pop(); m._anchor?.classList.remove('open'); m.remove(); }
  if (!keepOverlay) bmSync();
}
function refreshBmMenus() {
  for (let i = 0; i < S.bmMenus.length; i++) {
    const m = S.bmMenus[i]; const p = +m.dataset.parent;
    if (p && !S.bm.folders.has(p)) { closeBmMenus(i); return; }
    fillBmMenu(m, m._more ? overflowList() : bmKids(p));
    loadFavs(m);
  }
}

// Click a bookmark to open it (Ctrl or middle click: new tab); click a folder to open it.
function bmActivate(b, e, level) {
  if (b.classList.contains('more')) {
    if (S.bmMenus[0]?._anchor === b) closeBmMenus(); else openBmMenu(0, b, 0, overflowList());
  } else if (b.dataset.k === 'f') {
    if (S.bmMenus[level]?._anchor === b) closeBmMenus(level); else openBmMenu(+b.dataset.f, b, level);
  } else {
    send({ t: 'open', url: b.dataset.u, newTab: e.ctrlKey || e.button === 1 });
    closeBmMenus();
  }
}
document.addEventListener('click', (e) => {
  const b = e.target.closest('#bmbar .bm, .bm-menu .bm'); if (!b) return;
  const m = b.closest('.bm-menu');
  bmActivate(b, e, m ? m._level + 1 : 0);
});
document.addEventListener('auxclick', (e) => {
  const b = e.target.closest('#bmbar .bm[data-u], .bm-menu .bm[data-u]');
  if (b && e.button === 1) { e.preventDefault(); send({ t: 'open', url: b.dataset.u, newTab: true }); }
});
// Hovering moves between open folders, like a menu bar.
document.addEventListener('mouseover', (e) => {
  if (bmBusy()) return;
  const chip = e.target.closest('#bmbar .bm.fold');
  if (chip && S.bmMenus[0] && S.bmMenus[0]._anchor !== chip) { openBmMenu(+chip.dataset.f, chip, 0); return; }
  const m = e.target.closest('.bm-menu'); if (!m) return;
  const b = e.target.closest('.bm');
  clearTimeout(S.bmHover);
  S.bmHover = setTimeout(() => {
    if (!m.isConnected || S.drag || bmBusy()) return;
    if (b && b.dataset.k === 'f') { if (S.bmMenus[m._level + 1]?._anchor !== b) openBmMenu(+b.dataset.f, b, m._level + 1); }
    else if (b) closeBmMenus(m._level + 1, true);
  }, 220);
});
addEventListener('mousedown', (e) => {
  if (S.bmMenus.length && !e.target.closest('.bm-menu, #ctx, #bmbar .bm')) closeBmMenus();
}, true);

// ---- right-click menus
function bmContext(e) {
  const el = e.target.closest('.bm:not(.more)');
  const menu = e.target.closest('.bm-menu');
  const parent = menu ? +menu.dataset.parent : 0;
  const t = cur(); const page = t && t.url ? t : null;
  let items;
  if (el && el.dataset.k === 'b') {
    const x = findMark(el.dataset.u); if (!x) return;
    items = [
      { icon: 'arrow', label: 'Open', run: () => { send({ t: 'open', url: x.url }); closeBmMenus(); } },
      { icon: 'plus', label: 'Open in new tab', run: () => send({ t: 'open', url: x.url, newTab: true }) },
      { icon: 'edit', label: 'Edit…', run: () => bmEditMark(x) },
      { icon: 'external', label: 'Copy link', run: () => navigator.clipboard?.writeText(x.url).then(() => toast('Link copied')) },
      '-',
      { icon: 'trash', label: 'Delete', danger: true, run: () => send({ t: 'delBookmark', url: x.url }) },
    ];
  } else if (el) {
    const f = S.bm.folders.get(+el.dataset.f); if (!f) return;
    const urls = bmUrls(f.id); const n = urls.length;
    items = [
      ...(n ? [{ icon: 'plus', label: `Open all ${n > 1 ? n + ' ' : ''}in new tabs`, run: () => {
        urls.slice(0, 30).forEach((url) => send({ t: 'open', url, newTab: true }));
        if (n > 30) toast(`Opened the first 30 of ${n}`);
        closeBmMenus();
      } }] : []),
      { icon: 'edit', label: 'Rename…', run: () => bmNameDlg('Rename folder', f.title, (v) => send({ t: 'bmRename', id: f.id, title: v })) },
      { icon: 'folder', label: 'New folder inside…', run: () => bmNameDlg('New folder', '', (v) => send({ t: 'bmFolder', parent: f.id, title: v })) },
      ...(page ? [{ icon: 'star', label: 'Add this page here', run: () => send({ t: 'bmAdd', url: page.url, title: page.title, parent: f.id }) }] : []),
      '-',
      { icon: 'trash', label: n ? `Delete folder and ${n} bookmark${n > 1 ? 's' : ''}` : 'Delete folder', danger: true, run: () => send({ t: 'bmDelFolder', id: f.id }) },
    ];
  } else if (menu && menu._more) {
    return;
  } else {
    items = [
      ...(page ? [{ icon: 'star', label: 'Add this page', run: () => send({ t: 'bmAdd', url: page.url, title: page.title, parent }) }] : []),
      { icon: 'plus', label: 'Add bookmark…', run: () => bmEditMark(null, parent) },
      { icon: 'folder', label: 'New folder…', run: () => bmNameDlg('New folder', '', (v) => send({ t: 'bmFolder', parent, title: v })) },
      ...(menu ? [] : ['-', { icon: 'eye', label: 'Hide bookmarks bar', run: () => setSetting('bookmarks_bar', 'never') }]),
    ];
  }
  bmOverlay();
  showMenu(e.clientX, e.clientY, items);
}
document.addEventListener('contextmenu', (e) => {
  // The bar can redraw between pressing and releasing the button (a website swapping for its
  // snapshot re-renders it), leaving e.target detached: use what's under the mouse now.
  const target = e.target.isConnected ? e.target : document.elementFromPoint(e.clientX, e.clientY);
  if (target?.closest('#bmbar, .bm-menu')) { e.preventDefault(); bmContext({ target, clientX: e.clientX, clientY: e.clientY }); }
});

// ---- add / edit dialogs (the favourite dialog's frame)
function bmDialog(title, fields, save) {
  dlg.innerHTML = `<h3>${esc(title)}</h3>` + fields.map((f) =>
    `<label>${esc(f.label)}<input class="field" data-f="${f.id}" value="${esc(f.value || '')}" placeholder="${esc(f.ph || '')}" spellcheck="false"></label>`).join('') +
    `<div class="err" id="td-err"></div>` +
    `<div class="btns"><button class="btn" data-d="cancel">Cancel</button><button class="btn primary" data-d="save">Save</button></div>`;
  dlg._save = () => {
    const v = {}; for (const i of dlg.querySelectorAll('input[data-f]')) v[i.dataset.f] = i.value.trim();
    if (save(v) !== false) closeDlg();
  };
  bmOverlay();
  document.body.classList.add('dlg-open');
  closeBmMenus(0, true);
  setTimeout(() => { const i = dlg.querySelector('input'); i.focus(); i.select(); }, 30);
}
function bmNameDlg(title, value, done) {
  bmDialog(title, [{ id: 'name', label: 'Name', value, ph: 'New folder' }], (v) => done(v.name || 'New folder'));
}
function bmEditMark(x, parent = 0) {
  bmDialog(x ? 'Edit bookmark' : 'Add bookmark',
    [{ id: 'name', label: 'Name', value: x?.title, ph: 'Optional' }, { id: 'url', label: 'Address', value: x?.url, ph: 'youtube.com' }],
    (v) => {
      const u = normUrl(v.url);
      if (!u) { $('#td-err').textContent = "That doesn't look like a web address"; return false; }
      if (x) send({ t: 'bmEdit', url: x.url, newUrl: u, title: v.name });
      else send({ t: 'bmAdd', url: u, title: v.name || hostOf(u), parent });
    });
}

// ---- drag to reorder, drop into folders, drop links from pages
const ins = document.createElement('i'); ins.id = 'bm-ins'; document.body.appendChild(ins);
document.addEventListener('dragstart', (e) => {
  const b = e.target.closest?.('#bmbar .bm[data-k], .bm-menu .bm[data-k]'); if (!b) return;
  S.drag = b.dataset.k === 'f' ? { k: 'f', id: +b.dataset.f } : { k: 'b', url: b.dataset.u };
  e.dataTransfer.effectAllowed = 'copyMove';
  e.dataTransfer.setData('text/plain', S.drag.url || bmName(S.bm.folders.get(S.drag.id) || {}));
  if (S.drag.url) e.dataTransfer.setData('text/uri-list', S.drag.url);
});
document.addEventListener('dragend', () => { S.drag = null; dropClear(); });
function dropTarget(e) {
  const zone = e.target.closest?.('#bmbar, .bm-menu'); if (!zone) return null;
  const vertical = zone.classList.contains('bm-menu');
  const parent = vertical ? +zone.dataset.parent : 0;
  const el = e.target.closest('.bm:not(.more)');
  if (!el) {
    const last = [...zone.querySelectorAll('.bm:not(.more):not(.hide)')].pop();
    return { parent, el: last, after: true, vertical };
  }
  const r = el.getBoundingClientRect();
  const f = vertical ? (e.clientY - r.top) / r.height : (e.clientX - r.left) / r.width;
  if (el.dataset.k === 'f' && f > 0.25 && f < 0.75) return { into: +el.dataset.f, el };
  return { parent, el, after: f >= 0.5, vertical };
}
function dropShow(t) {
  dropClear();
  if (t.into) { t.el.classList.add('into'); return; }
  if (!t.el) return;
  const r = t.el.getBoundingClientRect();
  Object.assign(ins.style, t.vertical
    ? { left: r.left + 6 + 'px', width: r.width - 12 + 'px', top: (t.after ? r.bottom : r.top) - 1 + 'px', height: '2px' }
    : { left: (t.after ? r.right + 1 : r.left - 3) + 'px', width: '2px', top: r.top + 4 + 'px', height: r.height - 8 + 'px' });
  ins.style.display = 'block';
}
function dropClear() {
  ins.style.display = 'none';
  for (const x of document.querySelectorAll('.bm.into')) x.classList.remove('into');
}
function dropUrl(dt) {
  const u = (dt.getData('text/uri-list') || dt.getData('text/plain') || '').split(/\r?\n/).find((l) => l && !l.startsWith('#'))?.trim();
  return u && /^(https?|file):/i.test(u) ? u : '';
}
function dropTitle(dt) {
  const m = /<a\b[^>]*>([\s\S]*?)<\/a>/i.exec(dt.getData('text/html') || '');
  return m ? m[1].replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim() : '';
}
document.addEventListener('dragover', (e) => {
  const t = dropTarget(e);
  const self = t && S.drag && S.drag.k === 'f' && t.into === S.drag.id;
  if (!t || self) { dropClear(); if (!S.drag && e.dataTransfer.types.includes('Files')) e.preventDefault(); return; }
  e.preventDefault();
  e.dataTransfer.dropEffect = S.drag ? 'move' : 'copy';
  dropShow(t);
});
document.addEventListener('drop', (e) => {
  e.preventDefault(); // never let a dropped file or link replace the browser's own UI
  const t = dropTarget(e); dropClear();
  if (!t) return;
  let item = S.drag; S.drag = null;
  const parent = t.into ?? t.parent;
  if (!item) {
    const url = dropUrl(e.dataTransfer); if (!url) return;
    send({ t: 'bmAdd', url, title: dropTitle(e.dataTransfer) || hostOf(url) || url, parent });
    if (t.into) return;
    item = { k: 'b', url };
  }
  const list = bmKids(parent).filter((x) => keyOf(x) !== keyOf(item));
  let at = list.length;
  if (t.el && !t.into) {
    const i = list.findIndex((x) => keyOf(x) === elKey(t.el));
    if (i >= 0) at = i + (t.after ? 1 : 0);
  }
  list.splice(at, 0, item);
  send({ t: 'bmOrder', parent, items: list.map((x) => (x.k === 'f' ? { f: x.id } : { u: x.url })) });
});

// ---- side panel ------------------------------------------------------------
function togglePanel(p) {
  if (S.panel === p) return closePanel();
  S.panel = p; S.lastPanel = p;
  document.body.classList.add('side-open');
  document.body.classList.remove('panel-bookmarks', 'panel-history', 'panel-trackers');
  document.body.classList.add('panel-' + p);
  const btns = [...document.querySelectorAll('#side-seg button')];
  btns.forEach((b) => b.classList.toggle('on', b.dataset.p === p));
  movePill($('#side-seg'));
  $('#side-body').scrollTop = 0;
  if (p === 'trackers') { S.trkKey = ''; renderShield(); }
  else { $('#side-body').innerHTML = ''; send({ t: 'list', kind: p, q: p === 'history' ? $('#side-q').value : '' }); }
  if (p === 'history') setTimeout(() => $('#side-q').focus(), 60);
}
function closePanel() {
  if (S.hselecting) endPicking();
  S.panel = null;
  document.body.classList.remove('side-open');
}
addEventListener('mousedown', (e) => {
  if (S.panel && !e.target.closest('#side, #menu, #ctx, #tile-dlg, #scrim, #settings, .bm-menu, #shield-pop, #sugg')) closePanel();
}, true);
function movePill(seg) {
  const on = seg.querySelector('button.on'); const pill = seg.querySelector('.seg-pill');
  if (!on || !pill) return;
  pill.style.width = on.offsetWidth + 'px';
  pill.style.transform = `translateX(${on.offsetLeft}px)`;
}
$('#side-seg').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) togglePanel(b.dataset.p); });
$('#side-close').onclick = closePanel;
let histT;
$('#side-q').addEventListener('input', () => {
  clearTimeout(histT);
  histT = setTimeout(() => send({ t: 'list', kind: 'history', q: $('#side-q').value }), 90);
});

function dayLabel(ts) {
  const d = new Date(ts * 1000); const today = new Date(); today.setHours(0, 0, 0, 0);
  const diff = Math.round((today - new Date(d).setHours(0, 0, 0, 0)) / 864e5);
  if (diff <= 0) return 'Today';
  if (diff === 1) return 'Yesterday';
  return d.toLocaleDateString([], { weekday: diff < 7 ? 'long' : undefined, month: 'short', day: 'numeric' });
}
function renderList(kind, items) {
  const body = $('#side-body');
  if (!items.length) {
    body.innerHTML = `<div class="empty">${kind === 'history'
      ? ($('#side-q').value ? 'Nothing matches that.' : 'Pages you visit show up here.')
      : 'No bookmarks yet.<br>Press <kbd>Ctrl D</kbd> on any page to save it.'}</div>`;
    return;
  }
  let html = ''; let day = '';
  for (const i of items) {
    if (kind === 'history') {
      const l = dayLabel(i.time);
      if (l !== day) { day = l; html += `<div class="day">${esc(l)}</div>`; }
    }
    const time = kind === 'history' ? new Date(i.time * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : '';
    const picked = kind === 'history' && S.hsel.has(i.url);
    html += `<div class="row${picked ? ' picked' : ''}" data-url="${esc(i.url)}" title="${esc(i.url)}"><span class="fav">${knownIcon(i.url, i.title)}<i class="tick">${icon('check')}</i></span>` +
      `<div class="txt"><span class="t">${esc(i.title || pretty(i.url))}</span><span class="u">${esc(hostOf(i.url))}</span></div>` +
      `<span class="end">${time}</span><button class="icon sm del" title="Remove">${icon('x')}</button></div>`;
  }
  body.innerHTML = html;
  if (kind === 'history') { S.hitems = items; renderHistTools(); }
}
$('#side-body').addEventListener('click', (e) => {
  const act = e.target.closest('[data-act]');
  if (act) { runAction(act); return; }
  const row = e.target.closest('.row[data-url]'); if (!row) return;
  // Selecting: click (or click the icon) to pick, Shift+click for a range.
  if (S.panel === 'history' && !e.target.closest('.del') && (S.hselecting || e.target.closest('.fav'))) {
    pickRow(row, e.shiftKey); return;
  }
  if (e.target.closest('.del')) {
    send(S.panel === 'history' ? { t: 'delHistory', url: row.dataset.url, q: $('#side-q').value } : { t: 'delBookmark', url: row.dataset.url });
    return;
  }
  send({ t: 'open', url: row.dataset.url, newTab: e.ctrlKey });
});
$('#side-body').addEventListener('auxclick', (e) => {
  const row = e.target.closest('.row[data-url]');
  if (row && e.button === 1) send({ t: 'open', url: row.dataset.url, newTab: true });
});

// ---- history: select & delete, clear a time range, auto-delete
S.hsel = new Set(); S.hselecting = false; S.hitems = [];
const RANGES = [['Last hour', 3600], ['Last 24 hours', 86400], ['Last 7 days', 7 * 86400], ['Last 4 weeks', 28 * 86400], ['All time', 0]];
const KEEP = [[0, 'Forever'], [1, '1 day'], [7, '1 week'], [30, '1 month'], [90, '3 months'], [365, '1 year']];
const keepLabel = (d) => (KEEP.find(([v]) => v === d)?.[1] || `${d} days`);
function renderHistTools() {
  const n = S.hsel.size; const el = $('#hist-tools');
  el.innerHTML = S.hselecting
    ? `<span class="cnt">${n ? `${n} selected` : 'Pick pages to delete'}</span>` +
      `<button class="btn sm" data-h="all">${n && n === S.hitems.length ? 'None' : 'All'}</button>` +
      `<button class="btn sm danger" data-h="del" ${n ? '' : 'disabled'}>${icon('trash')}Delete</button>` +
      `<button class="btn sm" data-h="done">Done</button>`
    : `<button class="btn sm" data-h="select">${icon('check')}Select</button>` +
      `<button class="btn sm" data-h="clear">${icon('trash')}Clear…</button>` +
      `<button class="btn sm ghost" data-h="auto" title="Delete old history automatically">${icon('clock')}Auto-delete: ${esc(keepLabel(S.settings?.history_days | 0))}</button>`;
}
function pickRow(row, range) {
  const u = row.dataset.url;
  if (!S.hselecting) S.hselecting = true;
  if (range && S.hlast) {
    const urls = S.hitems.map((i) => i.url); const a = urls.indexOf(S.hlast); const b = urls.indexOf(u);
    if (a >= 0 && b >= 0) for (const x of urls.slice(Math.min(a, b), Math.max(a, b) + 1)) S.hsel.add(x);
  } else if (S.hsel.has(u)) S.hsel.delete(u); else S.hsel.add(u);
  S.hlast = u;
  for (const r of document.querySelectorAll('#side-body .row[data-url]')) r.classList.toggle('picked', S.hsel.has(r.dataset.url));
  document.body.classList.toggle('hist-picking', S.hselecting);
  renderHistTools();
}
function endPicking() {
  S.hselecting = false; S.hsel.clear(); S.hlast = null;
  document.body.classList.remove('hist-picking');
  for (const r of document.querySelectorAll('#side-body .row.picked')) r.classList.remove('picked');
  renderHistTools();
}
$('#hist-tools').addEventListener('click', (e) => {
  const b = e.target.closest('[data-h]'); if (!b) return;
  const h = b.dataset.h;
  if (h === 'select') { S.hselecting = true; document.body.classList.add('hist-picking'); renderHistTools(); }
  else if (h === 'done') endPicking();
  else if (h === 'all') {
    if (S.hsel.size === S.hitems.length) S.hsel.clear(); else for (const i of S.hitems) S.hsel.add(i.url);
    for (const r of document.querySelectorAll('#side-body .row[data-url]')) r.classList.toggle('picked', S.hsel.has(r.dataset.url));
    renderHistTools();
  } else if (h === 'del') {
    send({ t: 'delHistoryMany', urls: [...S.hsel], q: $('#side-q').value });
    endPicking();
  } else if (h === 'clear') {
    const r = b.getBoundingClientRect();
    showMenu(r.left, r.bottom + 4, RANGES.map(([label, secs]) => ({
      icon: secs ? 'clock' : 'trash', label: `Delete ${label.toLowerCase()}`, danger: !secs,
      run: () => send({ t: 'clearHistoryRange', secs, q: $('#side-q').value }),
    })));
  } else if (h === 'auto') {
    const r = b.getBoundingClientRect();
    const d = S.settings.history_days | 0;
    showMenu(r.left, r.bottom + 4, [
      ...KEEP.map(([v, l]) => ({ icon: v === d ? 'check' : 'clock', label: v ? `Keep ${l}` : 'Keep forever', run: () => { setSetting('history_days', v); renderHistTools(); } })),
      '-',
      { icon: 'edit', label: 'Custom…', run: () => openSettings('privacy') },
    ]);
  }
});

// Destructive buttons ask for a second click instead of a dialog.
function runAction(btn) {
  if (btn.classList.contains('danger') && !btn.classList.contains('armed')) {
    btn.classList.add('armed');
    btn._label = btn.innerHTML;
    btn.innerHTML = 'Click again to confirm';
    setTimeout(() => { if (btn.isConnected) { btn.classList.remove('armed'); btn.innerHTML = btn._label; } }, 3000);
    return;
  }
  btn.classList.remove('armed');
  if (btn._label) btn.innerHTML = btn._label;
  if (btn.dataset.act === 'newPrivate' && settingsOpen()) closeSettings();
  send({ t: btn.dataset.act });
}

// ---- settings --------------------------------------------------------------
// A centered window over a blurred view of the browser, with a section list on the left.
const ACCENTS = ['#8ab4ff', '#a78bfa', '#f472b6', '#fb7185', '#fb923c', '#facc15', '#4ade80', '#2dd4bf', '#e2e8f0'];
const PERM = [['ask', 'Ask'], ['block', 'Block']];
const DL_ACTIONS = [['allow', 'Allow'], ['ask', 'Ask where to save'], ['open', 'Open when finished'], ['block', 'Block']];
const DL_HINT = { allow: 'Downloads normally', ask: 'Asks where to save it', open: 'Opens the file when it finishes', block: 'Never downloaded' };
const DL_SUGG = ['exe', 'msi', 'zip', 'rar', '7z', 'pdf', 'apk', 'iso', 'torrent', 'mp3', 'mp4', 'jpg', 'png', 'docx', 'xlsx', 'bat', 'cmd', 'ps1', 'js', 'vbs', 'scr', 'jar', 'dmg'];
const DL_PROGRAMS = ['exe', 'msi', 'bat', 'cmd', 'scr', 'ps1', 'vbs', 'jar', 'com', 'pif', 'hta'];
// Filter lists: [id, name, what it does].
const LISTS = [
  ['easylist', 'EasyList', 'Ads'],
  ['easyprivacy', 'EasyPrivacy', 'Trackers and analytics'],
  ['peterlowe', 'Peter Lowe\'s list', 'Ad and tracking servers'],
  ['ublock', 'uBlock filters', 'More ads and trackers, plus fixes for sites the others break'],
  ['badware', 'Malware & scams', 'Fake download buttons, scam and malware sites'],
  ['cookies', 'Cookie banners', 'Hides "accept cookies" pop-ups'],
  ['annoyances', 'Annoyances', 'Newsletter pop-ups, chat boxes, social widgets'],
  ['miners', 'Crypto miners', 'Scripts that mine cryptocurrency on your computer'],
];
// Protection levels: what each sets. Anything else shows as Custom.
const LEVELS = {
  standard: { adblock: true, filter_lists: ['easylist', 'easyprivacy', 'peterlowe', 'ublock', 'badware'], tracking: 'balanced', gpc: true, strip_tracking: true, block_pings: true, https_upgrade: true, fingerprint: false, webrtc_protect: false },
  strict: { adblock: true, filter_lists: LISTS.map(([id]) => id), tracking: 'strict', gpc: true, strip_tracking: true, block_pings: true, https_upgrade: true, fingerprint: true, webrtc_protect: true },
};
function privLevel(s) {
  const same = (a, b) => Array.isArray(a) ? a.length === b.length && a.every((x) => b.includes(x)) : a === b;
  return Object.keys(LEVELS).find((l) => Object.entries(LEVELS[l]).every(([k, v]) => same(v, s[k]))) || 'custom';
}
const F = {
  adblock: { k: 'adblock', label: 'Block ads & trackers', desc: () => S.shieldReady ? `${(S.settings.filter_lists || []).length} filter lists on, pick them below` : 'Downloading filter lists...', type: 'toggle' },
  tracking: { k: 'tracking', label: 'Tracking prevention', desc: 'Built into the engine; Strict may break some sites', type: 'select', opts: [['strict', 'Strict'], ['balanced', 'Balanced'], ['basic', 'Basic'], ['off', 'Off']] },
  updateLists: { type: 'buttons', items: [['updateLists', 'refresh', 'Update filter lists']] },
};
const SECTIONS = [
  { id: 'account', icon: 'user', title: 'Account', groups: [
    ['Google account', [{ type: 'account', label: 'Google account', kw: 'profile sign in sign out gmail switch' }]],
    ['What Lurt saves on this PC', [{ type: 'saved', label: 'What gets saved', kw: 'data storage profile folder cookies passwords' }]],
    ['Lurt profile', [{ type: 'profileSync', label: 'Back up Lurt settings', kw: 'sync backup export import restore settings account' }]],
    ['Start over', [{ type: 'wipe', label: 'Delete all browser data', kw: 'clear reset erase wipe everything passwords bookmarks history' }]],
  ] },
  { id: 'appearance', icon: 'palette', title: 'Appearance', groups: [
    ['Page zoom', [{ type: 'zoom', label: 'Page zoom', desc: 'How big websites are shown. Ctrl + scroll, Ctrl + / - and Ctrl 0 work too', kw: 'scale size text bigger smaller magnify' }]],
    ['Theme', [
      { k: 'theme', label: 'Mode', type: 'seg', opts: [['system', 'System'], ['light', 'Light'], ['dark', 'Dark']] },
      { k: 'accent', label: 'Accent color', type: 'color' },
    ]],
    ['Glass', [
      { k: 'backdrop', label: 'Backdrop', type: 'seg', opts: [['blur', 'Blur'], ['acrylic', 'Acrylic'], ['mica', 'Mica'], ['none', 'Solid']], hint: backdropHint },
      { k: 'tint', label: 'Background opacity', desc: 'Lower shows more of what is behind the window', type: 'range', min: 0, max: 100, unit: '%' },
      { k: 'window_opacity', label: 'Window opacity', desc: 'Fades the whole window, websites included', type: 'range', min: 20, max: 100, unit: '%' },
    ]],
    ['Layout', [
      { k: 'radius', label: 'Roundness', type: 'range', min: 0, max: 24, unit: 'px' },
      { k: 'compact', label: 'Compact layout', desc: 'Tabs and address bar share one row', type: 'toggle' },
      { k: 'rounded_pages', label: 'Rounded page corners', type: 'toggle' },
      { k: 'reduce_motion', label: 'Reduce motion', desc: 'Turn off animations', type: 'toggle' },
    ]],
  ] },
  { id: 'start', icon: 'grid', title: 'Start page', groups: [
    ['Favourites', [{ type: 'shortcuts' }]],
    ['Removed sites', [{ type: 'hidden' }]],
    ['Show', [{ k: 'show_clock', label: 'Clock and date', type: 'toggle' }]],
  ] },
  { id: 'bookmarks', icon: 'star', title: 'Bookmarks', groups: [
    ['Bookmarks bar', [
      { k: 'bookmarks_bar', label: 'Show the bookmarks bar', desc: 'Ctrl+Shift+B shows or hides it', type: 'seg', opts: [['always', 'Always'], ['start', 'Start page only'], ['never', 'Hidden']], kw: 'toolbar favorites folders' },
    ]],
    ['Your bookmarks', [
      { type: 'bmStats', label: 'Your bookmarks', kw: 'manage folders count' },
      { type: 'links', label: 'Show all bookmarks', kw: 'panel list manage', items: [['panel:bookmarks', 'star', 'Show all bookmarks'], ['sec:data', 'download', 'Import bookmarks']] },
    ]],
  ] },
  { id: 'browsing', icon: 'globe', title: 'Browsing', groups: [
    ['Search', [{ k: 'search', label: 'Search engine', desc: 'Used when you type words in the address bar', type: 'select', opts: Object.entries(ENGINES) }]],
    ['Startup', [
      { k: 'restore_session', label: 'Reopen last session\'s tabs', type: 'toggle' },
      { k: 'homepage', label: 'Home button opens', desc: 'Leave empty for the start page', type: 'text', placeholder: 'Start page' },
    ]],
  ] },
  { id: 'privacy', icon: 'shield', title: 'Privacy & trackers', groups: [
    ['Protection level', [{ type: 'privLevel', label: 'Protection level', kw: 'standard strict custom privacy mode' }]],
    ['Blocking', [F.adblock, { type: 'lists', label: 'Filter lists', kw: 'easylist ublock cookie banners annoyances malware miners popups' }, { type: 'allow' }, F.updateLists]],
    ['Tracking', [
      F.tracking,
      { k: 'strip_tracking', label: 'Clean tracking from links', desc: 'Removes utm_, fbclid, gclid and other tracking tags from addresses you open', type: 'toggle' },
      { k: 'gpc', label: 'Ask sites not to sell or share my data', desc: 'Sends Global Privacy Control and Do Not Track. Reloads open tabs', type: 'toggle', kw: 'gpc dnt do not track' },
      { k: 'block_pings', label: 'Block link-click pings', desc: 'Stops pages reporting which links you click (hyperlink auditing)', type: 'toggle', kw: 'ping hyperlink auditing' },
    ]],
    ['Fingerprinting', [
      { k: 'fingerprint', label: 'Resist fingerprinting', desc: 'Hides your graphics card and CPU details and adds invisible noise to canvas and audio reads, different for every site. A few sites may act up. Reloads open tabs', type: 'toggle', kw: 'canvas webgl audio hardware' },
    ]],
    ['Connection', [
      { k: 'https_upgrade', label: 'Use HTTPS when possible', desc: 'Loads http:// sites securely, back to HTTP only if a site doesn\'t support it', type: 'toggle', kw: 'https only secure ssl upgrade' },
      { k: 'doh', label: 'Secure DNS', desc: 'Encrypts the lookups that turn site names into addresses, so your network can\'t see them. Applies after restarting Lurt', type: 'select', opts: [['off', 'Off (use Windows)'], ['cloudflare', 'Cloudflare 1.1.1.1'], ['quad9', 'Quad9'], ['mullvad', 'Mullvad'], ['google', 'Google']], kw: 'doh dns over https' },
      { k: 'webrtc_protect', label: 'Protect my IP address in calls', desc: 'WebRTC uses only your main connection: no local addresses, and it stays behind a VPN. Applies after restarting Lurt', type: 'toggle', kw: 'webrtc ip leak vpn' },
    ]],
    ['Site permissions', [
      { k: 'perm_location', label: 'Location', type: 'select', opts: PERM, kw: 'permission gps' },
      { k: 'perm_camera', label: 'Camera', type: 'select', opts: PERM, kw: 'permission webcam' },
      { k: 'perm_mic', label: 'Microphone', type: 'select', opts: PERM, kw: 'permission audio' },
      { k: 'perm_notifications', label: 'Notifications', type: 'select', opts: PERM, kw: 'permission push' },
    ]],
    ['Private browsing', [
      { type: 'buttons', items: [['newPrivate', 'mask', 'New private tab  (Ctrl+Shift+N)']] },
      { k: 'always_private', label: 'Always open private tabs', desc: 'New tabs keep no history and their cookies go when the last private tab closes. Sites you\'re signed in to will ask you to sign in again', type: 'toggle', kw: 'incognito inprivate' },
    ]],
    ['Delete when Lurt closes', [
      { k: 'exit_clear_history', label: 'History', type: 'toggle', kw: 'clear on exit' },
      { k: 'exit_clear_cookies', label: 'Cookies, site data and cache', desc: 'Signs you out of sites. Passwords stay', type: 'toggle', kw: 'clear on exit' },
    ]],
    ['History', [{ type: 'histKeep', label: 'Delete history automatically', desc: 'Pages older than this are removed, checked every hour' }]],
    ['Clear browsing data', [
      { type: 'buttons', items: [['clearHistory', 'trash', 'Clear history', true], ['clearData', 'trash', 'Clear cookies & cache', true]] },
      { type: 'wipe', label: 'Delete all browser data', kw: 'clear reset erase wipe everything passwords bookmarks history' },
    ]],
  ] },
  { id: 'downloads', icon: 'download', title: 'Downloads', groups: [
    ['Where downloads go', [
      { type: 'dlDir', label: 'Download folder', kw: 'location path save folder directory' },
      { k: 'download_ask', label: 'Ask where to save each file', desc: 'Shows a Save as window for every download', type: 'toggle', kw: 'save as location' },
    ]],
    ['File types', [{ type: 'dlRules', label: 'File type rules', kw: 'extension block allow exe zip pdf open automatically type' }]],
  ] },
  { id: 'data', icon: 'database', title: 'Your data', groups: [
    ['Import from browsers on this PC', [{ type: 'importPC' }]],
    ['Import from your Google account', [{ type: 'importGoogle' }]],
    ['Passwords & forms', [
      { k: 'save_passwords', label: 'Offer to save passwords', desc: "Saved in Lurt's profile on this PC", type: 'toggle' },
      { k: 'autofill', label: 'Autofill forms', desc: 'Addresses and other details you type', type: 'toggle' },
    ]],
  ] },
  { id: 'performance', icon: 'bolt', title: 'Performance', groups: [
    ['Memory', [
      { k: 'sleep_minutes', label: 'Sleep background tabs', desc: 'Frees memory; the tab reloads when you return', type: 'select', num: true, opts: [[0, 'Never'], [5, 'After 5 min'], [15, 'After 15 min'], [30, 'After 30 min'], [60, 'After 1 hour']] },
      { k: 'share_processes', label: 'Fewer processes', desc: 'Embedded frames from other sites (ads, payment and video embeds) share their page\'s process instead of one each. Uses about half the memory, with less isolation between sites. Applies after restarting Lurt.', type: 'toggle', kw: 'memory ram subframe processes site isolation' },
    ]],
  ] },
  { id: 'mods', icon: 'code', title: 'Mods', groups: [
    ['Browser style', [{ k: 'ui_css', label: 'Custom CSS', desc: 'Applied live. Try: :root { --r: 22px }', type: 'code' }]],
    ['Mods folder', [
      { type: 'note', text: () => `Put .css files in mods\\ui to theme the browser, and <domain>.css / .js in mods\\sites to change websites. Folder: ${S.modsDir}` },
      { type: 'buttons', items: [['openMods', 'folder', 'Open mods folder'], ['reloadMods', 'refresh', 'Reload mods']] },
    ]],
  ] },
  { id: 'about', icon: 'info', title: 'About', groups: [
    ['', [{ type: 'about' }]],
    ['Keyboard shortcuts', [{ type: 'keys' }]],
  ] },
];
S.setSec = 'appearance';
const settingsOpen = () => S.overlay === 'settings-open';
function openSettings(sec) {
  if (sec) S.setSec = sec;
  hideMenu();
  if (S.overlay === 'shield-open') closeShield(false);
  if (S.overlay === 'omni-open') box.blur();
  closeVol(false);
  if (!settingsOpen()) openOverlay('settings-open');
  renderSetNav();
  renderSettings(true);
  if (S.setSec === 'data') send({ t: 'importSources' });
  if (S.setSec === 'account') { send({ t: 'account' }); send({ t: 'dataInfo' }); }
}
function closeSettings() { if (settingsOpen()) closeOverlay(false); }
function renderSetNav() {
  const searching = !!$('#set-q').value.trim();
  $('#set-tabs').innerHTML = SECTIONS.map((s) =>
    `<button data-sec="${s.id}" class="${s.id === S.setSec && !searching ? 'on' : ''}">${icon(s.icon)}${esc(s.title)}</button>`).join('');
}
$('#set-tabs').addEventListener('click', (e) => {
  const b = e.target.closest('[data-sec]'); if (!b) return;
  if ($('#set-q').value) { $('#set-q').value = ''; document.body.classList.remove('set-searching'); }
  else if (b.dataset.sec === S.setSec) return;
  openSettings(b.dataset.sec);
});
$('#set-close').innerHTML = icon('x');
$('#set-close').onclick = () => closeSettings();
function backdropHint() {
  const want = S.settings.backdrop;
  if (want === 'mica' && S.backdrop !== 'mica') return 'Mica needs Windows 11. Using Blur instead.';
  if (want === 'acrylic') return 'Acrylic can stutter while dragging the window on Windows 10.';
  return '';
}
const val = (f) => (typeof f === 'function' ? f() : f) || '';
function groupsHtml(groups, s) {
  let html = '';
  groups.forEach(([title, items], gi) => {
    const inner = items.map((it) => field(it, s)).join('');
    if (!inner) return;
    html += `<div class="grp" data-g="${gi}">${title ? `<h4>${esc(title)}</h4>` : ''}<div class="card">${inner}</div></div>`;
  });
  return html;
}

// ---- settings search ---------------------------------------------------------------
// Every setting is indexed by its name, description, section, group, options and a few
// extra keywords. Picking a result opens its section and highlights it.
function settingsIndex() {
  const out = [];
  for (const sec of SECTIONS) {
    sec.groups.forEach(([gt, items], gi) => {
      items.forEach((it) => {
        const label = it.label || (it.type === 'buttons' ? it.items.map((x) => x[2]).join(', ') : gt);
        if (!label) return;
        const opts = (it.opts || []).map((o) => o[1]).join(' ');
        const btns = (it.items || []).map((x) => x[2]).join(' ');
        const text = [label, val(it.desc), gt, sec.title, opts, btns, it.kw || '', it.k || ''].join(' ').toLowerCase();
        out.push({ sec, gi, k: it.k, label, desc: val(it.desc), group: gt, text });
      });
    });
  }
  return out;
}
const SEARCH_KW = { appearance: 'look theme dark light colors', privacy: 'security ads blocker adblock', data: 'import export chrome google passwords', performance: 'speed ram sleep', mods: 'css themes customize', about: 'version help shortcuts keys' };
function searchSettings(q) {
  const words = q.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  const seen = new Set();
  return settingsIndex()
    .map((r) => ({ ...r, text: r.text + ' ' + (SEARCH_KW[r.sec.id] || '') }))
    .filter((r) => words.every((w) => r.text.includes(w)))
    .map((r) => ({ ...r, score: words.reduce((n, w) => n + (r.label.toLowerCase().includes(w) ? 2 : 0), 0) }))
    .sort((a, b) => b.score - a.score)
    .filter((r) => { const key = r.sec.id + r.gi + r.label; if (seen.has(key)) return false; seen.add(key); return true; })
    .slice(0, 30);
}
function mark(text, q) {
  let html = esc(text);
  for (const w of q.toLowerCase().split(/\s+/).filter((x) => x.length > 1)) {
    html = html.replace(new RegExp(`(${w.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'ig'), '<mark>$1</mark>');
  }
  return html;
}
function renderSearch() {
  const q = $('#set-q').value.trim();
  document.body.classList.toggle('set-searching', !!q);
  renderSetNav();
  if (!q) { renderSettings(true); return; }
  S.setHits = searchSettings(q); S.setPick = 0;
  $('#set-h').textContent = 'Search';
  $('#set-body').innerHTML = S.setHits.length
    ? `<div class="hits">` + S.setHits.map((r, i) =>
      `<button class="hit${i === 0 ? ' on' : ''}" data-i="${i}">${icon(r.sec.icon)}<div class="txt"><b>${mark(r.label, q)}</b>` +
      `<small>${esc(r.sec.title)}${r.group && r.group !== r.label ? ' › ' + esc(r.group) : ''}${r.desc ? ' · ' + mark(r.desc, q) : ''}</small></div>${icon('fwd')}</button>`).join('') + '</div>'
    : `<div class="empty">No settings match “${esc(q)}”.</div>`;
  $('#set-body').scrollTop = 0;
}
function goToSetting(r) {
  if (r.sec.id === 'downloads') S.dlTab = 'settings';
  $('#set-q').value = '';
  document.body.classList.remove('set-searching');
  openSettings(r.sec.id);
  requestAnimationFrame(() => {
    const body = $('#set-body');
    const el = (r.k && body.querySelector(`[data-k="${r.k}"]`)?.closest('.set')) || body.querySelector(`.grp[data-g="${r.gi}"]`);
    if (!el) return;
    el.scrollIntoView({ block: 'center', behavior: document.documentElement.hasAttribute('data-calm') ? 'auto' : 'smooth' });
    el.classList.remove('flash'); void el.offsetWidth; el.classList.add('flash');
  });
}
$('#set-search .ic').innerHTML = icon('search');
$('#set-q').addEventListener('input', renderSearch);
$('#set-q').addEventListener('keydown', (e) => {
  const hits = S.setHits || [];
  if (e.key === 'Escape' && $('#set-q').value) { e.preventDefault(); e.stopPropagation(); $('#set-q').value = ''; renderSearch(); return; }
  if (!$('#set-q').value.trim() || !hits.length) return;
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault();
    S.setPick = (S.setPick + (e.key === 'ArrowDown' ? 1 : -1) + hits.length) % hits.length;
    document.querySelectorAll('#set-body .hit').forEach((b, i) => b.classList.toggle('on', i === S.setPick));
    document.querySelector('#set-body .hit.on')?.scrollIntoView({ block: 'nearest' });
  }
  if (e.key === 'Enter') { e.preventDefault(); goToSetting(hits[S.setPick] || hits[0]); }
});
$('#set-body').addEventListener('click', (e) => {
  const h = e.target.closest('.hit[data-i]'); if (h) { goToSetting(S.setHits[+h.dataset.i]); return; }
  const g = e.target.closest('[data-go]'); if (!g) return;
  const [kind, where] = g.dataset.go.split(':');
  if (kind === 'sec') openSettings(where);
  else { closeSettings(); if (S.panel !== where) togglePanel(where); }
});
// Re-render whichever settings views are open (the window and/or the Trackers panel).
function renderSettings(top) {
  const s = S.settings; if (!s) return;
  if (settingsOpen() && $('#set-q').value.trim()) { if (S.panel === 'trackers') renderTrackers(); return; }
  if (settingsOpen()) {
    const sec = SECTIONS.find((x) => x.id === S.setSec) || SECTIONS[0];
    $('#set-h').textContent = sec.title;
    const body = $('#set-body');
    const y = top ? 0 : body.scrollTop;
    let html = sec.id === 'downloads' ? downloadsPage(sec, s) : groupsHtml(sec.groups, s);
    if (sec.id === 'data') {
      if (S.importing) html = `<div class="status"><i class="spin"></i>Importing from ${esc(S.importing)}…</div>` + html;
      else if (S.importMsg) html = `<div class="status">${icon('info')}${esc(S.importMsg)}</div>` + html;
    }
    body.innerHTML = html;
    body.scrollTop = y;
    body.querySelectorAll('.seg').forEach(movePill);
  }
  if (S.panel === 'trackers') renderTrackers();
}
function renderTrackers() {
  const s = S.settings; if (!s || S.panel !== 'trackers') return;
  if ($('#side').contains(document.activeElement) && document.activeElement.tagName === 'SELECT') return;
  const t = cur(); const host = hostOf(t?.url || '');
  const on = s.adblock && !S.allowed;
  const info = S.shieldInfo && S.shieldInfo.id === S.active ? S.shieldInfo : null;
  const n = on && t ? t.blocked : 0;
  let h = `<div class="trk-top${on ? '' : ' off'}">${icon(on ? 'shield' : 'shieldOff')}<div><strong>${n}</strong><span>` +
    (!host ? 'Open a website to see what gets blocked' : !s.adblock ? 'Blocking is off' : S.allowed ? `Paused on ${esc(host)}` : `blocked on ${esc(host)}`) + `</span></div></div>`;
  let card = field(F.adblock, s);
  if (host && s.adblock) card += `<label class="set"><div class="lbl"><b>Block on this site</b><small>${S.allowed ? 'Paused, ads will show here' : 'Turn off if the site breaks'}</small></div>` +
    `<span class="switch"><input type="checkbox" data-a="site" ${on ? 'checked' : ''}><i></i></span></label>`;
  h += `<div class="card">${card}</div>`;
  if (on && host) {
    h += `<div class="grp"><h4>Blocked on this page</h4>`;
    if (!info) h += `<div class="hint" style="padding:0 10px">Loading...</div>`;
    else if (!info.items.length) h += `<div class="hint" style="padding:0 10px">Nothing blocked yet. Trackers that load later show up here.</div>`;
    else h += `<div class="card">${blockedList(info)}</div>`;
    h += `</div>`;
  }
  h += groupsHtml([['Paused sites', [{ type: 'allow' }]], ['Engine', [F.tracking, F.updateLists]]], s);
  h += `<div class="btns" style="padding:14px 4px"><button class="btn" data-settings="privacy">${icon('gear')}All privacy settings</button></div>`;
  const body = $('#side-body'); const y = body.scrollTop;
  body.innerHTML = h; body.scrollTop = y;
}
const KEYS = [['New tab', 'Ctrl T'], ['New private tab', 'Ctrl Shift N'], ['Close tab', 'Ctrl W'], ['Reopen closed tab', 'Ctrl Shift T'], ['Address bar', 'Ctrl L'],
  ['Next tab', 'Ctrl Tab'], ['Bookmark page', 'Ctrl D'], ['History', 'Ctrl H'], ['Bookmarks bar', 'Ctrl Shift B'], ['All bookmarks', 'Ctrl Shift O'], ['Downloads', 'Ctrl J'], ['Settings', 'Ctrl ,'], ['Reload', 'F5'], ['Zoom in or out', 'Ctrl Scroll'], ['Reset zoom', 'Ctrl 0']];
function field(it, s) {
  const label = `<div class="lbl"><b>${esc(it.label)}</b>${it.desc ? `<small>${esc(val(it.desc))}</small>` : ''}</div>`;
  switch (it.type) {
    case 'toggle':
      return `<label class="set">${label}<span class="switch"><input type="checkbox" data-k="${it.k}" ${s[it.k] ? 'checked' : ''}><i></i></span></label>`;
    case 'seg': {
      const hint = it.hint ? val(it.hint) : '';
      return `<div class="set col">${label}<div class="seg" data-k="${it.k}"><i class="seg-pill"></i>` +
        it.opts.map(([v, l]) => `<button data-v="${v}" class="${s[it.k] === v ? 'on' : ''}">${l}</button>`).join('') +
        `</div>${hint ? `<div class="hint">${esc(hint)}</div>` : ''}</div>`;
    }
    case 'zoom':
      return `<div class="set">${label}<div class="zoom"><button class="btn sm" data-zoom="-1" title="Zoom out">−</button><span class="val">${s.page_zoom}%</span>` +
        `<button class="btn sm" data-zoom="1" title="Zoom in">+</button><button class="btn sm" data-zoom="0" ${s.page_zoom === 100 ? 'disabled' : ''}>Reset</button></div></div>`;
    case 'range': {
      const p = ((s[it.k] - it.min) / (it.max - it.min)) * 100;
      return `<div class="set col"><div style="display:flex">${label}<span class="val">${s[it.k]}${it.unit}</span></div>` +
        `<input type="range" data-k="${it.k}" min="${it.min}" max="${it.max}" value="${s[it.k]}" style="--p:${p}%" data-unit="${it.unit}"></div>`;
    }
    case 'select':
      return `<div class="set">${label}<select class="field" style="width:auto;min-width:150px" data-k="${it.k}" ${it.num ? 'data-num' : ''}>` +
        it.opts.map(([v, l]) => `<option value="${v}" ${String(s[it.k]) === String(v) ? 'selected' : ''}>${esc(l)}</option>`).join('') + '</select></div>';
    case 'dlDir': {
      const custom = !!s.download_dir;
      return `<div class="set">${`<div class="lbl"><b>${esc(it.label)}</b><small class="path">${esc(S.dlDir || s.download_dir || 'Downloads')}</small></div>`}` +
        `<div class="row-btns">${custom ? '<button class="btn" data-dir-reset>Use Downloads</button>' : ''}<button class="btn" data-act="dlPickDir">${icon('folder')}Change…</button></div></div>`;
    }
    case 'dlRules': {
      const rules = s.download_rules || [];
      const opts = (v) => DL_ACTIONS.map(([a, l]) => `<option value="${a}" ${a === v ? 'selected' : ''}>${l}</option>`).join('');
      return `<div class="rules">` +
        (rules.length ? rules.map((r, i) => `<div class="set rule"><span class="ext">.${esc(r.ext)}</span><div class="lbl"><small>${esc(DL_HINT[r.action] || '')}</small></div>` +
          `<select class="field" data-rule="${i}">${opts(r.action)}</select><button class="icon sm" data-rule-del="${i}" title="Remove">${icon('x')}</button></div>`).join('')
          : '<div class="set"><div class="lbl"><small>No rules yet: every file type downloads normally.</small></div></div>') +
        `<div class="set rule add"><input class="field" id="rule-ext" placeholder="File type, like exe or zip" list="rule-sugg" spellcheck="false">` +
        `<select class="field" id="rule-act">${opts('block')}</select><button class="btn" data-rule-add>Add</button></div>` +
        `<datalist id="rule-sugg">${DL_SUGG.map((x) => `<option value="${x}">`).join('')}</datalist>` +
        `<div class="btns"><button class="btn" data-rule-preset="programs">${icon('shield')}Block programs and scripts</button></div></div>`;
    }
    case 'privLevel': {
      const cur = privLevel(s);
      const card = (id, title, desc) => `<button class="lvl ${cur === id ? 'on' : ''}" data-level="${id}"><b>${title}</b><small>${desc}</small></button>`;
      return `<div class="lvls">${card('standard', 'Standard', 'Blocks ads, trackers and malware. Sites work as usual')}` +
        card('strict', 'Strict', 'Every list, strict tracking prevention, fingerprint and IP protection. Some sites may break') +
        `<div class="lvl ${cur === 'custom' ? 'on' : ''} static"><b>Custom</b><small>Your own mix of the settings below</small></div></div>`;
    }
    case 'lists': {
      const on = s.filter_lists || [];
      return `<div class="lists">` + LISTS.map(([id, name, what]) =>
        `<label class="set sub">${`<div class="lbl"><b>${esc(name)}</b><small>${esc(what)}</small></div>`}<span class="switch"><input type="checkbox" data-list="${id}" ${on.includes(id) ? 'checked' : ''} ${s.adblock ? '' : 'disabled'}><i></i></span></label>`).join('') + '</div>';
    }
    case 'bmStats': {
      const marks = S.bm ? [...S.bm.kids.values()].flat().filter((x) => x.k === 'b').length : 0;
      const folders = S.bm ? S.bm.folders.size : 0;
      return `<div class="set">${`<div class="lbl"><b>${marks} bookmark${marks === 1 ? '' : 's'} in ${folders} folder${folders === 1 ? '' : 's'}</b>` +
        '<small>Drag them on the bar to reorder, right-click for more</small></div>'}</div>`;
    }
    case 'links':
      return `<div class="btns">` + it.items.map(([go, ic, l]) => `<button class="btn" data-go="${go}">${icon(ic)}${esc(l)}</button>`).join('') + '</div>';
    case 'histKeep': {
      const d = s.history_days | 0; const custom = S.histCustom || !KEEP.some(([v]) => v === d);
      return `<div class="set">${label}<div class="hk"><select class="field" data-hk style="width:auto;min-width:130px">` +
        KEEP.map(([v, l]) => `<option value="${v}" ${!custom && v === d ? 'selected' : ''}>${v ? 'After ' + l : 'Never'}</option>`).join('') +
        `<option value="c" ${custom ? 'selected' : ''}>Custom…</option></select>` +
        (custom ? `<input class="field" type="number" data-hkd min="1" max="3650" value="${d || 14}"><span class="u">days</span>` : '') + '</div></div>';
    }
    case 'text':
      return `<div class="set col">${label}<input class="field" data-k="${it.k}" value="${esc(s[it.k])}" placeholder="${esc(it.placeholder || '')}" spellcheck="false"></div>`;
    case 'code':
      return `<div class="set col">${label}<textarea class="field" data-k="${it.k}" spellcheck="false" placeholder="/* your CSS */">${esc(s[it.k])}</textarea></div>`;
    case 'color': {
      const custom = !ACCENTS.includes(s.accent);
      return `<div class="set col">${label}<div class="swatches">` +
        ACCENTS.map((c) => `<button class="sw ${s.accent === c ? 'on' : ''}" data-color="${c}" style="background:${c}" title="${c}"></button>`).join('') +
        `<label class="sw sw-custom ${custom ? 'on' : ''}" title="Custom color"><input type="color" data-k="accent" value="${esc(s.accent)}"></label></div></div>`;
    }
    case 'buttons':
      return `<div class="btns">` + it.items.map(([act, ic, l, danger]) =>
        `<button class="btn${danger ? ' danger' : ''}" data-act="${act}">${icon(ic)}${esc(l)}</button>`).join('') + '</div>';
    case 'allow': {
      if (!s.adblock_allow.length) return '';
      return `<div class="set col"><div class="lbl"><small>Blocking is paused on these sites</small></div><div class="chips">` +
        s.adblock_allow.map((h) => `<span>${esc(h)}<button data-unallow="${esc(h)}" title="Resume blocking">${icon('x')}</button></span>`).join('') + '</div></div>';
    }
    case 'shortcuts': {
      const list = s.shortcuts || [];
      let h = list.map((p, i) => `<div class="lrow"><span class="fav">${iconImg(hostOf(p.url), letter(p.url, p.title))}</span>` +
        `<div class="txt"><span class="t">${esc(p.title || hostOf(p.url))}</span><span class="u">${esc(pretty(p.url))}</span></div>` +
        `<button class="icon sm" data-sc-edit="${i}" title="Edit">${icon('edit')}</button>` +
        `<button class="icon sm" data-sc-del="${i}" title="Remove">${icon('trash')}</button></div>`).join('');
      if (!list.length) h += `<div class="hint">Pin your own sites to the start page. You can also right-click any tile there.</div>`;
      return h + `<div class="btns"><button class="btn" data-sc-add>${icon('plus')}Add a favourite</button></div>`;
    }
    case 'hidden': {
      const list = s.hidden_tiles || [];
      if (!list.length) return `<div class="hint">Sites you remove from the start page show up here, so you can bring them back.</div>`;
      return `<div class="set col"><div class="chips">` +
        list.map((h) => `<span>${esc(h)}<button data-unhide="${esc(h)}" title="Show on the start page again">${icon('x')}</button></span>`).join('') + '</div></div>' +
        `<div class="btns"><button class="btn" data-unhide="*">${icon('eye')}Show all again</button></div>`;
    }
    case 'importPC': {
      const src = S.sources;
      const busy = !!S.importing;
      if (!src) return `<div class="hint">Looking for browsers...</div>`;
      if (!src.length) return `<div class="hint">No Chrome, Edge, Brave, Vivaldi or Opera profiles found on this PC.</div>`;
      let h = `<div class="lrow"><div class="txt"><span class="t">Bookmarks and history</span><span class="u">${src.length} ${src.length === 1 ? 'profile' : 'profiles'} found. Importing again won't create duplicates.</span></div>` +
        (src.length > 1 ? `<button class="btn sm primary" data-import="*" data-label="all browsers" ${busy ? 'disabled' : ''}>Import all</button>` : '') + `</div>`;
      h += src.map((x) => {
        const label = x.profile ? `${x.browser} · ${x.profile}` : x.browser;
        return `<div class="lrow"><span class="fav">${letter('https://' + x.browser.toLowerCase() + '.app', x.browser)}</span><div class="txt"><span class="t">${esc(x.browser)}</span><span class="u">${esc(x.profile || 'Default profile')}</span></div>` +
          `<button class="btn sm" data-import="${esc(x.id)}" data-label="${esc(label)}" ${busy ? 'disabled' : ''}>${icon('download')}Import</button></div>`;
      }).join('');
      return h;
    }
    case 'importGoogle': {
      const busy = !!S.importing;
      return `<ol class="steps"><li>Open <b>Google Takeout</b> and export your <b>Chrome</b> data.</li>` +
        `<li>Unzip the download.</li><li>Import <b>BrowserHistory.json</b> and <b>Bookmarks.html</b> from it. Any browser's bookmarks .html works too.</li></ol>` +
        `<div class="btns"><button class="btn" data-open="https://takeout.google.com/settings/takeout/custom/chrome">${icon('external')}Open Google Takeout</button>` +
        `<label class="btn" ${busy ? 'style="opacity:.5;pointer-events:none"' : ''}>${icon('folder')}Import files<input type="file" id="import-file" accept=".json,.html,.htm" multiple hidden></label></div>` +
        `<div class="hint">Passwords and signed-in sessions can't be copied: Chrome encrypts them so only Chrome can read them. Sign in once in Lurt and it remembers you.</div>`;
    }
    case 'account': {
      const [g, ...more] = S.accounts;
      if (!g) return `<div class="acct-card">${avatar(null, 52)}<div class="lbl"><b>Not signed in</b><small>Sign in to Google once and Gmail, YouTube and the other Google sites stay signed in. Your account then shows at the top right.</small></div></div>` +
        `<div class="btns"><button class="btn primary" data-ga="signIn">${icon('user')}Sign in to Google</button></div>`;
      return `<div class="acct-card">${avatar(g, 52)}<div class="lbl"><b>${esc(g.name || 'Google account')}</b><small>${esc(g.email)}</small>` +
        (more.length ? `<small>Also signed in: ${more.map((o) => esc(o.email)).join(', ')}</small>` : '') + '</div></div>' +
        `<div class="btns"><button class="btn" data-ga="manage">${icon('external')}Manage your Google Account</button>` +
        `<button class="btn" data-ga="choose">${icon('users')}Switch account</button>` +
        `<button class="btn" data-ga="add">${icon('plus')}Add account</button>` +
        `<button class="btn danger" data-ga="out">${icon('logout')}Sign out of Google</button></div>`;
    }
    case 'saved': {
      const d = S.dataInfo; const n = (x) => (d ? Number(x).toLocaleString() : '…');
      const mb = d ? (d.dbBytes / 1048576).toFixed(1) + ' MB' : '…';
      const row = (ic, t, u) => `<div class="saved-row">${icon(ic)}<div class="lbl"><b>${t}</b><small>${u}</small></div></div>`;
      return `<div class="saved">` +
        row('gear', 'Settings', 'Theme, glass, layout, search engine and everything else on these pages') +
        row('star', `${n(d?.bookmarks)} bookmarks in ${n(d?.folders)} folders`, 'With their order on the bookmarks bar') +
        row('clock', `${n(d?.history)} history entries`, S.settings?.history_days ? `Deleted after ${S.settings.history_days} days` : 'Kept until you clear them') +
        row('globe', 'Cookies, sign-ins and site data', 'Including your Google sign-in. Passwords and forms when you allow it') +
        `</div><div class="hint">Everything stays on this PC (${mb} database). Nothing is sent to Lurt. Folder: ${esc(d?.dir || '')}</div>` +
        `<div class="btns"><button class="btn" data-act="openData">${icon('folder')}Open data folder</button><button class="btn" data-go="sec:privacy">${icon('trash')}Clear browsing data</button></div>`;
    }
    case 'wipe':
      return `<div class="set"><div class="lbl"><b>Delete all browser data</b><small>History, bookmarks, passwords, cookies and sign-ins, settings and open tabs. ` +
        `${S.accounts.length ? "You'll be asked to sign out of Google first." : 'Lurt goes back to how it was on first launch.'}</small></div>` +
        `<button class="btn danger" data-wipe>${icon('trash')}Delete everything</button></div>`;
    case 'profileSync':
      return `<div class="lbl"><small>Save your Lurt settings to a file and load them on another PC or after reinstalling. Syncing them to your Google account is coming: it needs Lurt to be registered with Google first.</small></div>` +
        `<div class="btns"><button class="btn" data-act="exportSettings">${icon('download')}Save settings to a file</button>` +
        `<label class="btn">${icon('upload')}Load settings from a file<input type="file" id="settings-file" accept=".json" hidden></label></div>`;
    case 'about':
      return `<div class="about"><span class="logo">${LOGO}</span><div><b>Lurt ${VERSION}</b><small>A light, fast glass browser written in Rust, running on Microsoft Edge WebView2.</small></div></div>` +
        `<div class="btns"><button class="btn" data-open="https://github.com/ladroxd/Lurt">${icon('external')}Project page</button><button class="btn" data-open="https://github.com/ladroxd/Lurt/releases">${icon('download')}Releases</button></div>`;
    case 'keys':
      return `<div class="keys">` + KEYS.map(([a, k]) => `<span>${esc(a)}</span><kbd>${esc(k)}</kbd>`).join('') + `</div>`;
    case 'note':
      return `<div class="hint">${esc(val(it.text))}</div>`;
  }
  return '';
}

// The same steps the engine uses for Ctrl + scroll.
const ZOOMS = [25, 33, 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300, 400, 500];
// 1 zooms in a step, -1 out, 0 goes back to 100%.
function stepZoom(dir) {
  if (!S.settings) return;
  const now = S.settings.page_zoom;
  const next = !dir ? 100 : (dir > 0 ? ZOOMS.find((z) => z > now) : [...ZOOMS].reverse().find((z) => z < now)) ?? now;
  if (next === now) return;
  setSetting('page_zoom', next, { rerender: settingsOpen() });
  toast(`Zoom ${next}%`);
}
let saveT;
function setSetting(k, v, { lazy = false, rerender = false } = {}) {
  S.settings[k] = v;
  applyLook();
  if (k === 'shortcuts' || k === 'hidden_tiles') renderTiles();
  clearTimeout(saveT);
  saveT = setTimeout(() => send({ t: 'settings', s: S.settings }), lazy ? 300 : 0);
  if (rerender) renderSettings();
}
// Both the settings window and the Trackers panel use the same controls.
function bindSettings(el, active, ownActions) {
  el.addEventListener('change', (e) => {
    if (!active()) return;
    if (e.target.id === 'import-file') {
      for (const f of e.target.files) {
        S.importing = f.name; S.importMsg = '';
        f.text().then((text) => send({ t: 'importFile', name: f.name, text }));
      }
      renderSettings();
      return;
    }
    if (e.target.dataset.a === 'site') { send({ t: 'shield' }); return; }
    if (e.target.dataset.rule !== undefined && e.target.tagName === 'SELECT') {
      const rules = structuredClone(S.settings.download_rules || []);
      rules[+e.target.dataset.rule].action = e.target.value;
      setSetting('download_rules', rules, { rerender: true });
      return;
    }
    if (e.target.dataset.list) {
      const id = e.target.dataset.list;
      const cur = (S.settings.filter_lists || []).filter((x) => x !== id);
      if (e.target.checked) cur.push(id);
      setSetting('filter_lists', LISTS.map(([x]) => x).filter((x) => cur.includes(x)), { rerender: true });
      return;
    }
    if (e.target.hasAttribute('data-hk')) {
      S.histCustom = e.target.value === 'c';
      setSetting('history_days', S.histCustom ? (S.settings.history_days || 14) : +e.target.value, { rerender: true });
      return;
    }
    if (e.target.hasAttribute('data-hkd')) {
      setSetting('history_days', Math.max(1, Math.min(3650, Math.round(+e.target.value) || 1)));
      return;
    }
    const t = e.target; const k = t.dataset.k; if (!k) return;
    const lvlKey = k in LEVELS.standard;
    if (t.type === 'checkbox') setSetting(k, t.checked, { rerender: lvlKey });
    else if (t.tagName === 'SELECT') setSetting(k, t.hasAttribute('data-num') ? +t.value : t.value, { rerender: lvlKey });
    else if (t.type === 'color') setSetting(k, t.value, { rerender: true });
    else if (t.type === 'range') setSetting(k, +t.value);
  });
  el.addEventListener('input', (e) => {
    if (!active()) return;
    const t = e.target; const k = t.dataset.k; if (!k) return;
    if (t.type === 'range') {
      t.style.setProperty('--p', ((t.value - t.min) / (t.max - t.min)) * 100 + '%');
      t.closest('.set').querySelector('.val').textContent = t.value + t.dataset.unit;
      // Window opacity is applied natively: preview it live while dragging.
      if (k === 'window_opacity') send({ t: 'winOpacity', v: +t.value });
      setSetting(k, +t.value, { lazy: true });
    } else if (t.tagName === 'TEXTAREA' || t.type === 'text' || t.type === 'color') {
      setSetting(k, t.value, { lazy: true });
    }
  });
  el.addEventListener('click', (e) => {
    if (!active()) return;
    const q = (sel) => e.target.closest(sel);
    if (ownActions && q('[data-act]')) { runAction(q('[data-act]')); return; }
    const segBtn = q('.seg[data-k] button');
    if (segBtn) {
      const seg = segBtn.parentElement;
      seg.querySelectorAll('button').forEach((b) => b.classList.toggle('on', b === segBtn));
      movePill(seg);
      setSetting(seg.dataset.k, segBtn.dataset.v, { rerender: seg.dataset.k === 'backdrop' });
      return;
    }
    const tab = q('[data-dltab]');
    if (tab) { S.dlTab = tab.dataset.dltab; renderSettings(true); return; }
    if (q('[data-dir-reset]')) { setSetting('download_dir', '', { rerender: true }); send({ t: 'downloads' }); return; }
    const rdel = q('[data-rule-del]');
    if (rdel) { setSetting('download_rules', S.settings.download_rules.filter((_, i) => i !== +rdel.dataset.ruleDel), { rerender: true }); return; }
    if (q('[data-rule-add]')) { addRule($('#rule-ext').value, $('#rule-act').value); return; }
    const pre = q('[data-rule-preset]');
    if (pre) { for (const x of DL_PROGRAMS) addRule(x, 'block', true); renderSettings(); return; }
    const dla = q('[data-dlp]');
    if (dla) { dlPageAction(dla); return; }
    const lvl = q('[data-level]');
    if (lvl) {
      const v = LEVELS[lvl.dataset.level];
      Object.assign(S.settings, structuredClone(v));
      setSetting('adblock', v.adblock, { rerender: true });
      return;
    }
    const zm = q('[data-zoom]');
    if (zm) { stepZoom(+zm.dataset.zoom); return; }
    const sw = q('.sw[data-color]');
    if (sw) { setSetting('accent', sw.dataset.color, { rerender: true }); return; }
    const imp = q('[data-import]');
    if (imp) {
      S.importing = imp.dataset.label; S.importMsg = '';
      send({ t: 'importProfile', id: imp.dataset.import, label: imp.dataset.label });
      renderSettings();
      return;
    }
    const op = q('[data-open]');
    if (op) { closeSettings(); send({ t: 'open', url: op.dataset.open, newTab: true }); return; }
    const go = q('[data-settings]');
    if (go) { openSettings(go.dataset.settings); return; }
    const un = q('[data-unallow]');
    if (un) { setSetting('adblock_allow', S.settings.adblock_allow.filter((h) => h !== un.dataset.unallow), { rerender: true }); return; }
    const uh = q('[data-unhide]');
    if (uh) { const h = uh.dataset.unhide; setSetting('hidden_tiles', h === '*' ? [] : S.settings.hidden_tiles.filter((x) => x !== h), { rerender: true }); return; }
    if (q('[data-sc-add]')) { editTile(null); return; }
    const ed = q('[data-sc-edit]');
    if (ed) { const p = S.settings.shortcuts[+ed.dataset.scEdit]; editTile({ ...p, pinned: true }); return; }
    const del = q('[data-sc-del]');
    if (del) { removePin(S.settings.shortcuts[+del.dataset.scDel].url); }
  });
}
bindSettings($('#set-body'), settingsOpen, true);
bindSettings($('#side-body'), () => S.panel === 'trackers', false);

// ---- keyboard --------------------------------------------------------------
const FORWARD = new Set(['ctrl+j', 'ctrl+t', 'ctrl+w', 'ctrl+tab', 'ctrl+shift+tab', 'ctrl+d', 'ctrl+shift+t', 'ctrl+shift+n',
  'ctrl+1', 'ctrl+2', 'ctrl+3', 'ctrl+4', 'ctrl+5', 'ctrl+6', 'ctrl+7', 'ctrl+8', 'ctrl+9']);
addEventListener('keydown', (e) => {
  let k = e.key.toLowerCase(); if (k === ',') k = 'comma';
  const c = (e.ctrlKey ? 'ctrl+' : '') + (e.altKey ? 'alt+' : '') + (e.shiftKey ? 'shift+' : '') + k;
  // Zoom keys while the toolbar has the keyboard (pages handle their own).
  if (e.ctrlKey && !e.altKey && ['=', '+', '-', '0'].includes(e.key)) { e.preventDefault(); stepZoom(e.key === '-' ? -1 : e.key === '0' ? 0 : 1); return; }
  if (FORWARD.has(c)) { e.preventDefault(); send({ t: 'key', k: c }); return; }
  if (c === 'ctrl+l' || c === 'alt+d' || c === 'f6') { e.preventDefault(); addr.focus(); addr.select(); return; }
  if (c === 'ctrl+h') { e.preventDefault(); togglePanel('history'); return; }
  if (c === 'ctrl+shift+b') { e.preventDefault(); send({ t: 'key', k: c }); return; }
  if (c === 'ctrl+shift+o') { e.preventDefault(); togglePanel('bookmarks'); return; }
  if (c === 'escape' && S.bmMenus.length && !document.body.classList.contains('ctx-open')) { closeBmMenus(); return; }
  if (c === 'ctrl+comma') { e.preventDefault(); settingsOpen() ? closeSettings() : openSettings(); return; }
  if (c === 'escape' && document.body.classList.contains('ctx-open')) { hideMenu(); return; }
  if (c === 'ctrl+f' && settingsOpen()) { e.preventDefault(); $('#set-q').focus(); $('#set-q').select(); return; }
  if (c === 'escape' && settingsOpen() && !document.body.classList.contains('dlg-open')) { closeSettings(); return; }
  if (settingsOpen() || document.body.classList.contains('dlg-open')) return;
  if (c === 'f5' || c === 'ctrl+r') { e.preventDefault(); send({ t: 'nav', a: 'reload' }); return; }
  if (c === 'alt+arrowleft') { e.preventDefault(); send({ t: 'nav', a: 'back' }); return; }
  if (c === 'alt+arrowright') { e.preventDefault(); send({ t: 'nav', a: 'forward' }); return; }
  if (c === 'escape' && S.panel && document.activeElement !== addr) { closePanel(); return; }
  // Typing anywhere on the start page goes to its search box.
  const typing = document.activeElement && /INPUT|TEXTAREA|SELECT/.test(document.activeElement.tagName);
  if (S.blank && !typing && !e.ctrlKey && !e.altKey && e.key.length === 1) startQ.focus();
});
addEventListener('contextmenu', (e) => { if (!e.target.closest('input, textarea')) e.preventDefault(); });

send({ t: 'ready' });
send({ t: 'icons' });

// ---- Google account ------------------------------------------------------------------
// The account signed in to Google in this browser (read by Rust from Google's own
// cookies), shown as an avatar at the top right with a profile popover.
S.accounts = []; S.dataInfo = null;
const acctBtn = $('#acct'); const acctPop = $('#acct-pop');
const GOOGLE = {
  signIn: 'https://accounts.google.com/ServiceLogin?continue=https%3A%2F%2Fmyaccount.google.com%2F',
  manage: 'https://myaccount.google.com/',
  choose: 'https://accounts.google.com/AccountChooser?continue=https%3A%2F%2Fmyaccount.google.com%2F',
  add: 'https://accounts.google.com/AddSession?continue=https%3A%2F%2Fmyaccount.google.com%2F',
};
function avatar(a, size) {
  const initial = esc(((a && (a.name || a.email)) || '?').trim()[0]?.toUpperCase() || '?');
  if (!a) return `<span class="av none" style="--s:${size}px">${icon('user')}</span>`;
  const bg = `hsl(${hue(a.email)} 55% 45%)`;
  const photo = a.photo ? `<img src="${esc(a.photo)}" alt="" referrerpolicy="no-referrer" onerror="this.remove()">` : '';
  return `<span class="av" style="--s:${size}px;background:${bg}">${initial}${photo}</span>`;
}
H.account = (m) => {
  S.accounts = m.accounts || [];
  // Signing out on the way to deleting everything: go on to the delete step.
  if (S.wipeStep === 'signout' && document.body.classList.contains('dlg-open')) {
    S.wipeStep = null;
    wipeDialog(S.accounts.length ? 'failed' : undefined);
  }
  renderAcct();
  if (S.overlay === 'acct-open') renderAcctPop();
  if (settingsOpen() && S.setSec === 'account') renderSettings();
};
// Delete all browser data: signed in, it signs you out of Google first; then it asks
// you to type "delete" before wiping everything.
function wipeDialog(state) {
  const g = S.accounts[0];
  dlg._orig = null;
  if (g) {
    dlg.innerHTML = `<h3>Sign out first</h3><p>You're signed in to Google as <b>${esc(g.email)}</b>. Lurt signs you out before deleting your data, so no account stays signed in.</p>` +
      `<div class="err" id="td-err">${state === 'failed' ? "Couldn't sign out. Try again." : ''}</div>` +
      `<div class="btns"><button class="btn" data-d="cancel">Cancel</button><button class="btn primary" data-d="save" ${state === 'busy' ? 'disabled' : ''}>${state === 'busy' ? 'Signing out…' : 'Sign out of Google'}</button></div>`;
    dlg._save = () => { if (S.wipeStep) return; S.wipeStep = 'signout'; send({ t: 'signOut' }); wipeDialog('busy'); };
  } else {
    dlg.innerHTML = `<h3>Delete all browser data?</h3><p>This deletes, for good:</p><ul>` +
      '<li>History</li><li>Bookmarks and folders</li><li>Saved passwords and autofill</li><li>Cookies, sign-ins, cache and site data</li><li>Your settings and start page favourites</li><li>Open tabs</li></ul>' +
      `<p class="dim">Files in the mods folder stay.</p>` +
      `<label>Type <b>delete</b> to confirm<input class="field" id="wipe-ok" spellcheck="false" autocomplete="off"></label>` +
      `<div class="err" id="td-err"></div>` +
      `<div class="btns"><button class="btn" data-d="cancel">Cancel</button><button class="btn primary danger-fill" data-d="save">Delete everything</button></div>`;
    dlg._save = () => {
      if ($('#wipe-ok').value.trim().toLowerCase() !== 'delete') { $('#td-err').textContent = 'Type delete to confirm'; return; }
      send({ t: 'wipeAll' }); closeDlg();
    };
    setTimeout(() => $('#wipe-ok')?.focus(), 30);
  }
  document.body.classList.add('dlg-open');
}
// Signed in to Google for the first time: offer to bring in data from this PC's browsers.
H.importOffer = (m) => {
  const a = m.account; const src = m.sources || [];
  const mine = (x) => x.email && x.email.toLowerCase() === a.email.toLowerCase();
  S.offerPick = 0;
  const row = (x, i) => `<button class="pick${i === 0 ? ' on' : ''}" data-pick="${i}"><span class="dot"></span><span class="txt"><b>${esc(x.browser)}${x.profile ? ' · ' + esc(x.profile) : ''}</b>` +
    `<small>${mine(x) ? 'Signed in with this account' : x.email ? esc(x.email) : 'Not signed in to Google'}</small></span></button>`;
  dlg._orig = null;
  dlg.innerHTML = `<h3>Bring your data into Lurt</h3>` +
    `<p>You're signed in as <b>${esc(a.email)}</b>. Google doesn't let other browsers download Chrome Sync, so Lurt copies your bookmarks and history from a browser on this PC.</p>` +
    `<div class="picks">${src.slice(0, 6).map(row).join('')}</div>` +
    `<p class="dim">Passwords stay in that browser: it encrypts them so only it can read them. You can also import a Google Takeout export from Settings → Your data.</p>` +
    `<div class="btns"><button class="btn" data-d="cancel">Not now</button><button class="btn primary" data-d="save">Import</button></div>`;
  dlg._save = () => {
    const x = src[S.offerPick]; if (!x) return;
    const label = x.browser + (x.profile ? ' (' + x.profile + ')' : '');
    S.importing = label; S.importMsg = '';
    send({ t: 'importProfile', id: x.id, label });
    toast('Importing from ' + label + '…');
    closeDlg();
  };
  if (settingsOpen()) closeSettings();
  bmOverlay();
  document.body.classList.add('dlg-open');
};
dlg.addEventListener('click', (e) => {
  const b = e.target.closest('[data-pick]'); if (!b) return;
  S.offerPick = +b.dataset.pick;
  dlg.querySelectorAll('[data-pick]').forEach((x) => x.classList.toggle('on', x === b));
});
document.addEventListener('click', (e) => { if (e.target.closest('#settings [data-wipe]')) { S.wipeStep = null; wipeDialog(); } });
H.dataInfo = (m) => { S.dataInfo = m; if (settingsOpen() && S.setSec === 'account') renderSettings(); };
function renderAcct() {
  const a = S.accounts[0];
  acctBtn.innerHTML = avatar(a, 22);
  acctBtn.title = a ? `${a.name || 'Google account'}\n${a.email}` : 'Sign in to Google';
  acctBtn.classList.toggle('on', !!a);
}
function renderAcctPop() {
  const [a, ...more] = S.accounts;
  acctPop.innerHTML = a
    ? `<div class="me">${avatar(a, 64)}<b>${esc(a.name || 'Google account')}</b><small>${esc(a.email)}</small>` +
      `<button class="btn" data-ga="manage">Manage your Google Account</button></div>` +
      (more.length ? `<div class="others">` + more.map((o) => `<div class="other">${avatar(o, 28)}<div class="txt"><span class="t">${esc(o.name || o.email)}</span><span class="u">${esc(o.email)}</span></div></div>`).join('') + '</div>' : '') +
      `<div class="acts">` +
      `<button data-ga="choose">${icon('users')}Switch account</button>` +
      `<button data-ga="add">${icon('plus')}Add another account</button>` +
      `<button data-ga="settings">${icon('gear')}Account & data in Settings</button>` +
      `<button data-ga="out" class="danger">${icon('logout')}Sign out of Google</button></div>`
    : `<div class="me">${avatar(null, 64)}<b>Not signed in</b><small>Sign in to Google to use Gmail, YouTube and other Google sites signed in, and see your account here.</small>` +
      `<button class="btn primary" data-ga="signIn">Sign in to Google</button></div>` +
      `<div class="acts"><button data-ga="settings">${icon('gear')}Account & data in Settings</button></div>`;
}
function placeAcct() {
  const r = acctBtn.getBoundingClientRect();
  acctPop.style.top = r.bottom + 8 + 'px';
  acctPop.style.left = Math.max(8, Math.min(r.right - 300 + 6, innerWidth - 308)) + 'px';
}
function openAcct() {
  if (settingsOpen()) return;
  if (S.overlay === 'omni-open') box.blur();
  if (S.overlay === 'bm-open') closeBmMenus();
  if (S.overlay) closeOverlay(false);
  renderAcctPop(); placeAcct();
  openOverlay('acct-open');
  send({ t: 'account' });
}
function closeAcct(focusPage) { if (S.overlay === 'acct-open') closeOverlay(focusPage); }
acctBtn.onclick = () => (S.overlay === 'acct-open' ? closeAcct(true) : openAcct());
function googleAction(ga) {
  if (ga === 'out') {
    // A second click confirms, like the other destructive buttons.
    const b = document.querySelector('[data-ga="out"]');
    if (b && !b.classList.contains('armed')) {
      document.querySelectorAll('[data-ga="out"]').forEach((x) => { x.classList.add('armed'); x._l = x.innerHTML; x.innerHTML = 'Click again to sign out'; });
      setTimeout(() => document.querySelectorAll('[data-ga="out"].armed').forEach((x) => { x.classList.remove('armed'); x.innerHTML = x._l; }), 3000);
      return;
    }
    send({ t: 'signOut' }); closeAcct(false); return;
  }
  if (ga === 'settings') { closeAcct(false); openSettings('account'); return; }
  if (GOOGLE[ga]) { closeAcct(false); closeSettings(); send({ t: 'open', url: GOOGLE[ga], newTab: true }); }
}
document.addEventListener('click', (e) => { const b = e.target.closest('#settings [data-ga]'); if (b) googleAction(b.dataset.ga); });
document.addEventListener('change', (e) => {
  if (e.target.id !== 'settings-file' || !e.target.files[0]) return;
  e.target.files[0].text().then((text) => send({ t: 'importSettings', text }));
  e.target.value = '';
});
acctPop.addEventListener('click', (e) => { const b = e.target.closest('[data-ga]'); if (b) googleAction(b.dataset.ga); });
addEventListener('mousedown', (e) => {
  if (S.overlay === 'acct-open' && !acctPop.contains(e.target) && !acctBtn.contains(e.target)) closeAcct(false);
}, true);
addEventListener('keydown', (e) => { if (e.key === 'Escape' && S.overlay === 'acct-open') { e.preventDefault(); closeAcct(true); } });
addEventListener('resize', () => S.overlay === 'acct-open' && placeAcct());
renderAcct();

// ---- startup skeleton ----------------------------------------------------------------
// Rust draws a skeleton of this UI before the engine starts. Things whose size depends on
// your data (tiles, bookmarks, the clock) are measured here and saved for next launch.
let skelT, skelLast = '';
function saveSkeleton() {
  clearTimeout(skelT);
  skelT = setTimeout(() => {
    if (!document.body.classList.contains('blank')) return;
    const w = (el) => Math.round(el.getBoundingClientRect().width);
    // Width of the text itself (the clock lines are full-width blocks).
    const textW = (el) => { if (!el) return 0; const r = document.createRange(); r.selectNodeContents(el); return Math.round(r.getBoundingClientRect().width); };
    const shape = {
      tiles: [...document.querySelectorAll('#tiles .tile > span:not(.ic)')].map(w).filter((x) => x > 0),
      bm: [...document.querySelectorAll('#bmbar .bm')].filter((e) => e.offsetParent && e.getBoundingClientRect().right < innerWidth - 30).map(w),
      time_w: textW($('#time')) || 259,
      date_w: textW($('#date')) || 148,
    };
    const key = JSON.stringify(shape);
    if (key !== skelLast) { skelLast = key; send({ t: 'skel', shape }); }
  }, 1500);
}
new MutationObserver(saveSkeleton).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['class'] });

// ---- downloads ---------------------------------------------------------------
// Lurt handles downloads itself (no engine pop-up): a toolbar button with a progress ring,
// and a panel with pause, resume, cancel, open and show in folder.
const dlBtn = $('#dl'); const dlPop = $('#dl-pop');
dlBtn.innerHTML = `${icon('download')}<i class="ring"></i><span class="n"></span>`;
S.dl = []; S.dlSpeed = new Map();
const active = (d) => d.state === 'progress' || d.state === 'paused';
function fmtBytes(n) {
  if (!(n > 0)) return '0 B';
  const u = ['B', 'KB', 'MB', 'GB', 'TB']; let i = 0;
  while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
  return `${n < 10 && i ? n.toFixed(1) : Math.round(n)} ${u[i]}`;
}
function fmtTime(s) {
  if (!isFinite(s) || s < 0) return '';
  if (s < 60) return `${Math.max(1, Math.round(s))} s left`;
  if (s < 3600) return `${Math.round(s / 60)} min left`;
  return `${Math.floor(s / 3600)} h ${Math.round((s % 3600) / 60)} min left`;
}
function dlStatus(d) {
  const sp = S.dlSpeed.get(d.id);
  const size = d.total > 0 ? `${fmtBytes(d.recv)} of ${fmtBytes(d.total)}` : fmtBytes(d.recv);
  switch (d.state) {
    case 'progress': {
      if (!sp || !sp.bps) return size;
      const left = d.total > 0 ? fmtTime((d.total - d.recv) / sp.bps) : '';
      return [size, `${fmtBytes(sp.bps)}/s`, left].filter(Boolean).join(' · ');
    }
    case 'paused': return `Paused · ${size}`;
    case 'done': return d.gone ? 'Moved or deleted' : `${fmtBytes(d.total || d.recv)} · ${hostOf(d.url) || 'Done'}`;
    case 'canceled': return 'Canceled';
    default: return d.reason || 'Failed';
  }
}
function dlRow(d, big) {
  const ext = (d.name.match(/\.([a-z0-9]{1,5})$/i) || [, ''])[1].toUpperCase() || 'FILE';
  const pct = d.total > 0 ? Math.min(100, (d.recv / d.total) * 100) : 0;
  const b = (a, ic, title) => `<button class="icon sm" data-dl="${a}" title="${title}">${icon(ic)}</button>`;
  const acts = active(d)
    ? (d.state === 'paused' ? b('Resume', 'play', 'Resume') : b('Pause', 'pause', 'Pause')) + b('Cancel', 'x', 'Cancel')
    : d.state === 'done'
      ? (d.gone ? '' : b('Show', 'folder', 'Show in folder')) + b('Remove', 'x', 'Remove from list')
      : b('Retry', 'refresh', 'Try again') + b('Remove', 'x', 'Remove from list');
  return `<div class="dl-row ${d.state}${d.gone ? ' gone' : ''}" data-id="${d.id}" title="${esc(d.path)}">` +
    `<span class="dl-ic">${esc(ext)}</span>` +
    `<div class="dl-txt"><b>${esc(d.name)}</b>` + (big && d.url ? `<a class="dl-src" title="${esc(d.url)}">${esc(d.url.length > 90 ? d.url.slice(0, 90) + '…' : d.url)}</a>` : '') + `<small>${esc(dlStatus(d))}</small>` +
    (active(d) ? `<i class="bar ${d.total > 0 ? '' : 'indef'}"><i style="width:${pct}%"></i></i>` : '') + '</div>' +
    `<div class="dl-acts">${acts}</div></div>`;
}
function renderDlBtn() {
  const act = S.dl.filter(active);
  dlBtn.classList.toggle('busy', act.length > 0);
  const tot = act.reduce((a, d) => a + Math.max(0, d.total), 0), got = act.reduce((a, d) => a + d.recv, 0);
  dlBtn.style.setProperty('--p', tot > 0 ? `${(got / tot) * 100}%` : '25%');
  dlBtn.classList.toggle('indef', act.length > 0 && !(tot > 0));
  dlBtn.querySelector('.n').textContent = act.length > 1 ? act.length : '';
  dlBtn.title = act.length ? `Downloads: ${act.length} in progress` : 'Downloads';
}
function renderDlPop() {
  const finished = S.dl.some((d) => !active(d));
  const head = `<div class="dl-head"><b>Downloads</b><span>` +
    (finished ? `<button class="link" data-dlall="clear">Clear list</button>` : '') +
    `<button class="icon sm" data-dlall="folder" title="Open Downloads folder">${icon('folder')}</button></span></div>`;
  const body = S.dl.length ? `<div class="dl-list">${S.dl.map((d) => dlRow(d)).join('')}</div>`
    : `<div class="dl-empty">${icon('download')}<span>Files you download appear here</span></div>`;
  const foot = `<button class="dl-all" data-dlall="page">${icon('download')}Show all downloads<span>Ctrl+J</span></button>`;
  dlPop.innerHTML = head + body + foot;
}
// Update rows in place while the panel is open, so buttons stay clickable during progress.
function patchDlPop() {
  const list = dlPop.querySelector('.dl-list');
  const rows = list ? [...list.children] : [];
  if (!list || rows.length !== S.dl.length || rows.some((r, i) => +r.dataset.id !== S.dl[i].id)) { renderDlPop(); return; }
  S.dl.forEach((d, i) => {
    const r = rows[i];
    if (!r.classList.contains(d.state) || r.classList.contains('gone') !== !!d.gone) { r.outerHTML = dlRow(d); return; }
    r.querySelector('small').textContent = dlStatus(d);
    const bar = r.querySelector('.bar > i');
    if (bar && d.total > 0) bar.style.width = Math.min(100, (d.recv / d.total) * 100) + '%';
  });
}
H.downloads = (m) => {
  const now = performance.now();
  for (const d of m.list) {
    if (d.state !== 'progress') { S.dlSpeed.delete(d.id); continue; }
    const p = S.dlSpeed.get(d.id);
    if (!p) { S.dlSpeed.set(d.id, { recv: d.recv, t: now, bps: 0 }); continue; }
    const dt = (now - p.t) / 1000;
    if (dt >= 0.2) {
      const inst = Math.max(0, (d.recv - p.recv) / dt);
      S.dlSpeed.set(d.id, { recv: d.recv, t: now, bps: p.bps ? p.bps * 0.7 + inst * 0.3 : inst });
    }
  }
  S.dl = m.list;
  if (m.dir) S.dlDir = m.dir;
  renderDlBtn();
  if (S.overlay === 'dl-open') patchDlPop();
  if (dlPageOpen()) patchDlPage();
};
function dlPulse(cls) { dlBtn.classList.remove('pulse', 'done-pulse'); void dlBtn.offsetWidth; dlBtn.classList.add(cls); }
H.dlStart = () => dlPulse('pulse');
H.dlDone = () => dlPulse('done-pulse');
dlBtn.addEventListener('animationend', () => dlBtn.classList.remove('pulse', 'done-pulse'));
function placeDl() {
  const r = dlBtn.getBoundingClientRect();
  dlPop.style.top = r.bottom + 8 + 'px';
  dlPop.style.left = Math.max(8, Math.min(r.right - 360 + 6, innerWidth - 368)) + 'px';
}
function openDl() {
  if (settingsOpen()) return;
  if (S.overlay === 'omni-open') box.blur();
  if (S.overlay === 'bm-open') closeBmMenus();
  if (S.overlay) closeOverlay(false);
  renderDlPop(); placeDl();
  openOverlay('dl-open');
  send({ t: 'downloads' });
}
function closeDl(focusPage) { if (S.overlay === 'dl-open') closeOverlay(focusPage); }
dlBtn.onclick = () => (S.overlay === 'dl-open' ? closeDl(true) : openDl());
dlPop.addEventListener('click', (e) => {
  const all = e.target.closest('[data-dlall]');
  if (all) {
    if (all.dataset.dlall === 'page') { openDownloadsPage(); return; }
    send({ t: all.dataset.dlall === 'clear' ? 'dlClear' : 'dlFolder' }); if (all.dataset.dlall === 'folder') closeDl(false); return;
  }
  const row = e.target.closest('.dl-row'); if (!row) return;
  const id = +row.dataset.id;
  const b = e.target.closest('[data-dl]');
  if (b) { send({ t: 'dl' + b.dataset.dl, id }); if (b.dataset.dl === 'Show') closeDl(false); return; }
  if (row.classList.contains('done') && !row.classList.contains('gone')) { send({ t: 'dlOpen', id }); closeDl(false); }
});
addEventListener('mousedown', (e) => {
  if (S.overlay === 'dl-open' && !dlPop.contains(e.target) && !dlBtn.contains(e.target)) closeDl(false);
}, true);
addEventListener('keydown', (e) => { if (e.key === 'Escape' && S.overlay === 'dl-open') { e.preventDefault(); closeDl(true); } });
addEventListener('resize', () => S.overlay === 'dl-open' && placeDl());
send({ t: 'downloads' });

// ---- tab volume --------------------------------------------------------------
// A mixer for the tabs that make sound, or that you turned down.
const volBtn = $('#vol'); const volPop = $('#vol-pop');
const soundTabs = () => S.tabs.filter((t) => t.audible || t.muted || t.volume < 100);
function setSound(id, change) {
  const t = S.tabs.find((x) => x.id === id); if (!t) return;
  Object.assign(t, change);
  send({ t: 'sound', id, ...change });
  renderTabs(); renderVol();
}
function toggleMute(id) {
  const t = S.tabs.find((x) => x.id === id); if (!t) return;
  // Unmuting a tab that was slid to zero brings the sound back too.
  if (t.muted || t.volume === 0) setSound(id, { muted: false, volume: t.volume || 100 });
  else setSound(id, { muted: true });
}
function renderVol() {
  const list = soundTabs();
  volBtn.classList.toggle('on', list.length > 0);
  const silent = list.length > 0 && list.every((t) => t.muted || t.volume === 0);
  if (volBtn._silent !== silent) { volBtn._silent = silent; volBtn.innerHTML = icon(silent ? 'soundOff' : 'sound'); }
  if (S.overlay !== 'vol-open') return;
  if (!list.length || volPop._tab !== S.active) { closeVol(false); return; }
  // Rebuild only when the set of tabs changes, so a slider you're dragging stays put.
  const key = list.map((t) => t.id).join();
  if (volPop._key !== key) {
    volPop._key = key;
    volPop.innerHTML = '<div class="head"><b>Tab volume</b></div><div class="vol-list">' + list.map((t) =>
      `<div class="vol-row" data-id="${t.id}"><button class="icon sm" data-mute></button><div class="txt"><span class="t"></span>` +
      '<input type="range" min="0" max="100" aria-label="Volume"></div><span class="val"></span></div>').join('') + '</div>';
  }
  for (const t of list) {
    const row = volPop.querySelector(`.vol-row[data-id="${t.id}"]`);
    const quiet = t.muted || t.volume === 0;
    const range = row.querySelector('input');
    row.classList.toggle('quiet', quiet);
    row.querySelector('.t').textContent = t.title || pretty(t.url) || 'New tab';
    if (+range.value !== t.volume) range.value = t.volume;
    range.style.setProperty('--p', t.volume + '%');
    row.querySelector('.val').textContent = quiet ? 'Muted' : t.volume + '%';
    const b = row.querySelector('[data-mute]');
    if (b._quiet !== quiet) { b._quiet = quiet; b.innerHTML = icon(quiet ? 'soundOff' : 'sound'); b.title = quiet ? 'Unmute' : 'Mute'; }
  }
}
function placeVol() {
  const r = volBtn.getBoundingClientRect();
  volPop.style.top = r.bottom + 8 + 'px';
  volPop.style.left = Math.max(8, Math.min(r.right - 320 + 6, innerWidth - 328)) + 'px';
}
function openVol() {
  if (settingsOpen() || !soundTabs().length) return;
  if (S.overlay === 'omni-open') box.blur();
  if (S.overlay === 'bm-open') closeBmMenus();
  if (S.overlay) closeOverlay(false);
  volPop._key = ''; volPop._tab = S.active;
  openOverlay('vol-open');
  renderVol(); placeVol();
}
function closeVol(focusPage) { if (S.overlay === 'vol-open') closeOverlay(focusPage); }
volBtn.onclick = () => (S.overlay === 'vol-open' ? closeVol(true) : openVol());
volPop.addEventListener('input', (e) => {
  const row = e.target.closest('.vol-row'); if (row) setSound(+row.dataset.id, { volume: +e.target.value, muted: false });
});
volPop.addEventListener('click', (e) => {
  const b = e.target.closest('[data-mute]'); if (b) toggleMute(+b.closest('.vol-row').dataset.id);
});
addEventListener('mousedown', (e) => {
  if (S.overlay === 'vol-open' && !volPop.contains(e.target) && !volBtn.contains(e.target)) closeVol(false);
}, true);
addEventListener('keydown', (e) => { if (e.key === 'Escape' && S.overlay === 'vol-open') { e.preventDefault(); closeVol(true); } });
addEventListener('resize', () => S.overlay === 'vol-open' && placeVol());

// ---- downloads page (Settings > Downloads), like chrome://downloads ----------------------
S.dlTab = 'all'; S.dlQ = '';
function addRule(ext, action, quiet) {
  ext = String(ext || '').trim().toLowerCase().replace(/^\*?\./, '');
  if (!/^[a-z0-9]{1,12}$/.test(ext)) { if (!quiet) toast('Type a file type like exe or zip'); return; }
  const rules = (S.settings.download_rules || []).filter((r) => r.ext !== ext);
  rules.push({ ext, action });
  setSetting('download_rules', rules, { rerender: !quiet });
}
function openDownloadsPage() { S.dlTab = 'all'; closeDl(false); openSettings('downloads'); }
function dayLabel(sec) {
  const d = new Date(sec * 1000), t = new Date(); t.setHours(0, 0, 0, 0);
  const days = Math.round((t - new Date(d).setHours(0, 0, 0, 0)) / 864e5);
  if (days <= 0) return 'Today';
  if (days === 1) return 'Yesterday';
  return d.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric', year: d.getFullYear() === t.getFullYear() ? undefined : 'numeric' });
}
function dlShown() {
  const q = S.dlQ.trim().toLowerCase();
  return q ? S.dl.filter((d) => (d.name + ' ' + d.url).toLowerCase().includes(q)) : S.dl;
}
function dlPageList() {
  const list = dlShown();
  if (!S.dl.length) return `<div class="dl-empty big">${icon('download')}<span>Files you download appear here</span></div>`;
  if (!list.length) return `<div class="dl-empty big">${icon('search')}<span>No downloads match “${esc(S.dlQ)}”</span></div>`;
  let html = '', day = '';
  for (const d of list) {
    const l = dayLabel(d.at || 0);
    if (l !== day) { day = l; html += `<h4 class="dl-day">${esc(l)}</h4>`; }
    html += dlRow(d, true);
  }
  return html;
}
function downloadsPage(sec, s) {
  const tabs = `<div class="dlp-tabs"><button data-dltab="all" class="${S.dlTab === 'all' ? 'on' : ''}">${icon('download')}All downloads</button>` +
    `<button data-dltab="settings" class="${S.dlTab === 'settings' ? 'on' : ''}">${icon('gear')}Download settings</button></div>`;
  if (S.dlTab === 'settings') return tabs + groupsHtml(sec.groups, s);
  const n = S.dl.length, act = S.dl.filter(active).length;
  return tabs + `<div class="dlp-bar"><label class="dlp-search">${icon('search')}<input id="dlp-q" placeholder="Search downloads" value="${esc(S.dlQ)}" spellcheck="false" autocomplete="off"></label>` +
    `<span class="dlp-count">${n} file${n === 1 ? '' : 's'}${act ? ` · ${act} downloading` : ''}</span>` +
    `<button class="btn" data-dlp="folder">${icon('folder')}Open folder</button>` +
    (S.dl.some((d) => !active(d)) ? `<button class="btn" data-dlp="clear">${icon('trash')}Clear list</button>` : '') + '</div>' +
    `<div id="dlp-list" class="dl-list page">${dlPageList()}</div>`;
}
function dlPageOpen() { return settingsOpen() && S.setSec === 'downloads' && S.dlTab === 'all' && !$('#set-q').value.trim(); }
function dlPageAction(b) {
  const a = b.dataset.dlp;
  if (a === 'folder') { send({ t: 'dlFolder' }); return; }
  if (a === 'clear') { send({ t: 'dlClear' }); return; }
}
// Live updates without re-rendering rows, so clicks during progress aren't lost.
function patchDlPage() {
  const box = $('#dlp-list'); if (!box) return;
  const list = dlShown();
  const rows = [...box.querySelectorAll('.dl-row')];
  const same = rows.length === list.length && rows.every((r, i) => +r.dataset.id === list[i].id && r.classList.contains(list[i].state));
  if (!same) { box.innerHTML = dlPageList(); return; }
  list.forEach((d, i) => {
    rows[i].querySelector('small').textContent = dlStatus(d);
    const bar = rows[i].querySelector('.bar > i');
    if (bar && d.total > 0) bar.style.width = Math.min(100, (d.recv / d.total) * 100) + '%';
  });
  const c = $('#set-body .dlp-count');
  if (c) { const act = S.dl.filter(active).length; c.textContent = `${S.dl.length} file${S.dl.length === 1 ? '' : 's'}${act ? ` · ${act} downloading` : ''}`; }
}
$('#set-body').addEventListener('input', (e) => {
  if (e.target.id !== 'dlp-q') return;
  S.dlQ = e.target.value;
  $('#dlp-list').innerHTML = dlPageList();
});
// Row buttons work the same on the page as in the panel.
$('#set-body').addEventListener('click', (e) => {
  const row = e.target.closest('#dlp-list .dl-row'); if (!row) return;
  const id = +row.dataset.id;
  const b = e.target.closest('[data-dl]');
  if (b) { send({ t: 'dl' + b.dataset.dl, id }); return; }
  if (e.target.closest('.dl-src')) { send({ t: 'open', url: S.dl.find((d) => d.id === id)?.url, newTab: true }); closeSettings(); return; }
  if (row.classList.contains('done') && !row.classList.contains('gone')) send({ t: 'dlOpen', id });
});

// ---- bookmark card (star / Ctrl+D) ------------------------------------------------------
// Like Chrome: the star saves the page right away and opens a card to rename it, pick its
// folder (or make one), or remove it. On a bookmarked page it opens the same card to edit.
const starBtn = $('#star'); const starPop = $('#star-pop');
S.bmLast = 0; try { S.bmLast = +(localStorage.getItem('lurt.bmFolder') || 0); } catch (_) {}
function folderOptions(sel) {
  const out = [[0, 'Bookmarks bar', 0]];
  const walk = (p, depth) => {
    for (const x of bmKids(p)) if (x.k === 'f') { out.push([x.id, bmName(x), depth]); walk(x.id, depth + 1); }
  };
  walk(0, 1);
  return out.map(([id, name, d]) => `<option value="${id}" ${id === sel ? 'selected' : ''}>${'\u00a0\u00a0\u00a0'.repeat(d)}${esc(name)}</option>`).join('') +
    '<option value="new">New folder…</option>';
}
function renderStar() {
  const st = S.starEdit; if (!st) return;
  starPop.innerHTML = `<div class="sp-head"><b>${st.added ? 'Bookmark added' : 'Edit bookmark'}</b></div>` +
    `<label class="sp-row"><span>Name</span><input class="field" id="sp-name" value="${esc(st.title)}" spellcheck="false"></label>` +
    `<label class="sp-row"><span>Folder</span><select class="field" id="sp-folder">${folderOptions(st.parent)}</select></label>` +
    `<label class="sp-row sp-new ${st.newFolder ? '' : 'hide'}"><span>New folder</span><input class="field" id="sp-newname" placeholder="Folder name" spellcheck="false"></label>` +
    `<div class="sp-acts"><button class="btn" data-sp="remove">${icon('trash')}Remove</button><button class="btn primary" data-sp="done">Done</button></div>`;
}
function placeStar() {
  const r = starBtn.getBoundingClientRect();
  starPop.style.top = r.bottom + 10 + 'px';
  starPop.style.left = Math.max(8, Math.min(r.right - 320 + 10, innerWidth - 328)) + 'px';
}
function openStar() {
  const t = cur(); if (!t || !t.url || settingsOpen()) return;
  if (S.overlay === 'omni-open') box.blur();
  if (S.overlay === 'bm-open') closeBmMenus();
  if (S.overlay) closeOverlay(false);
  const mark = findMark(t.url);
  const title = mark ? bmName(mark) : (t.title || pretty(t.url));
  const parent = mark ? (S.bm.folders.has(mark.parent) ? mark.parent : 0) : (S.bm.folders.has(S.bmLast) ? S.bmLast : 0);
  S.starEdit = { url: t.url, title, parent, origTitle: title, origParent: parent, added: !mark, newFolder: false };
  if (!mark) send({ t: 'bmAdd', url: t.url, title, parent });
  renderStar(); placeStar();
  openOverlay('star-open');
  requestAnimationFrame(() => { const n = $('#sp-name'); if (n) { n.focus(); n.select(); } });
}
// Save what the card shows (on Done, Enter, or clicking away).
function saveStar() {
  const st = S.starEdit; if (!st || st.removed) return;
  const title = ($('#sp-name')?.value || '').trim() || st.origTitle;
  const fsel = $('#sp-folder')?.value;
  if (fsel === 'new') {
    const name = ($('#sp-newname')?.value || '').trim();
    if (name) { send({ t: 'bmAddNew', url: st.url, title, folder: name, parent: st.origParent }); S.bmPendingLast = true; }
    else if (title !== st.origTitle) send({ t: 'bmEdit', url: st.url, newUrl: st.url, title });
    return;
  }
  const parent = +fsel || 0;
  if (parent !== st.origParent) send({ t: 'bmAdd', url: st.url, title, parent });
  else if (title !== st.origTitle) send({ t: 'bmEdit', url: st.url, newUrl: st.url, title });
  S.bmLast = parent; try { localStorage.setItem('lurt.bmFolder', String(parent)); } catch (_) {}
}
function closeStar(focusPage) {
  if (S.overlay !== 'star-open') return;
  saveStar();
  S.starEdit = null;
  closeOverlay(focusPage);
}
H.starPop = () => (S.overlay === 'star-open' ? closeStar(true) : openStar());
H.bmFolderMade = (m) => {
  if (!S.bmPendingLast) return;
  S.bmPendingLast = false; S.bmLast = m.id;
  try { localStorage.setItem('lurt.bmFolder', String(m.id)); } catch (_) {}
};
starPop.addEventListener('change', (e) => {
  if (e.target.id !== 'sp-folder' || !S.starEdit) return;
  const isNew = e.target.value === 'new';
  starPop.querySelector('.sp-new').classList.toggle('hide', !isNew);
  if (isNew) $('#sp-newname').focus();
});
starPop.addEventListener('click', (e) => {
  const b = e.target.closest('[data-sp]'); if (!b || !S.starEdit) return;
  if (b.dataset.sp === 'remove') {
    send({ t: 'delBookmark', url: S.starEdit.url });
    S.starEdit.removed = true;
    toast('Bookmark removed');
  }
  closeStar(true);
});
starPop.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && e.target.tagName === 'INPUT') { e.preventDefault(); closeStar(true); }
});
addEventListener('mousedown', (e) => {
  if (S.overlay === 'star-open' && !starPop.contains(e.target) && !starBtn.contains(e.target)) closeStar(false);
}, true);
addEventListener('keydown', (e) => { if (e.key === 'Escape' && S.overlay === 'star-open') { e.preventDefault(); closeStar(true); } });
addEventListener('resize', () => S.overlay === 'star-open' && placeStar());
