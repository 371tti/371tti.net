---
title: エラー処理（Error Handling）
authors:
    - chatGPT
is_complete: true
---

# エラー処理（Error Handling）

Rust では、エラー処理にパニック（panic）とリザルト型（Result）を使い分けます。多くの場合、recoverable error（回復可能なエラー）は `Result<T, E>` で表現します。

## Result 型
```rust
fn divide(x: i32, y: i32) -> Result<i32, String> {
    if y == 0 {
        Err(String::from("0で割ることはできません"))
    } else {
        Ok(x / y)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println!("結果: {}", result),
        Err(e) => println!("エラー: {}", e),
    }
}
```

## panic!
致命的なエラー時には `panic!` マクロでプログラムを即座に停止できます。

## まとめ
Rust のエラー処理は安全で明示的です。