# wk-371tti-net
自作 web backend で動く高性能Webドキュメントサーバーです。

# 機能
- git remote 連携
- git versioning
- mdの完全なSSR
- 全種類のファイルを配信可能
- ドキュメントの全文検索
- ディレクトリページの自動生成生成

## Git submoduleについて
デフォルトのコンテンツリポジトリは  
基本的にこのソフト内では初期化しますが開発環境のgitクライアントに反映されない場合は以下

1. 以下でサブモジュールを初期化して
```bash
git submodule update --init --recursive
```

以上