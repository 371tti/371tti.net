(() => {
    const STYLE_ID = "code-tool-style-v3";
    const APPLIED = "data-code-tool-applied";
    const TOOLBAR_CLASS = "codeblock-toolbar";

    function injectStyle() {
        if (document.getElementById(STYLE_ID)) return;
        const style = document.createElement("style");
        style.id = STYLE_ID;
        style.textContent = `
:root{
    --codeblock-margin-y: 4mm;
    --toolbar-top-gap: 5mm;
    --toolbar-inset: 2mm;
    --toolbar-icon: 22px;
    --toolbar-gap-x: 8px;
}

.codeblock{
    position: relative;
    margin: var(--codeblock-margin-y) 0;
    max-width: 100%;
}

/* ここ重要：preを伸ばすような padding/top は一切付けない */

/* ツールバー（absolute/fixedはJSで切替） */
.${TOOLBAR_CLASS}{
    position: absolute;
    top: var(--toolbar-inset);
    right: var(--toolbar-inset);
    z-index: 9999;
    pointer-events: none;
}

.${TOOLBAR_CLASS} .codeblock-actions{
    pointer-events: auto;
    display: inline-flex;
    align-items: center;
    gap: var(--toolbar-gap-x);
}

.${TOOLBAR_CLASS} button{
    background: transparent;
    border: 0;
    padding: 0;
    margin: 0;
    cursor: pointer;
    opacity: 0.75;
}
.${TOOLBAR_CLASS} button:hover{ opacity: 1; }

.${TOOLBAR_CLASS} img{
    display: block;
    width: var(--toolbar-icon);
    height: var(--toolbar-icon);
    filter: brightness(0) invert(1);
}
`;
        document.head.appendChild(style);
    }

    function toPx(len, fallbackPx = 0) {
        const s = String(len ?? "").trim();
        if (!s) return fallbackPx;
        if (s.endsWith("px")) return parseFloat(s) || fallbackPx;

        const el = document.createElement("div");
        el.style.position = "absolute";
        el.style.visibility = "hidden";
        el.style.width = s;
        document.body.appendChild(el);
        const px = el.getBoundingClientRect().width;
        el.remove();
        return px || fallbackPx;
    }

    function cssVarPx(el, name, fallbackPx) {
        const v = getComputedStyle(el).getPropertyValue(name).trim();
        return toPx(v, fallbackPx);
    }

    function ensureCodeblock(pre) {
        const wrap = pre.closest(".codeblock");
        if (wrap) return wrap;

        const container = document.createElement("div");
        container.className = "codeblock";
        pre.parentNode.insertBefore(container, pre);
        container.appendChild(pre);
        return container;
    }

    function makeIconButton(iconPath, ariaLabel, className, pressed) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = className;
        btn.setAttribute("aria-label", ariaLabel);
        btn.title = ariaLabel;
        if (pressed !== undefined) {
            btn.setAttribute("aria-pressed", String(pressed));
        }

        const img = document.createElement("img");
        img.src = iconPath;
        img.alt = "";
        img.setAttribute("aria-hidden", "true");
        btn.appendChild(img);
        return btn;
    }

    async function copyText(text) {
        if (navigator.clipboard?.writeText) {
            await navigator.clipboard.writeText(text);
            return true;
        }
        const ta = document.createElement("textarea");
        ta.value = text;
        ta.style.position = "fixed";
        ta.style.left = "-9999px";
        document.body.appendChild(ta);
        ta.select();
        try {
            return document.execCommand("copy");
        } finally {
            ta.remove();
        }
    }

    function ensureToolbar(container, pre) {
        let tb = container.querySelector(`:scope > .${TOOLBAR_CLASS}`);
        if (tb) return tb;

        tb = document.createElement("div");
        tb.className = TOOLBAR_CLASS;

        const actions = document.createElement("div");
        actions.className = "codeblock-actions";
        tb.appendChild(actions);

        // Wrap
        const wrapBtn = makeIconButton(
            "/raw/static/icons/arrow-back-up.svg",
            "Wrap",
            "codeblock-wrap",
            false,
        );
        wrapBtn.addEventListener("click", () => {
            const isWrapped = container.classList.toggle("codeblock-wrap-on");
            wrapBtn.setAttribute("aria-pressed", String(isWrapped));
            pre.style.whiteSpace = isWrapped ? "pre-wrap" : "pre";
            pre.style.wordBreak = isWrapped ? "break-word" : "normal";
        });
        actions.appendChild(wrapBtn);

        // Copy
        const copyBtn = makeIconButton(
            "/raw/static/icons/copy.svg",
            "Copy",
            "codeblock-copy",
        );
        copyBtn.addEventListener("click", async () => {
            const code = pre.querySelector("code") || pre;
            const ok = await copyText(code.textContent || "");
            if (!ok) return;

            const img = copyBtn.querySelector("img");
            copyBtn.setAttribute("aria-label", "Copied");
            copyBtn.title = "Copied";
            if (img) img.src = "/raw/static/icons/copy-check.svg";
            setTimeout(() => {
                copyBtn.setAttribute("aria-label", "Copy");
                copyBtn.title = "Copy";
                if (img) img.src = "/raw/static/icons/copy.svg";
            }, 1200);
        });
        actions.appendChild(copyBtn);

        container.insertBefore(tb, container.firstChild);
        return tb;
    }

    const items = new Set();
    let raf = 0;

    function scheduleUpdate() {
        if (raf) return;
        raf = requestAnimationFrame(updateAll);
    }

    function updateAll() {
        raf = 0;
        const vw = window.innerWidth;
        const vh = window.innerHeight;

        for (const it of items) {
            const { container, toolbar } = it;
            if (!container.isConnected) {
                items.delete(it);
                continue;
            }

            const rect = container.getBoundingClientRect();
            const tbRect = toolbar.getBoundingClientRect();

            const inset = cssVarPx(container, "--toolbar-inset", 8);
            const fixedTop = cssVarPx(container, "--toolbar-top-gap", 16);

            const tbW = tbRect.width || 0;
            const tbH = tbRect.height || 0;

            if (rect.bottom <= 0 || rect.top >= vh) {
                toolbar.style.position = "absolute";
                toolbar.style.top = `${inset}px`;
                toolbar.style.right = `${inset}px`;
                toolbar.style.left = "auto";
                continue;
            }

            const bottomLimit = rect.bottom - inset - tbH;

            if (bottomLimit <= fixedTop) {
                toolbar.style.position = "absolute";
                const topInside = Math.max(
                    inset,
                    container.clientHeight - inset - tbH,
                );
                toolbar.style.top = `${topInside}px`;
                toolbar.style.right = `${inset}px`;
                toolbar.style.left = "auto";
            } else if (rect.top <= fixedTop) {
                toolbar.style.position = "fixed";
                toolbar.style.top = `${fixedTop}px`;

                let left = rect.right - inset - tbW;
                left = Math.max(inset, Math.min(left, vw - inset - tbW));
                toolbar.style.left = `${Math.round(left)}px`;
                toolbar.style.right = "auto";
            } else {
                toolbar.style.position = "absolute";
                toolbar.style.top = `${inset}px`;
                toolbar.style.right = `${inset}px`;
                toolbar.style.left = "auto";
            }
        }
    }

    function enhancePre(pre) {
        if (!(pre instanceof HTMLElement)) return;
        if (pre.tagName.toLowerCase() !== "pre") return;
        if (pre.getAttribute(APPLIED) === "true") return;

        pre.setAttribute(APPLIED, "true");

        const container = ensureCodeblock(pre);
        container.setAttribute(APPLIED, "true");

        container.querySelector(":scope > .codeblock-head")?.remove();

        const toolbar = ensureToolbar(container, pre);
        items.add({ container, toolbar });

        scheduleUpdate();
    }

    function enhanceAll(root = document) {
        root.querySelectorAll("pre").forEach(enhancePre);
    }

    function boot() {
        injectStyle();
        enhanceAll();

        window.addEventListener("scroll", scheduleUpdate, { passive: true });
        window.addEventListener("resize", scheduleUpdate);
        document.addEventListener("scroll", scheduleUpdate, {
            passive: true,
            capture: true,
        });

        const mo = new MutationObserver((mutations) => {
            for (const m of mutations) {
                for (const node of m.addedNodes) {
                    if (!(node instanceof HTMLElement)) continue;
                    if (node.tagName?.toLowerCase() === "pre") enhancePre(node);
                    node.querySelectorAll?.("pre").forEach(enhancePre);
                }
            }
            scheduleUpdate();
        });
        mo.observe(document.documentElement, {
            childList: true,
            subtree: true,
        });

        scheduleUpdate();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", boot);
    } else {
        boot();
    }
})();
