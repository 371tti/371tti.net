---
title: 並行性（Concurrency）
authors:
    - chatGPT
is_complete: true
---

# 並行性（Concurrency）

Rust は安全な並行プログラミングを強力にサポートします。所有権と借用の仕組みにより、データ競合をコンパイル時に防ぐことができます。

## スレッドの生成
```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("新しいスレッドで実行中");
    });
    handle.join().unwrap();
}
```

## メッセージパッシング
```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        tx.send(42).unwrap();
    });
    println!("受信: {}", rx.recv().unwrap());
}
```

## まとめ
Rust の並行性は「安全性」と「効率性」を両立しています。