(() => {
    // コンソールインターセプト機能付き通知システム
    let notificationId = 0;
    let notificationTimeouts = new Map(); // タイムアウトIDを管理

    // 通知コンテナの作成
    const notificationContainer = document.createElement('div');
    notificationContainer.id = 'notification-container';
    notificationContainer.style.cssText = `
        position: fixed;
        bottom: 80px;
        right: 20px;
        z-index: 9999;
        font-family: Consolas, Monaco, 'Andale Mono', monospace;
        pointer-events: none;
        max-width: 300px;
    `;

    // DOM要素の安全な追加を待つ関数
    const waitForDOM = (callback) => {
        if (document.body) {
            callback();
        } else {
            setTimeout(() => waitForDOM(callback), 10);
        }
    };

    // 通知を作成する関数
    const createNotification = (message, type = 'info') => {
        const notification = document.createElement('div');
        notification.id = `notification-${notificationId++}`;
        
        const colors = {
            info: { bg: 'var(--color-bg-0)', border: 'var(--color-accent-0)', text: 'var(--color-text-1)' },
            warn: { bg: 'var(--color-bg-0)', border: 'var(--color-accent-1)', text: 'var(--color-accent-1)' },
            error: { bg: 'var(--color-bg-0)', border: 'var(--color-accent-1)', text: 'var(--color-accent-1)' },
            success: { bg: 'var(--color-bg-0)', border: 'var(--color-accent-0)', text: 'var(--color-accent-1)' },
            debug: { bg: 'var(--color-bg-0)', border: 'var(--color-accent-0)', text: 'var(--color-accent-1)' }
        };

        const color = colors[type] || colors.info;

        notification.style.cssText = `
            background: ${color.bg};
            border-left: 1px solid ${color.border};
            color: ${color.text};
            padding: 20px 12px 12px 12px;
            margin-bottom: 8px;
            font-size: 11px;
            line-height: 1.4;
            pointer-events: auto;
            cursor: pointer;
            transition: opacity 0.3s cubic-bezier(0.12, 0.63, 0, 1), transform 0.3s cubic-bezier(0.12, 0.63, 0, 1);
            transform: translateX(100%);
            opacity: 0;
            word-wrap: break-word;
            max-width: 100%;
            position: relative;
            min-width: 220px;
        `;

        // ×ボタンを作成
        const closeButton = document.createElement('div');
        closeButton.textContent = '×';
        closeButton.style.cssText = `
            position: absolute;
            top: 6px;
            right: 8px;
            color: var(--color-text-0);
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
            closeButton.style.color = 'var(--color-text-1)';
        });
        closeButton.addEventListener('mouseleave', () => {
            closeButton.style.color = 'var(--color-text-0)';
        });

        const messageDiv = document.createElement('div');
        messageDiv.textContent = message;

        notification.appendChild(closeButton);
        notification.appendChild(messageDiv);

        // クリックで閉じる
        const closeNotification = () => {
            notification.style.opacity = '0';
            notification.style.transform = 'translateX(100%)';
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
            }, 300);
        };

        notification.addEventListener('click', closeNotification);
        closeButton.addEventListener('click', (e) => {
            e.stopPropagation();
            closeNotification();
        });

        return notification;
    };

    // 通知を表示する関数
    const showNotification = (message, type = 'info', duration = 5000) => {
        const notification = createNotification(message, type);
        notificationContainer.appendChild(notification);

        // アニメーション開始
        requestAnimationFrame(() => {
            notification.style.transform = 'translateX(0)';
            notification.style.opacity = '1';
        });

        // 自動削除タイムアウトを設定
        const timeoutId = setTimeout(() => {
            if (notification.parentNode) {
                notification.style.opacity = '0';
                notification.style.transform = 'translateX(100%)';
                setTimeout(() => {
                    if (notification.parentNode) {
                        notification.parentNode.removeChild(notification);
                    }
                    notificationTimeouts.delete(notification.id);
                }, 300);
            }
        }, duration);

        // タイムアウトIDを保存
        notificationTimeouts.set(notification.id, timeoutId);
    };

    // 全通知を閉じる関数
    const closeAllNotifications = () => {
        // 全てのタイムアウトをクリア
        notificationTimeouts.forEach(timeoutId => clearTimeout(timeoutId));
        notificationTimeouts.clear();

        // 全ての通知要素を削除
        const notifications = notificationContainer.querySelectorAll('[id^="notification-"]');
        notifications.forEach(notification => {
            notification.style.opacity = '0';
            notification.style.transform = 'translateX(100%)';
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
            }, 300);
        });
    };

    // 表示時間を延長する関数
    const extendNotificationTime = () => {
        const notifications = notificationContainer.querySelectorAll('[id^="notification-"]');
        notifications.forEach(notification => {
            const timeoutId = notificationTimeouts.get(notification.id);
            if (timeoutId) {
                clearTimeout(timeoutId);
                // 5秒延長
                const newTimeoutId = setTimeout(() => {
                    if (notification.parentNode) {
                        notification.style.opacity = '0';
                        notification.style.transform = 'translateX(100%)';
                        setTimeout(() => {
                            if (notification.parentNode) {
                                notification.parentNode.removeChild(notification);
                            }
                            notificationTimeouts.delete(notification.id);
                        }, 300);
                    }
                }, 5000);
                notificationTimeouts.set(notification.id, newTimeoutId);
            }
        });
    };

    // オリジナルのコンソールメソッドを保存
    const originalConsole = {
        log: console.log,
        warn: console.warn,
        error: console.error,
        info: console.info,
        debug: console.debug
    };

    // 安全な文字列化
    const safeStringify = (value) => {
        if (typeof value === 'string') return value;
        if (value === null || value === undefined) return String(value);
        if (typeof value === 'bigint') return value.toString();
        if (typeof value === 'function') return `[Function ${value.name || 'anonymous'}]`;
        if (value instanceof Error) return value.stack || value.message || String(value);
        try {
            const seen = new WeakSet();
            return JSON.stringify(value, (_key, val) => {
                if (typeof val === 'bigint') return val.toString();
                if (typeof val === 'function') return `[Function ${val.name || 'anonymous'}]`;
                if (val instanceof Error) return val.stack || val.message || String(val);
                if (typeof val === 'object' && val !== null) {
                    if (seen.has(val)) return '[Circular]';
                    seen.add(val);
                }
                return val;
            });
        } catch (_) {
            try {
                return String(value);
            } catch (__) {
                return '[Unserializable]';
            }
        }
    };

    // コンソールメソッドをインターセプト
    console.log = (...args) => {
        originalConsole.log(...args);
        const message = args.map(arg => safeStringify(arg)).join(' ');
        if (message.startsWith('[Debug]')) {
            showNotification(message, 'debug', 3000);
        } else if (message.startsWith('[Error]')) {
            showNotification(message, 'error', 7000);
        } else {
            showNotification(message, 'info', 4000);
        }
    };

    console.warn = (...args) => {
        originalConsole.warn(...args);
        const message = args.map(arg => safeStringify(arg)).join(' ');
        showNotification(message, 'warn', 6000);
    };

    console.error = (...args) => {
        originalConsole.error(...args);
        const message = args.map(arg => safeStringify(arg)).join(' ');
        showNotification(message, 'error', 8000);
    };

    console.info = (...args) => {
        originalConsole.info(...args);
        const message = args.map(arg => safeStringify(arg)).join(' ');
        showNotification(message, 'info', 4000);
    };

    console.debug = (...args) => {
        originalConsole.debug(...args);
        const message = args.map(arg => safeStringify(arg)).join(' ');
        showNotification(message, 'debug', 3000);
    };

    // DOM要素が利用可能になったら実行
    waitForDOM(() => {
        document.body.appendChild(notificationContainer);
    });

    // ページが既に読み込まれている場合の処理
    if (document.readyState === 'complete') {
        waitForDOM(() => {
            document.body.appendChild(notificationContainer);
        });
    }
})();
