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
    if (S.panel === 'settings' && !$('#side').contains(document.activeElement)) renderSettings();
  },
  list(m) {
    if (m.kind === 'top') renderTiles(m.items);
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
  panel(m) { togglePanel(m.p); },
  mods(m) { $('#mods-css').textContent = m.css || ''; },
  toast(m) { toast(m.text); },
  sources(m) {
    S.sources = m.items;
    if (S.panel === 'settings' && !$('#side').contains(document.activeElement)) renderSettings();
  },
  imported(m) {
    S.importing = null;
    S.importMsg = m.text;
    toast(m.text);
    if (S.panel === 'settings') renderSettings();
  },
  blocked(m) {
    const t = S.tabs.find((x) => x.id === m.id);
    if (t) t.blocked = m.n;
    if (m.id === S.active) renderShield();
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
  const t = cur(); const b = $('#shield'); const n = $('#shield-n');
  const on = S.settings ? S.settings.adblock && !S.allowed : true;
  b.classList.toggle('off', !on);
  const k = on ? 'shield' : 'shieldOff';
  if (b._k !== k) { b._k = k; b.querySelector('svg').outerHTML = icon(k); }
  const count = on && t ? t.blocked : 0;
  n.textContent = count > 99 ? '99+' : count || '';
  n.classList.toggle('on', count > 0);
  b.title = !S.settings?.adblock ? 'Blocking is off (Settings → Privacy)'
    : S.allowed ? `Blocking paused on ${hostOf(t?.url || '')}. Click to resume.`
    : `${count} blocked on this page. Click to pause on this site.`;
}
$('#newtab').onclick = () => send({ t: 'key', k: 'ctrl+t' });
$('#back').onclick = () => send({ t: 'nav', a: 'back' });
$('#fwd').onclick = () => send({ t: 'nav', a: 'forward' });
$('#reload').onclick = () => send({ t: 'nav', a: 'reload' });
$('#home').onclick = () => send({ t: 'nav', a: 'home' });
$('#star').onclick = () => send({ t: 'star' });
$('#shield').onclick = () => {
  if (!S.settings?.adblock) { togglePanel('settings'); return; }
  if (cur()?.url) send({ t: 'shield' });
};
$('#menu').onclick = () => togglePanel(S.panel || S.lastPanel);
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
function openOverlay() {
  if (S.overlay) return;
  S.overlay = true;
  document.body.classList.add('omni-open');
  send({ t: 'overlay', open: true });
}
function closeOverlay(focusPage) {
  if (!S.overlay) return;
  S.overlay = false;
  document.body.classList.remove('omni-open');
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
  const t = cur();
  addr.value = t?.url || '';
  addr.select();
  omni.classList.add('focus');
  S.sel = 0; S.rows = []; sugg.classList.remove('has');
  openOverlay();
});
let keepSelection = false;
addr.addEventListener('mousedown', () => { keepSelection = document.activeElement !== addr; });
addr.addEventListener('mouseup', (e) => { if (keepSelection) { e.preventDefault(); addr.select(); keepSelection = false; } });
addr.addEventListener('blur', () => {
  omni.classList.remove('focus');
  closeOverlay(false);
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
function renderTiles(items) {
  $('#tiles').innerHTML = items.slice(0, 8).map((i) => {
    const host = hostOf(i.url);
    const name = (i.title || host).split(/[|\-–—:·]/)[0].trim() || host;
    return `<button class="tile" data-url="${esc(i.url)}" title="${esc(i.title || i.url)}"><span class="ic glass">` +
      `<img src="https://${esc(host)}/favicon.ico" alt="" onerror="this.outerHTML=${esc(JSON.stringify(letter(i.url, i.title)))}"></span><span>${esc(name)}</span></button>`;
  }).join('');
}
$('#tiles').addEventListener('click', (e) => {
  const b = e.target.closest('.tile'); if (b) send({ t: 'open', url: b.dataset.url, newTab: e.ctrlKey });
});
$('#tiles').addEventListener('auxclick', (e) => {
  const b = e.target.closest('.tile'); if (b && e.button === 1) send({ t: 'open', url: b.dataset.url, newTab: true });
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
  document.body.classList.remove('panel-bookmarks', 'panel-history', 'panel-settings');
  document.body.classList.add('panel-' + p);
  const btns = [...document.querySelectorAll('#side-seg button')];
  btns.forEach((b) => b.classList.toggle('on', b.dataset.p === p));
  movePill($('#side-seg'));
  $('#side-body').scrollTop = 0;
  if (p === 'settings') { renderSettings(); send({ t: 'importSources' }); }
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
const ACCENTS = ['#8ab4ff', '#a78bfa', '#f472b6', '#fb7185', '#fb923c', '#facc15', '#4ade80', '#2dd4bf', '#e2e8f0'];
const SCHEMA = [
  ['Appearance', [
    { k: 'theme', label: 'Theme', type: 'seg', opts: [['system', 'System'], ['light', 'Light'], ['dark', 'Dark']] },
    { k: 'backdrop', label: 'Glass', type: 'seg', opts: [['blur', 'Blur'], ['acrylic', 'Acrylic'], ['mica', 'Mica'], ['none', 'Solid']], hint: backdropHint },
    { k: 'tint', label: 'Background opacity', desc: 'Lower shows more of what is behind the window', type: 'range', min: 0, max: 100, unit: '%' },
    { k: 'window_opacity', label: 'Window opacity', desc: 'Fades the whole window, websites included', type: 'range', min: 20, max: 100, unit: '%' },
    { k: 'radius', label: 'Roundness', type: 'range', min: 0, max: 24, unit: 'px' },
    { k: 'accent', label: 'Accent color', type: 'color' },
    { k: 'compact', label: 'Compact layout', desc: 'Tabs and address bar share one row', type: 'toggle' },
    { k: 'rounded_pages', label: 'Rounded page corners', type: 'toggle' },
    { k: 'show_clock', label: 'Clock on the start page', type: 'toggle' },
    { k: 'reduce_motion', label: 'Reduce motion', desc: 'Turn off animations', type: 'toggle' },
  ]],
  ['Browsing', [
    { k: 'search', label: 'Search engine', type: 'select', opts: Object.entries(ENGINES) },
    { k: 'homepage', label: 'Home button opens', desc: 'Leave empty for the start page', type: 'text', placeholder: 'Start page' },
    { k: 'restore_session', label: 'Reopen last session\'s tabs', type: 'toggle' },
  ]],
  ['Privacy', [
    { k: 'adblock', label: 'Block ads & trackers', desc: () => S.shieldReady ? 'EasyList, EasyPrivacy and Peter Lowe\'s list' : 'Downloading filter lists...', type: 'toggle' },
    { type: 'allow' },
    { k: 'tracking', label: 'Tracking prevention', desc: 'Built into the engine; Strict may break some sites', type: 'select', opts: [['strict', 'Strict'], ['balanced', 'Balanced'], ['basic', 'Basic'], ['off', 'Off']] },
    { type: 'buttons', items: [['updateLists', 'refresh', 'Update filter lists'], ['clearHistory', 'trash', 'Clear history', true], ['clearData', 'trash', 'Clear cookies & cache', true]] },
  ]],
  ['Your data', [
    { type: 'import' },
    { k: 'save_passwords', label: 'Offer to save passwords', desc: "Saved in Lurt's profile on this PC", type: 'toggle' },
    { k: 'autofill', label: 'Autofill forms', desc: 'Addresses and other details you type', type: 'toggle' },
  ]],
  ['Performance', [
    { k: 'sleep_minutes', label: 'Sleep background tabs', desc: 'Frees memory; the tab reloads when you return', type: 'select', num: true, opts: [[0, 'Never'], [5, 'After 5 min'], [15, 'After 15 min'], [30, 'After 30 min'], [60, 'After 1 hour']] },
  ]],
  ['Mods', [
    { k: 'ui_css', label: 'Custom CSS for the browser', desc: 'Applied live. Try: :root { --r: 22px }', type: 'code' },
    { type: 'buttons', items: [['openMods', 'folder', 'Open mods folder'], ['reloadMods', 'refresh', 'Reload mods']] },
    { type: 'note', text: () => `Put .css files in mods\\ui to theme the browser, and <domain>.css / .js in mods\\sites to change websites. Folder: ${S.modsDir}` },
  ]],
];
function backdropHint() {
  const want = S.settings.backdrop;
  if (want === 'mica' && S.backdrop !== 'mica') return 'Mica needs Windows 11. Using Blur instead.';
  if (want === 'acrylic') return 'Acrylic can stutter while dragging the window on Windows 10.';
  return '';
}
const val = (f) => (typeof f === 'function' ? f() : f) || '';

function renderSettings() {
  const s = S.settings; if (!s) return;
  let html = '';
  for (const [sec, items] of SCHEMA) {
    html += `<div class="sec"><h3>${sec}</h3>`;
    for (const it of items) html += field(it, s);
    html += '</div>';
  }
  html += `<div class="hint" style="padding:0 12px 16px">Lurt 0.1 · Shortcuts: Ctrl+T, Ctrl+W, Ctrl+L, Ctrl+Tab, Ctrl+D, Ctrl+H, Ctrl+Shift+T</div>`;
  const body = $('#side-body');
  const top = body.scrollTop;
  body.innerHTML = html;
  body.scrollTop = top;
  body.querySelectorAll('.seg').forEach(movePill);
}
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
      return `<div class="set">${label}<select class="field" data-k="${it.k}" ${it.num ? 'data-num' : ''}>` +
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
      return `<div class="set col"><div class="lbl"><small>Paused on these sites</small></div><div class="chips">` +
        s.adblock_allow.map((h) => `<span>${esc(h)}<button data-unallow="${esc(h)}" title="Resume blocking">${icon('x')}</button></span>`).join('') + '</div></div>';
    }
    case 'import': {
      const src = S.sources || [];
      const busy = S.importing;
      let h = `<div class="set col"><div class="lbl"><b>Import bookmarks & history</b><small>` +
        (src.length ? 'From browsers on this PC:' : 'No other browsers found on this PC.') + '</small></div>';
      if (src.length) {
        h += '<div class="btns" style="padding:0">' +
          (src.length > 1 ? `<button class="btn" data-import="*" data-label="all browsers" ${busy ? 'disabled' : ''}>${icon('arrow')}Import all (${src.length} profiles)</button>` : '') +
          src.map((x) => {
          const label = x.profile ? `${x.browser} · ${x.profile}` : x.browser;
          return `<button class="btn" data-import="${esc(x.id)}" data-label="${esc(label)}" ${busy ? 'disabled' : ''}>${icon('arrow')}${esc(label)}</button>`;
        }).join('') + '</div>';
      }
      h += `<div class="lbl" style="margin-top:6px"><small>From your Google account: export <b>Chrome</b> data at Google Takeout, then import <code>BrowserHistory.json</code> and <code>Bookmarks.html</code>. Any browser's bookmarks .html export works too.</small></div>` +
        `<div class="btns" style="padding:0"><button class="btn" data-open="https://takeout.google.com/settings/takeout/custom/chrome">${icon('globe')}Open Google Takeout</button>` +
        `<label class="btn" ${busy ? 'style="opacity:.5;pointer-events:none"' : ''}>${icon('folder')}Import a file<input type="file" id="import-file" accept=".json,.html,.htm" multiple hidden></label></div>`;
      if (busy) h += `<div class="hint"><i class="spin" style="display:inline-block;vertical-align:-3px;margin-right:6px"></i>Importing from ${esc(busy)}…</div>`;
      else if (S.importMsg) h += `<div class="hint">${esc(S.importMsg)}</div>`;
      h += `<div class="hint">Passwords and signed-in sessions can't be copied: Chrome encrypts them so only Chrome can read them. Sign in once in Lurt and it remembers you.</div>`;
      return h + '</div>';
    }
    case 'note':
      return `<div class="hint" style="padding:6px 6px 0">${esc(val(it.text))}</div>`;
  }
  return '';
}

let saveT;
function setSetting(k, v, { lazy = false, rerender = false } = {}) {
  S.settings[k] = v;
  applyLook();
  clearTimeout(saveT);
  saveT = setTimeout(() => send({ t: 'settings', s: S.settings }), lazy ? 300 : 0);
  if (rerender) renderSettings();
}
const side = $('#side-body');
side.addEventListener('change', (e) => {
  if (e.target.id === 'import-file') {
    for (const f of e.target.files) {
      S.importing = f.name; S.importMsg = '';
      f.text().then((text) => send({ t: 'importFile', name: f.name, text }));
    }
    renderSettings();
    return;
  }
  const el = e.target; const k = el.dataset.k; if (!k || S.panel !== 'settings') return;
  if (el.type === 'checkbox') setSetting(k, el.checked, { rerender: k === 'adblock' });
  else if (el.tagName === 'SELECT') setSetting(k, el.hasAttribute('data-num') ? +el.value : el.value);
  else if (el.type === 'color') setSetting(k, el.value, { rerender: true });
  else if (el.type === 'range') setSetting(k, +el.value);
});
side.addEventListener('input', (e) => {
  const el = e.target; const k = el.dataset.k; if (!k || S.panel !== 'settings') return;
  if (el.type === 'range') {
    el.style.setProperty('--p', ((el.value - el.min) / (el.max - el.min)) * 100 + '%');
    el.parentElement.querySelector('.val').textContent = el.value + el.dataset.unit;
    setSetting(k, +el.value, { lazy: true });
  } else if (el.tagName === 'TEXTAREA' || el.type === 'text') {
    setSetting(k, el.value, { lazy: true });
  } else if (el.type === 'color') {
    setSetting(k, el.value, { lazy: true });
  }
});
side.addEventListener('click', (e) => {
  if (S.panel !== 'settings') return;
  const segBtn = e.target.closest('.seg[data-k] button');
  if (segBtn) {
    const seg = segBtn.parentElement;
    seg.querySelectorAll('button').forEach((b) => b.classList.toggle('on', b === segBtn));
    movePill(seg);
    setSetting(seg.dataset.k, segBtn.dataset.v, { rerender: seg.dataset.k === 'backdrop' });
    return;
  }
  const sw = e.target.closest('.sw[data-color]');
  if (sw) { setSetting('accent', sw.dataset.color, { rerender: true }); return; }
  const imp = e.target.closest('[data-import]');
  if (imp) {
    S.importing = imp.dataset.label; S.importMsg = '';
    send({ t: 'importProfile', id: imp.dataset.import, label: imp.dataset.label });
    renderSettings();
    return;
  }
  const op = e.target.closest('[data-open]');
  if (op) { send({ t: 'open', url: op.dataset.open, newTab: true }); return; }
  const un = e.target.closest('[data-unallow]');
  if (un) { setSetting('adblock_allow', S.settings.adblock_allow.filter((h) => h !== un.dataset.unallow), { rerender: true }); }
});

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
  if (c === 'ctrl+comma') { e.preventDefault(); togglePanel('settings'); return; }
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
