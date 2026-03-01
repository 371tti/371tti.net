---
title: サンプル記事1
authors: 
  - 371tti
is_complete: true
---
# MD記法 標準 + 拡張
[rawデータはこちら](/raw/samples/post1.md)

# 見出し1
本文テスト。**太字**、*強調*、~~取り消し~~、H₂O。  
==ハイライト== ||スポイラー||
数式インライン：$a^2 + b^2 = c^2$  
WikiLink: [[Rust]] とパイプ版 [[Language|Rust言語]]



## 見出し2
段落テスト。  
改行テスト  
次の行。

### 見出し3



## Blockquote

> これは普通の引用です。

> [!NOTE]
> 補足・注釈 これは Note タイプの GFM ブロック引用。

> [!TIP]
> ヒント・最適解

> [!WARNING]
> 注意（やや強め）

> [!IMPORTANT]
> 重要事項

> [!CAUTION]
> 危険・安全警告

## List

### Unordered
- item A
- item B
  - nested 1
  - nested 2

### Ordered (start=5)
5. five
6. six
7. seven

### Task list
- [x] 完了タスク
- [ ] 未完了タスク



## Code

```rust
fn main() {
    println!("Hello, world!");
}
````

```
Fenceなしコード
そのまま扱われる
```

Inline code: `let x = 10;`



## Table

| 名前    | 年齢 |  国籍  |
| :---- | -: | :--: |
| Alice | 23 | 🇯🇵 |
| Bob   | 34 | 🇺🇸 |
| Carol | 29 | 🇩🇪 |



## Math

インライン：$E = mc^2$
ディスプレイ：

$$
\int_0^\infty e^{-x} dx = 1
$$



## Footnote test

Markdown の脚注[^1] をテスト。

[^1]: これは脚注です。
    続きの行もテスト。



## Definition List

Term 1
: Definition 1

Term 2
: Definition 2



## HTML block test

<div class="test-block">
  <p>HTML ブロックはそのまま扱われる必要がある。</p>
</div>



## Link types

[通常リンク](https://example.com)

[https://example.com/auto](https://example.com/auto)

[test@example.com](mailto:test@example.com)

![画像テスト](https://placehold.jp/300x200.png "画像タイトル")
![画像テスト](https://placehold.jp/300x200.png "画像タイトル")
![画像テスト](https://placehold.jp/300x200.png "画像タイトル")

