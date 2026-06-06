// band.js — 底部嵌入式滚动横幅（独立小窗，只滚动显示自选股行情）
// 数据自取：从 SQLite 读自选/设置，按设置的数据源拉行情；点击任意处还原主窗。
import { inTauri, invoke } from './bridge.js';

const track = document.getElementById('track');

function render(quotes, codes) {
  if (!quotes || !quotes.length) {
    track.innerHTML = '<span class="hint">暂无自选股 · 点击返回主窗添加</span>';
    track.style.animation = 'none';
    return;
  }
  const make = () => quotes.map((q, i) => {
    const up = q.change_pct >= 0;
    return `<span class="it">
      <span class="n">${q.name || (codes[i] || '').toUpperCase()}</span>
      <span class="v ${up ? 'up' : 'down'}">${q.price.toFixed(2)}</span>
      <span class="c ${up ? 'up' : 'down'}">${up ? '+' : ''}${q.change_pct.toFixed(2)}%</span>
    </span>`;
  }).join('');
  track.style.animation = '';
  track.innerHTML = make() + make(); // 双份内容做无缝滚动
}

async function load() {
  if (!inTauri) { // 浏览器预览
    render([
      { name: '贵州茅台', price: 1685.0, change_pct: 1.85 },
      { name: '宁德时代', price: 228.4, change_pct: -0.92 },
      { name: '比亚迪', price: 352.1, change_pct: 0.43 },
    ], []);
    return;
  }
  try {
    const wl = JSON.parse((await invoke('db_get', { key: 'watchlist' })) || '[]');
    if (!wl.length) { render([], []); return; }
    const settings = JSON.parse((await invoke('db_get', { key: 'settings' })) || '{}');
    const quotes = await invoke('fetch_quotes', { source: settings.source || 'sina', codes: wl });
    if (Array.isArray(quotes) && quotes.length) render(quotes, wl);
  } catch (e) {
    console.warn('band 行情失败:', e);
  }
}

document.getElementById('band').addEventListener('click', () => {
  if (inTauri) invoke('restore_main');
});

load();
setInterval(load, 10_000);
// 横幅显示出来时立即刷一次（主窗隐藏期间自选可能变了）
document.addEventListener('visibilitychange', () => { if (!document.hidden) load(); });
