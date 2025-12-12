// prism-loader.js
(() => {
    const PRISM_VERSION = "1.29.0";
    const CDN = `https://cdn.jsdelivr.net/npm/prismjs@${PRISM_VERSION}`;

    /** 動的scriptロード */
    function loadScript(src) {
        return new Promise((resolve, reject) => {
            const s = document.createElement("script");
            s.src = src;
            s.onload = resolve;
            s.onerror = reject;
            document.head.appendChild(s);
        });
    }

    /** 動的CSSロード */
    function loadCss(href) {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = href;
        document.head.appendChild(link);
    }

    /** DOM 準備 */
    function ready(fn) {
        if (document.readyState === "loading")
            document.addEventListener("DOMContentLoaded", fn, { once: true });
        else fn();
    }

    async function main() {
        // ▼ Prism テーマ（好きなものに変更可能）
        loadCss(`${CDN}/themes/prism-okaidia.min.css`);

        // ▼ 行番号プラグインの CSS
        loadCss(`${CDN}/plugins/line-numbers/prism-line-numbers.min.css`);

        // ▼ Prism 本体ロード
        await loadScript(`${CDN}/prism.min.js`);

        // ▼ Autoloader プラグイン（言語自動ロード）
        await loadScript(`${CDN}/plugins/autoloader/prism-autoloader.min.js`);
        Prism.plugins.autoloader.languages_path = `${CDN}/components/`;

        // ▼ 行番号プラグイン JS
        await loadScript(`${CDN}/plugins/line-numbers/prism-line-numbers.min.js`);

        // ▼ コードブロックを行番号対応に変更
        document.querySelectorAll("pre code").forEach(code => {
            const pre = code.parentElement;
            if (!pre.classList.contains("line-numbers")) {
                pre.classList.add("line-numbers");
            }
        });

        // ▼ Prism ハイライト実行
        Prism.highlightAll();
    }

    ready(() => {
        main().catch(err => console.error("[Prism Loader] Failed:", err));
    });
})();
