// bridge.js — Tauri 桥接层。浏览器预览时安全降级（mock）。
export const inTauri = !!(window.__TAURI__ || window.__TAURI_INTERNALS__);

// 平台检测（移动端隐藏桌面专属 UI、关闭等比缩放）。Tauri 安卓 webview UA 含 "Android"。
const _ua = navigator.userAgent || '';
export const isAndroid = /android/i.test(_ua);
export const isMobile = isAndroid || /iphone|ipad|ipod/i.test(_ua);

export async function invoke(cmd, args) {
  if (!inTauri) { console.log('[mock invoke]', cmd, args || ''); return; }
  return window.__TAURI__.core.invoke(cmd, args);
}

export async function listen(event, cb) {
  if (!inTauri) return;
  return window.__TAURI__.event.listen(event, cb);
}
