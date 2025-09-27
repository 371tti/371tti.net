// 初期設定: box要素とその全ての子要素をスタイルして隠す
const hideBoxAndAllChildren = (box) => {
    // box自身と全ての子孫要素を取得
    const elements = [box, ...Array.from(box.querySelectorAll('*'))];
    
    // 各要素に対してアニメーション初期スタイルを設定
    elements.forEach((el, index) => {
        // 左から右に表示されるような初期状態
        el.style.opacity = '0';
        el.style.transform = 'translateX(-30px)';
        el.style.transition = `opacity 0.4s cubic-bezier(0.12, 0.63, 0, 1), 
                              transform 0.5s cubic-bezier(0.12, 0.63, 0, 1)`;
    });
};

// 要素の階層の深さを取得する関数
const getElementDepth = (element, parent) => {
    let depth = 0;
    let current = element;
    
    // 親要素に到達するまで遡る
    while (current !== parent && current.parentElement) {
        depth++;
        current = current.parentElement;
    }
    
    return depth;
};

// 新しい.box要素がDOMに追加された場合の処理
const observeNewBoxes = (node) => {
    if (node.classList?.contains('box')) {
        hideBoxAndAllChildren(node);
        observer.observe(node); // IntersectionObserverで監視
    }
};

// 既存の.box要素とその子要素を初期化
document.querySelectorAll('.box').forEach(hideBoxAndAllChildren);

// IntersectionObserverの作成
const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            const box = entry.target;
            const elements = [box, ...Array.from(box.querySelectorAll('*'))];
            
            elements.forEach((el, index) => {
                const delay = index * 0;
                setTimeout(() => {
                    el.style.opacity = '1';
                    el.style.transform = 'translateX(0)';
                }, delay);
            });

            observer.unobserve(entry.target);

            // アニメーション完了後にインラインスタイルをクリーンアップ
            setTimeout(() => {
                elements.forEach(el => {
                    el.style.removeProperty('opacity');
                    el.style.removeProperty('transform');
                    el.style.removeProperty('transition');
                });
            }, 600);
        }
    });
}, { threshold: 0, rootMargin: '0px' });

// MutationObserverの作成
const mutationObserver = new MutationObserver((mutations) => {
    mutations.forEach(mutation => {
        mutation.addedNodes.forEach(node => {
            if (node.nodeType === 1) { // Elementノードのみ
                observeNewBoxes(node);
            }
        });
    });
});

// body以下の変更を監視（すぐに開始）
mutationObserver.observe(document.body, {
    childList: true,
    subtree: true
});

// CSSで画面全体に表示するためのスタイルを定義
const fullScreenStyle = {
    position: "fixed",
    top: "0",
    left: "0",
    width: "100vw",
    height: "100vh",
    zIndex: "9999",
};

// 元のスタイルを保持するためのマップ
const originalStyles = new Map();