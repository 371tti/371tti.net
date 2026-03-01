---
title: Rust Code Examples
authors:
    - chatGPT
is_complete: true
---

# Rust Code Examples

このファイルでは、Rust の基本的なコード例をいくつか紹介します。これらの例を通じて、Rust の特徴や使い方を学ぶことができます。

## Hello, World!
Rust の基本的なプログラムは以下のように書きます：

```rust
fn main() {
    println!("Hello, world!");
}
```

このコードは、コンソールに "Hello, world!" を出力します。

## 所有権（Ownership）
Rust の所有権システムの基本的な例：

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // 所有権が s1 から s2 に移動

    // println!("{}", s1); // エラー: s1 はもう有効ではない
    println!("{}", s2);
}
```

この例では、`s1` の所有権が `s2` に移動するため、`s1` を使用することはできません。

## 借用（Borrowing）
所有権を移動せずに値を参照する例：

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);

    println!("The length of '{}' is {}.", s1, len);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

このコードでは、`&s1` を渡すことで、所有権を移動せずに関数内で値を使用しています。

## 並行性（Concurrency）
スレッドを使用した並行処理の例：

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap();
}
```

この例では、スレッドを生成して並行して実行しています。

## まとめ
これらのコード例は、Rust の基本的な概念を学ぶための出発点です。さらに詳しい情報は公式ドキュメントを参照してください。