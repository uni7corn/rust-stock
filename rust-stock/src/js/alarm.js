// alarm.js — 自选股涨跌报警（纯本地，交易日内，每股 5 分钟去重）
import { fetchQuotes } from './api.js';
import { state } from './store.js';
import { flashHint } from './ui.js';
import { isTradingDay } from './tradingcal.js';

const lastAlert = {}; // code -> 上次报警时间戳
const COOLDOWN = 5 * 60 * 1000;

function notify(q) {
  const dir = q.change_pct >= 0 ? '涨' : '跌';
  const msg = `${q.name} ${dir} ${q.change_pct >= 0 ? '+' : ''}${q.change_pct.toFixed(2)}%（现价 ${q.price.toFixed(2)}）`;
  flashHint('⚠️ 涨跌报警 · ' + msg);
  try {
    if (typeof Notification !== 'undefined') {
      if (Notification.permission === 'granted') new Notification('rust-stock 涨跌报警', { body: msg });
      else if (Notification.permission !== 'denied') Notification.requestPermission().then(p => { if (p === 'granted') new Notification('rust-stock 涨跌报警', { body: msg }); });
    }
  } catch (e) { /* 部分 WebView 无 Notification，靠应用内提示即可 */ }
}

export async function checkAlarms() {
  const s = state.settings;
  if (!s || !s.alarm) return;
  if (!isTradingDay()) return;            // 非交易日不报警
  const codes = state.watchlist;
  if (!codes || !codes.length) return;
  const th = Math.max(0.5, +s.alarmPct || 5);
  const quotes = await fetchQuotes(codes);
  if (!quotes || !quotes.length) return;
  const now = Date.now();
  for (const q of quotes) {
    if (typeof q.change_pct !== 'number') continue;
    if (Math.abs(q.change_pct) < th) continue;
    if (now - (lastAlert[q.code] || 0) < COOLDOWN) continue;
    lastAlert[q.code] = now;
    notify(q);
  }
}
