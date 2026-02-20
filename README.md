# Whale C

This repository is an experimental C compiler for the Whale toolchain and extracts Whale IR.

run command:
```bash
cargo run -p whale-c -- examples/demo.c
```

`demo.c`

```c
const int A = 123;

int add(int a, int b) {
    return a + b;
}

int main() {
    int x;
    x = A;
    if (x > 100) {
        x = x + 1;
    } else {
        x = x - 1;
    }
    return x;
}
```

`output.wir`

```llvm
module {
  target "x86_64-whale-linux"
  datalayout { ptr=64, endian=little }

  global @A: i32 = const i32 123, align 4

  fn @add(a: i32, b: i32) -> i32 {
  entry:
    %v2: ptr<i32> = alloca i32, align 4
    %v3: ptr<i32> = alloca i32, align 4
    store i32 %v0, ptr<i32> %v2, align 4
    store i32 %v1, ptr<i32> %v3, align 4
    %v4: i32 = load i32, ptr<i32> %v2, align 4
    %v5: i32 = load i32, ptr<i32> %v3, align 4
    %v6: i32 = add i32 %v4, %v5
    ret i32 %v6
  }

  fn @main() -> i32 {
  entry:
    %v8: ptr<i32> = alloca i32, align 4
    %v7: i32 = undef i32
    store i32 %v7, ptr<i32> %v8, align 4
    %v9: i32 = const i32 123
    store i32 %v9, ptr<i32> %v8, align 4
    %v10: i32 = load i32, ptr<i32> %v8, align 4
    %v11: i32 = const i32 100
    %v12: i1 = cmp sgt i32 %v10, %v11
    cbr i1 %v12, label 2, label 3
  if.then:
    %v13: i32 = load i32, ptr<i32> %v8, align 4
    %v14: i32 = const i32 1
    %v15: i32 = add i32 %v13, %v14
    store i32 %v15, ptr<i32> %v8, align 4
    br label 4
  if.else:
    %v16: i32 = load i32, ptr<i32> %v8, align 4
    %v17: i32 = const i32 1
    %v18: i32 = sub i32 %v16, %v17
    store i32 %v18, ptr<i32> %v8, align 4
    br label 4
  if.cont:
    %v19: i32 = load i32, ptr<i32> %v8, align 4
    ret i32 %v19
  }

}
```