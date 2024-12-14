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



// CSSで画面全体に表示するためのスタイルを定義
const fullScreenStyle = {
    position: "fixed",
    top: "0",
    left: "0",
    width: "100vw",
    height: "100vh",
    zIndex: "9999",
    backgroundColor: "#000000",
};

// 元のスタイルを保持するためのマップ
const originalStyles = new Map();

// イベントリスナーを追加
document.addEventListener("click", function (event) {
    console.log(event);
    // Ctrlキーを押しているか確認
    if (event.ctrlKey && event.target.classList.contains("box")) {
        const box = event.target;

        // 元のスタイルを保存（初回のみ）
        if (!originalStyles.has(box)) {
            originalStyles.set(box, {
                position: box.style.position || "static",
                top: box.style.top || "",
                left: box.style.left || "",
                width: box.style.width || "",
                height: box.style.height || "",
                zIndex: box.style.zIndex || "",
            });
        }

        // 画面全体に表示するスタイルを適用
        Object.assign(box.style, fullScreenStyle);

        // もう一度クリックすると元に戻す
        box.addEventListener(
            "click",
            function restoreOriginal(event) {
                event.stopPropagation(); // イベントのバブリングを防止

                // 元のスタイルを復元
                const original = originalStyles.get(box);
                if (original) {
                    Object.assign(box.style, original);
                }

                // 復元後、このイベントリスナーを削除
                box.removeEventListener("click", restoreOriginal);

                // 元のスタイルを削除（必要なら）
                originalStyles.delete(box);
            },
            { once: true } // 一度だけ実行されるリスナー
        );
    }
});
