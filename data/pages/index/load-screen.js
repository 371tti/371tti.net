(() => {
    const startTime = Date.now();
    let resourceCount = 0;

    // DOM要素の安全な追加を待つ関数
    const waitForDOM = (callback) => {
        if (document.head && document.body) {
            callback();
        } else {
            setTimeout(() => waitForDOM(callback), 10);
        }
    };

    // デバッグ画面のHTML要素を作成
    const debugScreen = document.createElement('div');
    debugScreen.id = 'debug-screen';
    
    // CSS-in-JSで右下表示のデバッグパネル
    debugScreen.style.cssText = `
        position: fixed;
        bottom: 20px;
        right: 20px;
        background: rgba(0, 0, 0, 0.6);
        border-left: 1px solid #232323;
        padding: 20px 12px 12px 12px;
        z-index: 9999;
        font-family: Consolas, Monaco, 'Andale Mono', monospace;
        font-size: 11px;
        color: #ffffff;
        min-width: 220px;
        max-width: 300px;
        transition: opacity 200ms ease-out;
    `;

    // ×ボタンを作成
    const closeButton = document.createElement('div');
    closeButton.textContent = '×';
    closeButton.style.cssText = `
        position: absolute;
        top: 6px;
        right: 8px;
        color: #aaaaaa;
        cursor: pointer;
        font-size: 14px;
        line-height: 1;
        user-select: none;
        transition: color 0.2s;
        width: 16px;
        height: 16px;
        text-align: center;
    `;
    
    // ×ボタンのホバー効果
    closeButton.addEventListener('mouseenter', () => {
        closeButton.style.color = '#ffffff';
    });
    closeButton.addEventListener('mouseleave', () => {
        closeButton.style.color = '#aaaaaa';
    });
    
    // ×ボタンのクリックイベント
    closeButton.addEventListener('click', () => {
        debugScreen.style.opacity = '0';
        setTimeout(() => {
            if (debugScreen.parentNode) {
                debugScreen.parentNode.removeChild(debugScreen);
            }
        }, 200);
    });

    // 各デバッグ情報の要素を作成
    const domState = document.createElement('div');
    domState.style.cssText = `color: #e8af7f; margin-bottom: 4px;`;
    
    const loadTime = document.createElement('div');
    loadTime.style.cssText = `color: #aaaaaa; margin-bottom: 4px;`;
    
    const resourceInfo = document.createElement('div');
    resourceInfo.style.cssText = `color: #815a44; margin-bottom: 4px;`;
    
    const performanceInfo = document.createElement('div');
    performanceInfo.style.cssText = `color: #aaaaaa; margin-bottom: 4px;`;
    
    const navigationInfo = document.createElement('div');
    navigationInfo.style.cssText = `color: #e8af7f; margin-bottom: 4px;`;

    const scriptInfo = document.createElement('div');
    scriptInfo.style.cssText = `color: #aaaaaa; font-size: 10px;`;

    // DOM要素が利用可能になったら実行
    waitForDOM(() => {
        // 要素を組み立て
        debugScreen.appendChild(closeButton);
        debugScreen.appendChild(domState);
        debugScreen.appendChild(loadTime);
        debugScreen.appendChild(resourceInfo);
        debugScreen.appendChild(performanceInfo);
        debugScreen.appendChild(navigationInfo);
        debugScreen.appendChild(scriptInfo);
        document.body.appendChild(debugScreen);
    });

    // 詳細なデバッグ情報の更新
    const updateDebugInfo = () => {
        const elapsed = Date.now() - startTime;
        const state = document.readyState;
        
        // DOM状態
        domState.textContent = `DOM: ${state}`;
        
        // 経過時間
        loadTime.textContent = `Time: ${elapsed}ms`;
        
        // リソース情報
        if (performance && performance.getEntriesByType) {
            const resources = performance.getEntriesByType('resource');
            resourceInfo.textContent = `Resources: ${resources.length} loaded`;
        } else {
            resourceInfo.textContent = `Resources: monitoring...`;
        }
        
        // パフォーマンス情報
        if (performance && performance.timing) {
            const timing = performance.timing;
            const dnsDuration = timing.domainLookupEnd - timing.domainLookupStart;
            const connectDuration = timing.connectEnd - timing.connectStart;
            performanceInfo.textContent = `DNS: ${dnsDuration}ms, Connect: ${connectDuration}ms`;
        } else {
            performanceInfo.textContent = `Performance: measuring...`;
        }
        
        // ナビゲーション情報
        if (performance && performance.navigation) {
            const navType = performance.navigation.type;
            const navTypes = ['navigate', 'reload', 'back_forward', 'prerender'];
            navigationInfo.textContent = `Nav: ${navTypes[navType] || 'unknown'}`;
        } else {
            navigationInfo.textContent = `Nav: detecting...`;
        }
        
        // スクリプト情報
        const scripts = document.querySelectorAll('script').length;
        const stylesheets = document.querySelectorAll('link[rel="stylesheet"]').length;
        scriptInfo.textContent = `Scripts: ${scripts}, CSS: ${stylesheets}`;
    };

    // 時間更新とデバッグ情報監視の処理
    const updateInterval = setInterval(updateDebugInfo, 50);

    // DOM準備完了時
    document.addEventListener('DOMContentLoaded', () => {
        domState.textContent = 'DOM: ready!';
    });

    // ページ読み込み完了時の処理
    window.addEventListener('load', () => {
        clearInterval(updateInterval);
        
        const finalTime = Date.now() - startTime;
        loadTime.textContent = `Final: ${finalTime}ms`;
        domState.textContent = 'DOM: complete!';
        
        setTimeout(() => {
            debugScreen.style.opacity = '0';
            setTimeout(() => {
                if (debugScreen.parentNode) {
                    debugScreen.parentNode.removeChild(debugScreen);
                }
            }, 200);
        }, 2000);
    });

    // ページが既に読み込まれている場合の処理
    if (document.readyState === 'complete') {
        window.dispatchEvent(new Event('load'));
    }
})();
