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

            // VS Code風のコマンドパレット
            const paletteDiv = document.createElement('div');
            paletteDiv.id = 'command-palette-div';
            paletteDiv.style.position = 'fixed';
            paletteDiv.style.left = '50%';
            paletteDiv.style.top = '10%';
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

            // コマンドツリー（テーマコマンド追加）
            this.commandTree = [
                { cmd: "Go: Home", desc: "Navigate to home page", action: () => location.href = "/" },
                { cmd: "Go: Login", desc: "Navigate to login page", action: () => location.href = "/login" },
                { cmd: "Go: Terms", desc: "Navigate to terms page", action: () => location.href = "/terms" },
                { cmd: "Go: License", desc: "Navigate to license page", action: () => location.href = "/license" },
                { cmd: "Go: Tools", desc: "Navigate to tools page", action: () => location.href = "/tools" },
                { cmd: "Go: Release", desc: "Navigate to release page", action: () => location.href = "/release" },
                { cmd: "Browser: Back", desc: "Go back in browser history", action: () => history.back() },
                { cmd: "Browser: Forward", desc: "Go forward in browser history", action: () => history.forward() },
                { cmd: "Browser: Reload", desc: "Reload current page", action: () => location.reload() },
                { cmd: "Theme", desc: "Change theme (white/dark/coffee/ocean/forest/sunset/kawaii/mono-dark/mono-white/paper)", action: (args) => this.changeTheme(args) },
                { cmd: "Theme white", desc: "Change to white theme", action: () => this.changeTheme(['white']) },
                { cmd: "Theme dark", desc: "Change to dark theme", action: () => this.changeTheme(['dark']) },
                { cmd: "Theme coffee", desc: "Change to coffee theme", action: () => this.changeTheme(['coffee']) },
                { cmd: "Theme ocean", desc: "Change to ocean theme", action: () => this.changeTheme(['ocean']) },
                { cmd: "Theme forest", desc: "Change to forest theme", action: () => this.changeTheme(['forest']) },
                { cmd: "Theme sunset", desc: "Change to sunset theme", action: () => this.changeTheme(['sunset']) },
                { cmd: "Theme kawaii", desc: "Change to kawaii theme", action: () => this.changeTheme(['kawaii']) },
                { cmd: "Theme mono-dark", desc: "Change to mono-dark theme", action: () => this.changeTheme(['mono-dark']) },
                { cmd: "Theme mono-white", desc: "Change to mono-white theme", action: () => this.changeTheme(['mono-white']) },
                { cmd: "Theme paper", desc: "Change to paper theme", action: () => this.changeTheme(['paper']) },
            ];

            // テーマ設定
            this.themes = {
                white: {
                    '--color-bg-0': '#ffffff',
                    '--color-bg-1': '#e9e9e9',
                    '--color-bg-2': '#875e46',
                    '--color-text-0': '#3c3c3c',
                    '--color-text-1': '#000000',
                    '--color-accent-0': '#ffdcc8',
                    '--color-accent-1': '#e4873a'
                },
                dark: {
                    '--color-bg-0': '#000000',
                    '--color-bg-1': '#232323',
                    '--color-bg-2': '#875e46',
                    '--color-text-0': '#d1d1d1',
                    '--color-text-1': '#ffffff',
                    '--color-accent-0': '#815a44',
                    '--color-accent-1': '#e8af7f'
                },
                coffee: {
                    '--color-bg-0': '#1a1410',
                    '--color-bg-1': '#251e18',
                    '--color-bg-2': '#302820',
                    '--color-text-0': '#b3b3b3',
                    '--color-text-1': '#e6e6e6',
                    '--color-accent-0': '#5d4a37',
                    '--color-accent-1': '#9b7e64ff'
                },
                ocean: {
                    '--color-bg-0': '#0f1419',
                    '--color-bg-1': '#161d24',
                    '--color-bg-2': '#1d262f',
                    '--color-text-0': '#b3b3b3',
                    '--color-text-1': '#e6e6e6',
                    '--color-accent-0': '#5b6e81ff',
                    '--color-accent-1': '#4a5a6a'
                },
                forest: {
                    '--color-bg-0': '#0f1510',
                    '--color-bg-1': '#151d18',
                    '--color-bg-2': '#1b2520',
                    '--color-text-0': '#b3b3b3',
                    '--color-text-1': '#e6e6e6',
                    '--color-accent-0': '#3a4a3a',
                    '--color-accent-1': '#6f8e6fff'
                },
                sunset: {
                    '--color-bg-0': '#1a1218',
                    '--color-bg-1': '#251820',
                    '--color-bg-2': '#301e28',
                    '--color-text-0': '#b3b3b3',
                    '--color-text-1': '#e6e6e6',
                    '--color-accent-0': '#5a3a5a',
                    '--color-accent-1': '#8c648cff'
                },
                kawaii: {
                    '--color-bg-0': '#fff0f5',
                    '--color-bg-1': '#ffe4e1',
                    '--color-bg-2': '#ffb6c1',
                    '--color-text-0': '#c95995ff',
                    '--color-text-1': '#ff7cc2ff',
                    '--color-accent-0': '#ff69b4',
                    '--color-accent-1': '#ff7cc2ff'
                },
                'mono-dark': {
                    '--color-bg-0': '#0a0a0a',
                    '--color-bg-1': '#1a1a1a',
                    '--color-bg-2': '#333333',
                    '--color-text-0': '#999999',
                    '--color-text-1': '#cccccc',
                    '--color-accent-0': '#666666',
                    '--color-accent-1': '#aaaaaa'
                },
                'mono-white': {
                    '--color-bg-0': '#fafafa',
                    '--color-bg-1': '#f0f0f0',
                    '--color-bg-2': '#cccccc',
                    '--color-text-0': '#666666',
                    '--color-text-1': '#333333',
                    '--color-accent-0': '#999999',
                    '--color-accent-1': '#555555'
                },
                paper: {
                    '--color-bg-0': '#f7f5f3',
                    '--color-bg-1': '#e8e5e0',
                    '--color-bg-2': '#d4c5a9',
                    '--color-text-0': '#5d5347',
                    '--color-text-1': '#2d2520',
                    '--color-accent-0': '#a67c52',
                    '--color-accent-1': '#8b5a3c'
                }
            };

            // 初期化
            this.selectedIndex = -1;
            this.filteredCommands = [];
            this.isAnimating = false; // アニメーション状態フラグを追加
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
            
            // 保存されたテーマを読み込み
            this.loadSavedTheme();
        }

        addAnimations() {
            const style = document.createElement('style');
            style.textContent = `
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                @keyframes fadeOut {
                    from { opacity: 1; }
                    to { opacity: 0; }
                }
                @keyframes slideIn {
                    from { 
                        opacity: 0;
                        transform: translateX(-50%) translateY(-30px);
                    }
                    to { 
                        opacity: 1;
                        transform: translateX(-50%) translateY(0);
                    }
                }
                @keyframes slideOut {
                    from { 
                        opacity: 1;
                        transform: translateX(-50%) translateY(0);
                    }
                    to { 
                        opacity: 0;
                        transform: translateX(-50%) translateY(-30px);
                    }
                }
                #command-palette-overlay.show {
                    animation: fadeIn 0.2s ease forwards;
                }
                #command-palette-overlay.hide {
                    animation: fadeOut 0.2s ease forwards;
                }
                #command-palette-div.show {
                    animation: slideIn 0.2s ease forwards;
                }
                #command-palette-div.hide {
                    animation: slideOut 0.2s ease forwards;
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
            menuBtn.onclick = () => this.togglePalette();

            // キーボードショートカット (Ctrl+Shift+P または /)
            document.addEventListener('keydown', e => {
                if (((e.ctrlKey && e.shiftKey && e.key === 'P') || 
                     (e.key === '/' && !e.ctrlKey && !e.altKey && !e.metaKey)) &&
                    !this.isVisible() &&
                    !(document.activeElement.tagName.match(/INPUT|TEXTAREA/) || document.activeElement.isContentEditable)) {
                    e.preventDefault();
                    this.showPalette();
                } else if (e.key === 'Escape' && this.isVisible()) {
                    this.hidePalette();
                }
            });

            // 入力フィールドのイベント
            inputField.addEventListener('input', e => this.filterCommands(e.target.value, resultsDiv));
            inputField.addEventListener('keydown', e => this.handleKeyNavigation(e));

            // オーバーレイクリックで閉じる
            overlay.addEventListener('click', e => {
                if (e.target === overlay) {
                    this.hidePalette();
                }
            });
        }

        togglePalette() {
            if (this.isVisible()) {
                this.hidePalette();
            } else {
                this.showPalette();
            }
        }

        isVisible() {
            return this.overlay.style.display === 'block';
        }

        showPalette() {
            if (this.isAnimating) return;
            this.isAnimating = true;
            
            // 要素を表示
            this.overlay.style.display = 'block';
            
            // アニメーションクラスをリセット
            this.overlay.className = '';
            document.getElementById('command-palette-div').className = '';
            
            // 次のフレームでアニメーション開始
            requestAnimationFrame(() => {
                this.overlay.classList.add('show');
                document.getElementById('command-palette-div').classList.add('show');
            });
            
            // ボタンアイコン変更
            this.menuBtn.innerHTML = '';
            this.menuBtn.appendChild(this.closeIcon);
            
            // フォーカスとコマンド表示
            setTimeout(() => {
                this.inputField.focus();
                this.filterCommands('', this.resultsDiv);
                this.isAnimating = false;
            }, 100);
        }

        hidePalette() {
            if (this.isAnimating) return;
            this.isAnimating = true;
            
            // アニメーションクラスをリセット
            this.overlay.className = '';
            document.getElementById('command-palette-div').className = '';
            
            // 次のフレームでアニメーション開始
            requestAnimationFrame(() => {
                this.overlay.classList.add('hide');
                document.getElementById('command-palette-div').classList.add('hide');
            });
            
            // ボタンアイコン変更
            this.menuBtn.innerHTML = '';
            this.menuBtn.appendChild(this.paletteIcon);
            
            // 入力フィールドクリア
            this.inputField.value = '';
            this.selectedIndex = -1;
            
            // アニメーション完了後に非表示
            setTimeout(() => {
                this.overlay.style.display = 'none';
                this.overlay.className = '';
                document.getElementById('command-palette-div').className = '';
                this.isAnimating = false;
            }, 200);
        }

        filterCommands(query, resultsDiv) {
            const parts = query.trim().split(/\s+/);
            const baseCmd = parts[0] || '';
            const arg = parts[1] || '';

            // テーマコマンドの特別処理
            if (baseCmd.toLowerCase() === 'theme' && parts.length === 2) {
                this.filteredCommands = Object.keys(this.themes)
                    .filter(theme => theme.toLowerCase().includes(arg.toLowerCase()))
                    .map(theme => ({
                        cmd: `Theme ${theme}`,
                        desc: `Change to ${theme} theme`,
                        action: () => this.changeTheme([theme])
                    }));
            } else {
                this.filteredCommands = this.commandTree.filter(cmd => 
                    cmd.cmd.toLowerCase().includes(query.toLowerCase()) ||
                    cmd.desc.toLowerCase().includes(query.toLowerCase())
                );
            }
            
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

        handleKeyNavigation(e) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                this.selectedIndex = Math.min(this.selectedIndex + 1, this.filteredCommands.length - 1);
                this.renderResults(this.resultsDiv);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
                this.renderResults(this.resultsDiv);
            } else if (e.key === 'Tab') {
                e.preventDefault();
                if (this.selectedIndex >= 0 && this.filteredCommands[this.selectedIndex]) {
                    this.inputField.value = this.filteredCommands[this.selectedIndex].cmd;
                    this.filterCommands(this.inputField.value, this.resultsDiv);
                    setTimeout(() => {
                        this.inputField.setSelectionRange(this.inputField.value.length, this.inputField.value.length);
                    }, 0);
                }
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
                // コマンドライン引数の解析
                const cmdParts = this.inputField.value.trim().split(/\s+/);
                const args = cmdParts.slice(1);
                
                this.filteredCommands[index].action(args);
                this.hidePalette();
            }
        }

        changeTheme(args) {
            const themeName = args && args[0] ? args[0].toLowerCase() : null;
            
            if (!themeName) {
                console.log('Usage: Theme <white|dark|coffee|ocean|forest|sunset|kawaii|mono-dark|mono-white|paper>');
                return;
            }
            
            if (!this.themes[themeName]) {
                console.log(`Unknown theme: ${themeName}. Available themes: ${Object.keys(this.themes).join(', ')}`);
                return;
            }
            
            const theme = this.themes[themeName];
            const root = document.documentElement;
            
            // CSS変数を設定
            Object.entries(theme).forEach(([property, value]) => {
                root.style.setProperty(property, value);
            });
            
            // ローカルストレージに保存
            localStorage.setItem('selectedTheme', themeName);
            console.log(`Theme changed to: ${themeName}`);
        }

        // ページ読み込み時にテーマを復元
        loadSavedTheme() {
            const savedTheme = localStorage.getItem('selectedTheme');
            if (savedTheme && this.themes[savedTheme]) {
                this.changeTheme([savedTheme]);
            } else {
                // デフォルトはdarkテーマ
                this.changeTheme(['dark']);
            }
        }
    }

    new ConsoleEmulator();
});