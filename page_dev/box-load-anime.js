// 初期設定: box要素とその直下の子要素をスタイルして隠す
const hideBoxAndDirectChildren = (box) => {
    // box自身とその直下の子要素のみ取得
    const elements = [box, ...box.children];
    elements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(10mm)';
        el.style.transition = 'opacity 0.6s ease-out, transform 0.6s ease-out';
    });
};

// 新しい.box要素がDOMに追加された場合の処理
const observeNewBoxes = (node) => {
    if (node.classList?.contains('box')) {
        hideBoxAndDirectChildren(node);
        observer.observe(node); // IntersectionObserverで監視
    }
};

// 既存の.box要素とその直下の子要素を初期化
document.querySelectorAll('.box').forEach(hideBoxAndDirectChildren);

// IntersectionObserverの作成
const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            // フェードインスタイルを適用（対象とその直下の子要素）
            const elements = [entry.target, ...entry.target.children];
            elements.forEach(el => {
                el.style.opacity = '1';
                el.style.transform = 'translateY(0)';
            });

            observer.unobserve(entry.target); // 一度監視したら解除
        }
    });
}, { threshold: 0.1 });

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
