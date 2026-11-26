'use strict';
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
  x: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
  star: '<path d="M12 3.8l2.5 5.1 5.6.8-4 3.9.9 5.6-5-2.6-5 2.6.9-5.6-4-3.9 5.6-.8z"/>',
  shield: '<path d="M12 3.5l7 2.8v5.2c0 4.3-3 7.9-7 9-4-1.1-7-4.7-7-9V6.3z"/><path d="M9 12l2 2 4-4"/>',
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
  eye: '<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.8"/>',
};
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
    renderTabs(); renderBar(); renderStart();
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
  suggest(m) { if (m.q === addr.value.trim()) renderSugg(m.items); },
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
  panel(m) { if (m.p === 'settings') openSettings(); else togglePanel(m.p); },
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
  return letter(t.url, t.title);
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
      el.innerHTML = `<span class="fav"></span><span class="title"></span><button class="x" title="Close (Ctrl+W)">${icon('x')}</button>`;
      el.addEventListener('animationend', () => el.classList.remove('enter'), { once: true });
      tabEls.set(t.id, el);
    }
    if (list.children[i] !== el) list.insertBefore(el, list.children[i] || null);
    el.classList.toggle('active', t.id === S.active);
    el.classList.toggle('asleep', !!t.asleep);
    const title = t.title || (t.url ? pretty(t.url) : 'New tab');
    if (el._title !== title) { el._title = title; el.title = title; el.querySelector('.title').textContent = title; }
    const fk = t.loading ? 'spin' : `${t.url}|${t.fav}`;
    if (el._fav !== fk) { el._fav = fk; el.querySelector('.fav').innerHTML = favHtml(t); }
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
  if (e.target.closest('.x')) return;
  if (id !== S.active) send({ t: 'activate', id });
});
$('#tablist').addEventListener('click', (e) => {
  const x = e.target.closest('.x'); if (!x) return;
  send({ t: 'close', id: +x.closest('.tab').dataset.id });
});
addEventListener('resize', () => requestAnimationFrame(() => {
  for (const el of tabEls.values()) el.classList.toggle('narrow', el.offsetWidth < 74);
}));

// ---- toolbar ---------------------------------------------------------------
const addr = $('#addr');
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
  document.body.classList.toggle('waking', !!(t && t.url && !t.shown));
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
$('#back').onclick = () => send({ t: 'nav', a: 'back' });
$('#fwd').onclick = () => send({ t: 'nav', a: 'forward' });
$('#reload').onclick = () => send({ t: 'nav', a: 'reload' });
$('#home').onclick = () => send({ t: 'nav', a: 'home' });
$('#star').onclick = () => send({ t: 'star' });
$('#shield').onclick = () => (S.overlay === 'shield-open' ? closeShield() : openShield());

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
$('#start-search').onclick = () => addr.focus();
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
  send({ t: 'win', a: 'drag' });
});
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
  const r = omni.getBoundingClientRect();
  Object.assign(sugg.style, { left: r.left + 'px', top: r.bottom + 6 + 'px', width: r.width + 'px' });
}
function renderSugg(items) {
  const q = addr.value.trim();
  if (!q) { S.rows = []; sugg.classList.remove('has'); return; }
  const engine = ENGINES[S.settings?.search] || 'your search engine';
  const first = looksLikeUrl(q)
    ? { kind: 'go', icon: 'arrow', t: q, u: 'Open address' }
    : { kind: 'search', icon: 'search', t: q, u: `Search with ${engine}` };
  S.rows = [first, ...items.map((i) => ({ kind: 'url', icon: 'globe', t: i.title || pretty(i.url), u: pretty(i.url), url: i.url }))];
  S.sel = Math.min(S.sel, S.rows.length - 1);
  sugg.innerHTML = S.rows.map((r, i) =>
    `<div class="row${i === S.sel ? ' sel' : ''}" data-i="${i}">${r.kind === 'url' ? `<span class="fav">${letter(r.url, r.t)}</span>` : icon(r.icon)}` +
    `<div class="txt"><span class="t">${esc(r.t)}</span><span class="u">${esc(r.u)}</span></div></div>`).join('');
  placeSugg();
  sugg.classList.add('has');
}
function pick(i, newTab) {
  const r = S.rows[i] || { kind: 'search' };
  const input = addr.value.trim();
  if (!input && r.kind !== 'url') return;
  if (r.kind === 'url') send({ t: 'open', url: r.url, newTab });
  else send({ t: 'go', input, newTab });
  addr.blur();
}
let suggT;
addr.addEventListener('focus', () => {
  if (S.overlay === 'shield-open') closeShield();
  if (settingsOpen()) closeSettings();
  const t = cur();
  addr.value = t?.url || '';
  addr.select();
  omni.classList.add('focus');
  S.sel = 0; S.rows = []; sugg.classList.remove('has');
  openOverlay();
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
addr.addEventListener('input', () => {
  S.sel = 0;
  const q = addr.value.trim();
  renderSugg(S.rows.length > 1 ? S.rows.slice(1).map((r) => ({ url: r.url, title: r.t })) : []);
  clearTimeout(suggT);
  if (q) suggT = setTimeout(() => send({ t: 'suggest', q }), 35);
});
addr.addEventListener('keydown', (e) => {
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
    const t = cur();
    if (addr.value !== (t?.url || '') && addr.value) { addr.value = t?.url || ''; addr.select(); renderSugg([]); return; }
    S.overlay && closeOverlay(true);
    addr.blur();
  }
});
sugg.addEventListener('mousedown', (e) => e.preventDefault());
sugg.addEventListener('click', (e) => {
  const r = e.target.closest('.row'); if (r) pick(+r.dataset.i, e.ctrlKey);
});
addEventListener('resize', () => S.overlay && placeSugg());

// ---- start page ------------------------------------------------------------
function renderStart() {
  const t = cur();
  const blank = !t || !t.url;
  document.body.classList.toggle('blank', blank);
  if (blank && !S.blank) send({ t: 'list', kind: 'top' });
  if (blank && !S.blankInit) { S.blankInit = true; send({ t: 'list', kind: 'top' }); }
  S.blank = blank;
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
function renderTiles() {
  S.tiles = tileList();
  $('#tiles').innerHTML = S.tiles.map((i, n) => {
    const host = hostOf(i.url);
    return `<button class="tile${i.pinned ? ' pinned' : ''}" data-i="${n}" data-url="${esc(i.url)}" title="${esc(i.title || i.url)}"><span class="ic glass">` +
      `<img src="https://${esc(host)}/favicon.ico" alt="" onerror="this.outerHTML=${esc(JSON.stringify(letter(i.url, i.title)))}"></span><span>${esc(tileName(i))}</span></button>`;
  }).join('') + `<button class="tile add" data-add title="Add a favourite"><span class="ic glass">${icon('plus')}</span><span>Add</span></button>`;
}
$('#tiles').addEventListener('click', (e) => {
  if (e.target.closest('[data-add]')) { editTile(null); return; }
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
const ctx = $('#ctx');
function showMenu(x, y, items) {
  ctx._items = items;
  ctx.innerHTML = items.map((it, i) => it === '-' ? '<hr>'
    : `<button data-i="${i}" class="${it.danger ? 'danger' : ''}">${icon(it.icon)}${esc(it.label)}</button>`).join('');
  document.body.classList.add('ctx-open');
  ctx.style.left = Math.max(6, Math.min(x, innerWidth - ctx.offsetWidth - 6)) + 'px';
  ctx.style.top = Math.max(6, Math.min(y, innerHeight - ctx.offsetHeight - 6)) + 'px';
}
function hideMenu() { document.body.classList.remove('ctx-open'); }
ctx.addEventListener('click', (e) => {
  const b = e.target.closest('button[data-i]'); if (!b) return;
  const it = ctx._items[+b.dataset.i]; hideMenu(); it.run();
});
addEventListener('mousedown', (e) => { if (!ctx.contains(e.target)) hideMenu(); }, true);
addEventListener('blur', hideMenu);

// ---- add / edit favourite dialog ------------------------------------------------
const dlg = $('#tile-dlg');
function editTile(t) {
  dlg._orig = t ? t.url : null;
  dlg.innerHTML = `<h3>${t ? 'Edit favourite' : 'Add a favourite'}</h3>` +
    `<label>Address<input class="field" id="td-url" value="${esc(t?.url || '')}" placeholder="youtube.com" spellcheck="false"></label>` +
    `<label>Name<input class="field" id="td-name" value="${esc(t ? tileName(t) : '')}" placeholder="Optional" spellcheck="false"></label>` +
    `<div class="err" id="td-err"></div>` +
    `<div class="btns"><button class="btn" data-d="cancel">Cancel</button><button class="btn primary" data-d="save">Save</button></div>`;
  document.body.classList.add('dlg-open');
  setTimeout(() => $('#td-url').focus(), 30);
}
function closeDlg() { document.body.classList.remove('dlg-open'); }
function saveDlg() {
  let u = $('#td-url').value.trim();
  if (!u) { $('#td-err').textContent = 'Type an address, like youtube.com'; return; }
  if (!/^[a-z][a-z0-9+.-]*:/i.test(u)) u = 'https://' + u;
  try { if (!new URL(u).hostname.includes('.') && !/^https?:\/\/localhost/i.test(u)) throw 0; }
  catch { $('#td-err').textContent = "That doesn't look like a web address"; return; }
  const name = $('#td-name').value.trim() || hostOf(u).split('.')[0].replace(/^./, (c) => c.toUpperCase());
  savePin(dlg._orig, name, u);
  closeDlg();
}
dlg.addEventListener('click', (e) => {
  const d = e.target.closest('[data-d]')?.dataset.d;
  if (d === 'cancel') closeDlg();
  if (d === 'save') saveDlg();
});
dlg.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') { e.preventDefault(); saveDlg(); }
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
  S.panel = null;
  document.body.classList.remove('side-open');
}
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
    html += `<div class="row" data-url="${esc(i.url)}" title="${esc(i.url)}"><span class="fav">${letter(i.url, i.title)}</span>` +
      `<div class="txt"><span class="t">${esc(i.title || pretty(i.url))}</span><span class="u">${esc(hostOf(i.url))}</span></div>` +
      `<span class="end">${time}</span><button class="icon sm del" title="Remove">${icon('x')}</button></div>`;
  }
  if (kind === 'history') {
    html += `<div class="btns" style="padding:14px 4px"><button class="btn danger" data-act="clearHistory">${icon('trash')}Clear all history</button></div>`;
  }
  body.innerHTML = html;
}
$('#side-body').addEventListener('click', (e) => {
  const act = e.target.closest('[data-act]');
  if (act) { runAction(act); return; }
  const row = e.target.closest('.row[data-url]'); if (!row) return;
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
  send({ t: btn.dataset.act });
}

// ---- settings --------------------------------------------------------------
// A centered window over a blurred view of the browser, with a section list on the left.
const ACCENTS = ['#8ab4ff', '#a78bfa', '#f472b6', '#fb7185', '#fb923c', '#facc15', '#4ade80', '#2dd4bf', '#e2e8f0'];
const F = {
  adblock: { k: 'adblock', label: 'Block ads & trackers', desc: () => S.shieldReady ? 'EasyList, EasyPrivacy and Peter Lowe\'s list' : 'Downloading filter lists...', type: 'toggle' },
  tracking: { k: 'tracking', label: 'Tracking prevention', desc: 'Built into the engine; Strict may break some sites', type: 'select', opts: [['strict', 'Strict'], ['balanced', 'Balanced'], ['basic', 'Basic'], ['off', 'Off']] },
  updateLists: { type: 'buttons', items: [['updateLists', 'refresh', 'Update filter lists']] },
};
const SECTIONS = [
  { id: 'appearance', icon: 'palette', title: 'Appearance', groups: [
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
  { id: 'browsing', icon: 'globe', title: 'Browsing', groups: [
    ['Search', [{ k: 'search', label: 'Search engine', desc: 'Used when you type words in the address bar', type: 'select', opts: Object.entries(ENGINES) }]],
    ['Startup', [
      { k: 'restore_session', label: 'Reopen last session\'s tabs', type: 'toggle' },
      { k: 'homepage', label: 'Home button opens', desc: 'Leave empty for the start page', type: 'text', placeholder: 'Start page' },
    ]],
  ] },
  { id: 'privacy', icon: 'shield', title: 'Privacy & trackers', groups: [
    ['Blocking', [F.adblock, { type: 'allow' }, F.updateLists]],
    ['Tracking', [F.tracking]],
    ['Clear browsing data', [{ type: 'buttons', items: [['clearHistory', 'trash', 'Clear history', true], ['clearData', 'trash', 'Clear cookies & cache', true]] }]],
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
    ['Memory', [{ k: 'sleep_minutes', label: 'Sleep background tabs', desc: 'Frees memory; the tab reloads when you return', type: 'select', num: true, opts: [[0, 'Never'], [5, 'After 5 min'], [15, 'After 15 min'], [30, 'After 30 min'], [60, 'After 1 hour']] }]],
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
  if (S.overlay === 'omni-open') addr.blur();
  if (!settingsOpen()) openOverlay('settings-open');
  renderSetNav();
  renderSettings(true);
  if (S.setSec === 'data') send({ t: 'importSources' });
}
function closeSettings() { if (settingsOpen()) closeOverlay(false); }
function renderSetNav() {
  $('#set-tabs').innerHTML = SECTIONS.map((s) =>
    `<button data-sec="${s.id}" class="${s.id === S.setSec ? 'on' : ''}">${icon(s.icon)}${esc(s.title)}</button>`).join('');
}
$('#set-tabs').addEventListener('click', (e) => {
  const b = e.target.closest('[data-sec]'); if (!b || b.dataset.sec === S.setSec) return;
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
  for (const [title, items] of groups) {
    const inner = items.map((it) => field(it, s)).join('');
    if (!inner) continue;
    html += `<div class="grp">${title ? `<h4>${esc(title)}</h4>` : ''}<div class="card">${inner}</div></div>`;
  }
  return html;
}
// Re-render whichever settings views are open (the window and/or the Trackers panel).
function renderSettings(top) {
  const s = S.settings; if (!s) return;
  if (settingsOpen()) {
    const sec = SECTIONS.find((x) => x.id === S.setSec) || SECTIONS[0];
    $('#set-h').textContent = sec.title;
    const body = $('#set-body');
    const y = top ? 0 : body.scrollTop;
    let html = groupsHtml(sec.groups, s);
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
const KEYS = [['New tab', 'Ctrl T'], ['Close tab', 'Ctrl W'], ['Reopen closed tab', 'Ctrl Shift T'], ['Address bar', 'Ctrl L'],
  ['Next tab', 'Ctrl Tab'], ['Bookmark page', 'Ctrl D'], ['History', 'Ctrl H'], ['Bookmarks', 'Ctrl Shift B'], ['Settings', 'Ctrl ,'], ['Reload', 'F5']];
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
    case 'range': {
      const p = ((s[it.k] - it.min) / (it.max - it.min)) * 100;
      return `<div class="set col"><div style="display:flex">${label}<span class="val">${s[it.k]}${it.unit}</span></div>` +
        `<input type="range" data-k="${it.k}" min="${it.min}" max="${it.max}" value="${s[it.k]}" style="--p:${p}%" data-unit="${it.unit}"></div>`;
    }
    case 'select':
      return `<div class="set">${label}<select class="field" style="width:auto;min-width:150px" data-k="${it.k}" ${it.num ? 'data-num' : ''}>` +
        it.opts.map(([v, l]) => `<option value="${v}" ${String(s[it.k]) === String(v) ? 'selected' : ''}>${esc(l)}</option>`).join('') + '</select></div>';
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
      let h = list.map((p, i) => `<div class="lrow"><span class="fav">${letter(p.url, p.title)}</span>` +
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
    case 'about':
      return `<div class="about"><span class="logo">${icon('sparkle')}</span><div><b>Lurt 0.1.0</b><small>A light, fast glass browser written in Rust, running on Microsoft Edge WebView2.</small></div></div>` +
        `<div class="btns"><button class="btn" data-open="https://github.com/ladroxd/Lurt">${icon('external')}Project page</button><button class="btn" data-open="https://github.com/ladroxd/Lurt/releases">${icon('download')}Releases</button></div>`;
    case 'keys':
      return `<div class="keys">` + KEYS.map(([a, k]) => `<span>${esc(a)}</span><kbd>${esc(k)}</kbd>`).join('') + `</div>`;
    case 'note':
      return `<div class="hint">${esc(val(it.text))}</div>`;
  }
  return '';
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
    const t = e.target; const k = t.dataset.k; if (!k) return;
    if (t.type === 'checkbox') setSetting(k, t.checked, { rerender: k === 'adblock' });
    else if (t.tagName === 'SELECT') setSetting(k, t.hasAttribute('data-num') ? +t.value : t.value);
    else if (t.type === 'color') setSetting(k, t.value, { rerender: true });
    else if (t.type === 'range') setSetting(k, +t.value);
  });
  el.addEventListener('input', (e) => {
    if (!active()) return;
    const t = e.target; const k = t.dataset.k; if (!k) return;
    if (t.type === 'range') {
      t.style.setProperty('--p', ((t.value - t.min) / (t.max - t.min)) * 100 + '%');
      t.closest('.set').querySelector('.val').textContent = t.value + t.dataset.unit;
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
const FORWARD = new Set(['ctrl+t', 'ctrl+w', 'ctrl+tab', 'ctrl+shift+tab', 'ctrl+d', 'ctrl+shift+t',
  'ctrl+1', 'ctrl+2', 'ctrl+3', 'ctrl+4', 'ctrl+5', 'ctrl+6', 'ctrl+7', 'ctrl+8', 'ctrl+9']);
addEventListener('keydown', (e) => {
  let k = e.key.toLowerCase(); if (k === ',') k = 'comma';
  const c = (e.ctrlKey ? 'ctrl+' : '') + (e.altKey ? 'alt+' : '') + (e.shiftKey ? 'shift+' : '') + k;
  if (FORWARD.has(c)) { e.preventDefault(); send({ t: 'key', k: c }); return; }
  if (c === 'ctrl+l' || c === 'alt+d' || c === 'f6') { e.preventDefault(); addr.focus(); addr.select(); return; }
  if (c === 'ctrl+h') { e.preventDefault(); togglePanel('history'); return; }
  if (c === 'ctrl+shift+b') { e.preventDefault(); togglePanel('bookmarks'); return; }
  if (c === 'ctrl+comma') { e.preventDefault(); settingsOpen() ? closeSettings() : openSettings(); return; }
  if (c === 'escape' && document.body.classList.contains('ctx-open')) { hideMenu(); return; }
  if (c === 'escape' && settingsOpen() && !document.body.classList.contains('dlg-open')) { closeSettings(); return; }
  if (settingsOpen() || document.body.classList.contains('dlg-open')) return;
  if (c === 'f5' || c === 'ctrl+r') { e.preventDefault(); send({ t: 'nav', a: 'reload' }); return; }
  if (c === 'alt+arrowleft') { e.preventDefault(); send({ t: 'nav', a: 'back' }); return; }
  if (c === 'alt+arrowright') { e.preventDefault(); send({ t: 'nav', a: 'forward' }); return; }
  if (c === 'escape' && S.panel && document.activeElement !== addr) { closePanel(); return; }
  // Typing anywhere on the start page goes to the address bar.
  const typing = document.activeElement && /INPUT|TEXTAREA|SELECT/.test(document.activeElement.tagName);
  if (S.blank && !typing && !e.ctrlKey && !e.altKey && e.key.length === 1) addr.focus();
});
addEventListener('contextmenu', (e) => { if (!e.target.closest('input, textarea')) e.preventDefault(); });

send({ t: 'ready' });
