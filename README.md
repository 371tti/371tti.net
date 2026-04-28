# wk-371tti-net
自作 web backend で動く高性能Webドキュメントサーバーです。

# Roadmap
## Booting
- [x] git remote 連携
- [x] git versioning
- [x] mdの完全なSSR
- [x] 全種類のファイルを配信可能
- [x] ドキュメントの全文検索
- [x] ディレクトリページの自動生成生成
## Features1
- [x] リーダーの共通化
- [x] レンダリングのキャッシュ実装
- [x] 画像のサムネイル生成
## Features2
- [x] sudachi dict の mmap load
- [ ] github ソーシャルログイン

# 開発手順...
git cloneして
起動時に Sudachi dictionary を自動ダウンロードして `static/system.dic` にキャッシュします。
初回起動だけネットワークが必要です。


# えとせとら
[デフォルトのコンテンツリポジトリはこちら](https://github.com/371tti/371tti.net-contents)