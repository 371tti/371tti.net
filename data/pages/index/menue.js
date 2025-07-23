document.addEventListener('DOMContentLoaded', () => {
    class ConsoleEmulator {
        constructor() {
            // メニューボタン作成
            const menuBtn = document.createElement('button');
            menuBtn.id = 'command-palette-btn';
            menuBtn.style.position = 'fixed';
            menuBtn.style.bottom = '20px';
            menuBtn.style.right = '20px';
            menuBtn.style.zIndex = '99999';
            menuBtn.style.width = '44px';
            menuBtn.style.height = '44px';
            menuBtn.style.padding = '0';
            menuBtn.style.borderRadius = '0';
            menuBtn.style.background = 'var(--color-bg-1)';
            menuBtn.style.color = 'var(--color-text-1)';
            menuBtn.style.border = '1px solid var(--color-bg-2)';
            menuBtn.style.boxShadow = '0 2px 8px rgba(0,0,0,0.15)';
            menuBtn.style.cursor = 'pointer';
            menuBtn.style.display = 'flex';
            menuBtn.style.alignItems = 'center';
            menuBtn.style.justifyContent = 'center';
            menuBtn.style.transition = 'all 0.2s ease';

            // 普通のハンバーガーメニューアイコン
            const paletteIcon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            paletteIcon.setAttribute('width', '20');
            paletteIcon.setAttribute('height', '20');
            paletteIcon.setAttribute('viewBox', '0 0 24 24');
            paletteIcon.setAttribute('id', 'hamburger-icon');
            paletteIcon.innerHTML = `
                <path fill="currentColor" d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
            `;
            const closeIcon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            closeIcon.setAttribute('width', '18');
            closeIcon.setAttribute('height', '18');
            closeIcon.setAttribute('viewBox', '0 0 24 24');
            closeIcon.setAttribute('id', 'close-icon');
            closeIcon.innerHTML = `
                <path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            `;
            menuBtn.appendChild(paletteIcon);

            // オーバーレイ作成
            const overlay = document.createElement('div');
            overlay.id = 'command-palette-overlay';
            overlay.style.position = 'fixed';
            overlay.style.top = '0';
            overlay.style.left = '0';
            overlay.style.width = '100vw';
            overlay.style.height = '100vh';
            overlay.style.background = 'rgba(0,0,0,0.4)';
            overlay.style.backdropFilter = 'blur(8px)';
            overlay.style.zIndex = '99998';
            overlay.style.display = 'none';
            overlay.style.animation = 'fadeIn 0.2s ease';

            // VS Code風のコマンドパレット
            const paletteDiv = document.createElement('div');
            paletteDiv.id = 'command-palette-div';
            paletteDiv.style.position = 'fixed';
            paletteDiv.style.left = '50%';
            paletteDiv.style.top = '25%';
            paletteDiv.style.transform = 'translateX(-50%)';
            paletteDiv.style.width = 'min(600px, 95vw)';
            paletteDiv.style.maxHeight = '60vh';
            paletteDiv.style.background = 'var(--color-bg-0)';
            paletteDiv.style.color = 'var(--color-text-1)';
            paletteDiv.style.border = '1px solid var(--color-bg-2)';
            paletteDiv.style.boxShadow = '0 8px 24px rgba(0,0,0,0.3)';
            paletteDiv.style.borderRadius = '0';
            paletteDiv.style.fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif';
            paletteDiv.style.overflow = 'hidden';
            paletteDiv.style.zIndex = '100000';
            paletteDiv.style.animation = 'slideIn 0.2s ease';

            // 検索入力部
            const inputContainer = document.createElement('div');
            inputContainer.style.padding = '12px';
            inputContainer.style.borderBottom = '1px solid var(--color-bg-2)';
            inputContainer.style.display = 'flex';
            inputContainer.style.alignItems = 'center';
            inputContainer.style.gap = '8px';

            // 検索アイコン
            const searchIcon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            searchIcon.setAttribute('width', '18');
            searchIcon.setAttribute('height', '18');
            searchIcon.setAttribute('viewBox', '0 0 24 24');
            searchIcon.style.color = 'var(--color-text-2)';
            searchIcon.innerHTML = `
                <path fill="currentColor" d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            `;

            const inputField = document.createElement('input');
            inputField.type = 'text';
            inputField.id = 'palette-input';
            inputField.placeholder = 'Type a command...';
            inputField.style.flex = '1';
            inputField.style.background = 'transparent';
            inputField.style.border = 'none';
            inputField.style.outline = 'none';
            inputField.style.color = 'var(--color-text-1)';
            inputField.style.fontSize = '16px';
            inputField.style.fontFamily = 'inherit';
            inputField.style.margin = '0';
            inputField.autocomplete = 'off';
            inputField.spellcheck = false;

            inputContainer.appendChild(searchIcon);
            inputContainer.appendChild(inputField);

            // 結果リスト
            const resultsDiv = document.createElement('div');
            resultsDiv.id = 'palette-results';
            resultsDiv.style.maxHeight = '400px';
            resultsDiv.style.overflowY = 'auto';
            resultsDiv.style.padding = '8px 0';

            paletteDiv.appendChild(inputContainer);
            paletteDiv.appendChild(resultsDiv);

            // コマンドツリー（シンプル化）
            this.commandTree = [
                { cmd: "Go: Home", desc: "Navigate to home page", action: () => location.href = "/" },
                { cmd: "Go: Login", desc: "Navigate to login page", action: () => location.href = "/login" },
                { cmd: "Go: Terms", desc: "Navigate to terms page", action: () => location.href = "/terms" },
                { cmd: "Go: License", desc: "Navigate to license page", action: () => location.href = "/license" },
                { cmd: "Go: Tools", desc: "Navigate to tools page", action: () => location.href = "/tools" },
                { cmd: "Browser: Back", desc: "Go back in browser history", action: () => history.back() },
                { cmd: "Browser: Forward", desc: "Go forward in browser history", action: () => history.forward() },
                { cmd: "Browser: Reload", desc: "Reload current page", action: () => location.reload() },
            ];

            // 初期化
            this.selectedIndex = -1;
            this.filteredCommands = [];
            this.menuBtn = menuBtn;
            this.overlay = overlay;
            this.paletteIcon = paletteIcon;
            this.closeIcon = closeIcon;
            this.inputField = inputField;
            this.resultsDiv = resultsDiv;

            // イベント設定
            this.setupEvents(menuBtn, overlay, paletteIcon, closeIcon, inputField, resultsDiv);

            // DOM追加
            document.body.appendChild(menuBtn);
            document.body.appendChild(overlay);
            overlay.appendChild(paletteDiv);

            // CSS アニメーション追加
            this.addAnimations();
        }

        addAnimations() {
            const style = document.createElement('style');
            style.textContent = `
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                @keyframes slideIn {
                    from { 
                        opacity: 0;
                        transform: translateX(-50%) translateY(-20px);
                    }
                    to { 
                        opacity: 1;
                        transform: translateX(-50%) translateY(0);
                    }
                }
                .palette-item {
                    padding: 8px 12px;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    transition: background-color 0.1s ease;
                    border-radius: 0;
                    margin: 0 4px;
                }
                .palette-item:hover, .palette-item.selected {
                    background-color: var(--color-bg-2);
                }
                .palette-item-title {
                    font-weight: 500;
                    color: var(--color-text-1);
                    font-size: 14px;
                }
                .palette-item-desc {
                    color: var(--color-text-2);
                    font-size: 12px;
                    margin-top: 1px;
                }
                .palette-item-icon {
                    width: 14px;
                    height: 14px;
                    opacity: 0.7;
                }
            `;
            document.head.appendChild(style);
        }

        setupEvents(menuBtn, overlay, paletteIcon, closeIcon, inputField, resultsDiv) {
            // メニューボタン
            menuBtn.onclick = () => this.togglePalette(overlay, menuBtn, paletteIcon, closeIcon, inputField);

            // キーボードショートカット (Ctrl+Shift+P または /)
            document.addEventListener('keydown', e => {
                if (((e.ctrlKey && e.shiftKey && e.key === 'P') || 
                     (e.key === '/' && !e.ctrlKey && !e.altKey && !e.metaKey)) &&
                    overlay.style.display !== 'block' &&
                    !(document.activeElement.tagName.match(/INPUT|TEXTAREA/) || document.activeElement.isContentEditable)) {
                    e.preventDefault();
                    this.showPalette(overlay, menuBtn, closeIcon, inputField);
                } else if (e.key === 'Escape' && overlay.style.display === 'block') {
                    this.hidePalette(overlay, menuBtn, paletteIcon);
                }
            });

            // 入力フィールドのイベント
            inputField.addEventListener('input', e => this.filterCommands(e.target.value, resultsDiv));
            inputField.addEventListener('keydown', e => this.handleKeyNavigation(e, resultsDiv, overlay, menuBtn, paletteIcon));

            // オーバーレイクリックで閉じる
            overlay.addEventListener('click', e => {
                if (e.target === overlay) {
                    this.hidePalette(overlay, menuBtn, paletteIcon);
                }
            });
        }

        togglePalette(overlay, menuBtn, paletteIcon, closeIcon, inputField) {
            if (overlay.style.display === 'block') {
                this.hidePalette();
            } else {
                this.showPalette();
            }
        }

        showPalette() {
            this.overlay.style.display = 'block';
            this.menuBtn.innerHTML = '';
            this.menuBtn.appendChild(this.closeIcon);
            setTimeout(() => {
                this.inputField.focus();
                this.filterCommands('', this.resultsDiv);
            }, 100);
        }

        hidePalette() {
            this.overlay.style.display = 'none';
            this.menuBtn.innerHTML = '';
            this.menuBtn.appendChild(this.paletteIcon);
            this.inputField.value = '';
            this.selectedIndex = -1;
        }

        filterCommands(query, resultsDiv) {
            this.filteredCommands = this.commandTree.filter(cmd => 
                cmd.cmd.toLowerCase().includes(query.toLowerCase()) ||
                cmd.desc.toLowerCase().includes(query.toLowerCase())
            );
            this.selectedIndex = this.filteredCommands.length > 0 ? 0 : -1;
            this.renderResults(resultsDiv);
        }

        renderResults(resultsDiv) {
            if (this.filteredCommands.length === 0) {
                resultsDiv.innerHTML = '<div style="padding: 16px; text-align: center; color: var(--color-text-2);">No commands found</div>';
                return;
            }

            resultsDiv.innerHTML = this.filteredCommands.map((cmd, idx) => `
                <div class="palette-item ${idx === this.selectedIndex ? 'selected' : ''}" data-idx="${idx}">
                    <svg class="palette-item-icon" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                    </svg>
                    <div>
                        <div class="palette-item-title">${cmd.cmd}</div>
                        <div class="palette-item-desc">${cmd.desc}</div>
                    </div>
                </div>
            `).join('');

            // クリックイベント
            resultsDiv.querySelectorAll('.palette-item').forEach(item => {
                item.addEventListener('click', () => {
                    const idx = parseInt(item.dataset.idx);
                    this.executeCommand(idx);
                });
            });
        }

        handleKeyNavigation(e, resultsDiv, overlay, menuBtn, paletteIcon) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                this.selectedIndex = Math.min(this.selectedIndex + 1, this.filteredCommands.length - 1);
                this.renderResults(this.resultsDiv);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
                this.renderResults(this.resultsDiv);
            } else if (e.key === 'Enter') {
                e.preventDefault();
                if (this.selectedIndex >= 0) {
                    this.executeCommand(this.selectedIndex);
                }
            } else if (e.key === 'Escape') {
                this.hidePalette();
            }
        }

        executeCommand(index) {
            if (this.filteredCommands[index] && this.filteredCommands[index].action) {
                this.filteredCommands[index].action();
                this.hidePalette();
            }
        }
    }

    new ConsoleEmulator();
});