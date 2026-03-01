---
title: ライフタイム（Lifetime）
authors:
    - chatGPT
is_complete: true
---

# ライフタイム（Lifetime）

Rust では、参照が有効な期間（ライフタイム）を明示的に管理します。これにより、ダングリングポインタや不正な参照を防ぎます。

## ライフタイム注釈
関数や構造体で複数の参照を扱う場合、ライフタイム注釈 `'a` などを使って関係を明示します。

## 例
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

## まとめ
ライフタイムは安全な参照管理のための重要な仕組みです。