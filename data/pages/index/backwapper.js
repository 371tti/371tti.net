// File: data/pages/index/backwapper.js
window.addEventListener('load', () => {
    /* ===== ここから ".main のスクロール ×0.3" 処理 ===== */
    const root     = document.documentElement;
    const scroller = document.querySelector('.main');
    if (!scroller) {
        console.warn('.main 要素が見つかりません');
        return;
    }

    const base    = scroller.scrollTop;   // 読み込み時の位置を 0
    let ticking   = false;

    function sync() {
        const offsetY = (scroller.scrollTop - base) * -0.1;
        const offsetX = (scroller.scrollTop - base) * 0.05; // X軸方向にも移動
        root.style.setProperty('--scrollOffsetX', offsetX.toFixed(1) + 'px');
        root.style.setProperty('--scrollOffsetY', offsetY.toFixed(1) + 'px');
        ticking = false;
    }

    scroller.addEventListener('scroll', () => {
        if (!ticking) {
            requestAnimationFrame(sync);
            ticking = true;
        }
    }, { passive: true });

    sync();          // 初回反映
});