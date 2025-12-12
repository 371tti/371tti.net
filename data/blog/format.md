

# 見出し1 {#main-title .big}
本文テスト。**太字**、*強調*、~~取り消し~~、H₂O。  
数式インライン：$a^2 + b^2 = c^2$  
WikiLink: [[Rust]] とパイプ版 [[Language|Rust言語]]



## 見出し2 {#h2-test .blue}
段落テスト。  
改行テスト  
次の行。

### 見出し3 {#h3-test}



## Blockquote

> これは普通の引用です。

> [!NOTE]
> これは Note タイプの GFM ブロック引用。

> [!TIP]
> テスト用 TIP。

> [!WARNING]
> 注意が必要。



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

![画像テスト](https://example.com/img.png "画像タイトル")

