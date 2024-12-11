document.addEventListener("DOMContentLoaded", () => {
    // 各要素にモダンボーダーを適用
    document.querySelectorAll(".modern-border").forEach((element) => {
        const borderMask = document.createElement("div");
        borderMask.classList.add("border-mask");
        const contentMask = document.createElement("div");
        contentMask.classList.add("content-mask");
        element.appendChild(borderMask);
        element.appendChild(contentMask);

        borderMask.style.opacity = "0";
        contentMask.style.opacity = "0";
        borderMask.style.setProperty("--mouse-x", `-9999px`);
        borderMask.style.setProperty("--mouse-y", `-9999px`);
        contentMask.style.setProperty("--mouse-x", `-9999px`);
        contentMask.style.setProperty("--mouse-y", `-9999px`);

        // マウスやタッチ、スクロール、リサイズ時の共通処理
        const updateMaskVisibility = (x, y) => {
            const rect = element.getBoundingClientRect();
            const margin = 200; // 反応範囲
            const extendedRect = {
                left: rect.left - margin,
                top: rect.top - margin,
                right: rect.right + margin,
                bottom: rect.bottom + margin,
            };

            const isNear = x >= extendedRect.left &&
                x <= extendedRect.right &&
                y >= extendedRect.top &&
                y <= extendedRect.bottom;

            if (isNear) {
                const localX = x - rect.left;
                const localY = y - rect.top;

                // マスクを表示
                borderMask.style.opacity = "1";
                contentMask.style.opacity = "1";

                // CSSカスタムプロパティを更新
                borderMask.style.setProperty("--mouse-x", `${localX}px`);
                borderMask.style.setProperty("--mouse-y", `${localY}px`);
                contentMask.style.setProperty("--mouse-x", `${localX}px`);
                contentMask.style.setProperty("--mouse-y", `${localY}px`);
            } else {
                // マスクを非表示
                borderMask.style.opacity = "0";
                contentMask.style.opacity = "0";
            }
        };

        // マウスやタッチ移動時
        const handleMouseMove = (e) => {
            updateMaskVisibility(e.clientX, e.clientY);
        };

        const handleTouchMove = (e) => {
            const touch = e.touches[0]; // 最初のタッチポイント
            if (touch) {
                updateMaskVisibility(touch.clientX, touch.clientY);
            }
        };

        // マスクを非表示にする処理
        const hideMask = () => {
            borderMask.style.opacity = "0";
            contentMask.style.opacity = "0";
        };

        // イベントリスナー登録
        document.addEventListener("mousemove", handleMouseMove);
        document.addEventListener("touchmove", handleTouchMove);
        document.addEventListener("mouseleave", hideMask);
        document.addEventListener("touchend", hideMask);
        document.addEventListener("touchcancel", hideMask);
    });
});
