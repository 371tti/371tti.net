---
title: ジェネリクスとトレイト（Generics & Traits）
authors:
    - chatGPT
is_complete: true
---

# ジェネリクスとトレイト（Generics & Traits）

Rust では、型に依存しない汎用的なコードを「ジェネリクス」で書くことができます。また、トレイトは型に共通の振る舞いを定義します。

## ジェネリクスの例
```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

## トレイトの例
```rust
trait Greet {
    fn greet(&self);
}

struct Person;

impl Greet for Person {
    fn greet(&self) {
        println!("Hello!");
    }
}
```

## まとめ
ジェネリクスとトレイトにより、柔軟で再利用性の高いコードが書けます。