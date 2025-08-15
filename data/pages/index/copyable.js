document.addEventListener('DOMContentLoaded', () => {
    // コピー可能な要素に右クリックイベントを設定
    document.querySelectorAll('.copyable').forEach(element => {
        element.addEventListener('contextmenu', (event) => {
            const code = element.getAttribute('data-code'); // コピーするコードを取得

            // クリップボードにコードをコピー
            navigator.clipboard.writeText(code).then(() => {
                const message = element.querySelector('.copy-message');
                message.textContent = "Copied!";  // メッセージを「Copied!」に変更

                // 2秒後に元のメッセージに戻す
                setTimeout(() => {
                    message.textContent = "Right click to copy";
                }, 2000);
            }).catch(err => {
                console.error("コピーに失敗しました:", err);
            });

            // デフォルトのコンテキストメニューを無効化
            event.preventDefault();
        });
    });
});
