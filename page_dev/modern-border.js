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
        
        // マウス移動イベント
        document.addEventListener("mousemove", (e) => {
            const rect = element.getBoundingClientRect();
            const margin = 200; // 反応範囲
            const extendedRect = {
                left: rect.left - margin,
                top: rect.top - margin,
                right: rect.right + margin,
                bottom: rect.bottom + margin,
            };

            const isMouseNear = e.clientX >= extendedRect.left &&
                e.clientX <= extendedRect.right &&
                e.clientY >= extendedRect.top &&
                e.clientY <= extendedRect.bottom;

            if (isMouseNear) {
                const x = e.clientX - rect.left;
                const y = e.clientY - rect.top;

                // マスクを表示
                borderMask.style.opacity = "1";
                contentMask.style.opacity = "1";

                // CSSカスタムプロパティを更新
                borderMask.style.setProperty("--mouse-x", `${x}px`);
                borderMask.style.setProperty("--mouse-y", `${y}px`);
                contentMask.style.setProperty("--mouse-x", `${x}px`);
                contentMask.style.setProperty("--mouse-y", `${y}px`);
            } else {
                // マスクを非表示
                borderMask.style.opacity = "0";
                contentMask.style.opacity = "0";
            }
        });
    });
});
