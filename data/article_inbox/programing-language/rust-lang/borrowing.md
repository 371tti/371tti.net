---
title: 借用（Borrowing）
authors:
    - chatGPT
is_complete: true
---

# 借用（Borrowing）

Rustでは、値の所有権を移動せずに他の場所から参照できる仕組みとして「借用」がある。
所有権をムーブせずに値を使いたいとき、借用を使う。
これにより、複数の場所から安全にデータへアクセスできる。

## 借用の種類
- 不変借用（&T）：読み取り専用の参照。何個でも同時に作れる。
- 可変借用（&mut T）：書き込み可能な参照。同時に1つだけ。

Rustは「不変借用と可変借用を同時に持てない」ルールでデータ競合を防ぐ。

## 例：不変借用と可変借用
```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &s; // 不変借用
    let r2 = &s; // 不変借用は複数OK
    // let r3 = &mut s; // エラー: 不変借用が存在中は可変借用できない
    println!("{} {}", r1, r2);
}
```

可変借用は同時に1つだけ許される。
```rust
fn main() {
    let mut s = String::from("hello");
    let r = &mut s; // 可変借用
    r.push_str(" world");
    println!("{}", r);
}
```

## 借用と関数
関数に参照を渡すことで、所有権を移さずに値を使える。
```rust
fn calc_len(s: &String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("hello");
    let len = calc_len(&s); // sの所有権はそのまま
    println!("{} {}", s, len);
}
```

## 注意点
- 借用のルールは最初は厳しく感じるが、データ競合や不正なメモリアクセスを防げる。
- 可変借用と不変借用は同時に存在できない。

## まとめ
借用はRustの安全性の要だ。所有権を移さずに値を使いたいときは必ず借用を使う。