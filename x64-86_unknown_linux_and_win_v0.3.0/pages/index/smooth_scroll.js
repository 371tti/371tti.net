/**
 * スムーズスクロール
 * マウスホイールなどによるスクロールを滑らかにします
 */

// スムーズスクロールの設定
document.addEventListener('DOMContentLoaded', () => {
  // 設定（必要に応じて調整）
  const inertiaConfig = {
    friction:    0.80,   // 減速率（0〜1）。値が小さいほど早く減速
    minVelocity: 0.1,    // この速度以下で慣性を停止
    wheelStep:   0.3,    // deltaY に掛ける倍率（ステップサイズ）
    maxVelocity: 1000    // vel の最大／最小値（±）
  };

  // スクロール対象を収集：window と overflow 要素
  const containers = new Set([window]);
  document.querySelectorAll('*').forEach(el => {
    const style = getComputedStyle(el);
    if (el.scrollHeight > el.clientHeight &&
       (style.overflowY === 'auto' || style.overflowY === 'scroll')) {
      containers.add(el);
    }
  });

  // 各コンテナに慣性スクロールを初期化
  containers.forEach(initInertia);

  function initInertia(container) {
    let vel = 0;
    let rafId = null;

    container.addEventListener('wheel', e => {
      e.preventDefault();        // デフォルトを無効化
      // ステップ倍率と最大速度を適用
      vel += e.deltaY * inertiaConfig.wheelStep;
      vel = Math.max(-inertiaConfig.maxVelocity,
                    Math.min( vel, inertiaConfig.maxVelocity ));
      // 即時スクロール
      scrollBy(container, e.deltaY);
      if (!rafId) rafId = requestAnimationFrame(step);
    }, { passive: false });

    function step() {
      vel *= inertiaConfig.friction;
      if (Math.abs(vel) < inertiaConfig.minVelocity) {
        rafId = null;
        vel = 0;
        return;
      }
      scrollBy(container, vel);
      rafId = requestAnimationFrame(step);
    }
  }

  function scrollBy(container, delta) {
    if (container === window) {
      window.scrollBy(0, delta);
    } else {
      container.scrollTop += delta;
    }
  }
});

// タッチデバイスの場合はスムーズスクロールを無効化する
if ('ontouchstart' in window || navigator.maxTouchPoints) {
  document.documentElement.style.scrollBehavior = 'auto';
}

// スクロール速度の最適化
window.addEventListener('scroll', () => {
  if (window.scrollTimeout) {
    clearTimeout(window.scrollTimeout);
  }
  
  // スクロール中はクラスを追加
  document.body.classList.add('is-scrolling');
  
  // スクロール停止時にクラスを削除
  window.scrollTimeout = setTimeout(() => {
    document.body.classList.remove('is-scrolling');
  }, 150);
}, { passive: true });
