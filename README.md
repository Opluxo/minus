# minus

A statically-typed, compiled programming language that generates LLVM IR, written in Rust.

## Features

- Static type system with type checking
- LLVM IR code generation
- C-style syntax with some modern features
- Built-in `Print` function for output
- String interpolation with `$"Hello {name}"`
- `Result<T, E>` type for error handling
- Block-level scoping
- `struct` compound types

## Installation

```bash
# Requires Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/Opluxo/minus.git
cd minus
cargo build --release
```

## Usage

```bash
# Compile a .mi file
cargo run -- input.mi

# Execute code directly
cargo run -- -e 'void main() { Print(42); }'

# Run tests
cargo test
```

## Language Syntax

### Variables

```min
int x = 10;
float pi = 3.14;
string name = "minus";
bool flag = true;
char c = 'a';
```

### Functions

```min
int add(int a, int b) {
    return a + b;
}

void main() {
    int result = add(1, 2);
    Print(result);
}
```

### Control Flow

```min
# if/else
if (x > 0) {
    Print("positive");
} else if (x == 0) {
    Print("zero");
} else {
    Print("negative");
}

# while loop
while (x > 0) {
    x = x - 1;
}

# for loop
for (int i = 0; i < 10; i = i + 1) {
    Print(i);
}

# switch statement
switch (x) {
    case 1:
        Print("one");
        break;
    case 2:
        Print("two");
        break;
    default:
        Print("other");
        break;
}
```

### Break and Continue

```min
for (int i = 0; i < 10; i = i + 1) {
    if (i == 3) {
        continue;
    }
    if (i == 7) {
        break;
    }
    Print(i);
}
```

### Structs

```min
struct Point {
    x: int,
    y: int
}

void main() {
    Point p;
    p.x = 10;
    p.y = 20;
}
```

### String Operations

```min
string s = "hello";
int len = s.len();           # 获取长度
string ch = s.at(0);         # 获取字符
string combined = s + " world";  # 字符串拼接

# 字符串插值
string name = "minus";
string greeting = $"Hello, {name}!";
```

### Result Type

```min
Result<int, string> divide(int a, int b) {
    if (b == 0) {
        return error("division by zero");
    }
    return ok(a / b);
}
```

### Comments

```min
# This is a single line comment
```

## Type System

| Type | Description |
|------|-------------|
| `int` | 32-bit signed integer |
| `float` | 64-bit floating point |
| `char` | Single character |
| `bool` | Boolean value (`true`/`false`) |
| `string` | String literal |
| `void` | No return value |
| `struct` | User-defined compound type |
| `Result<T, E>` | Ok/Error result type |

## Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `%` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical | `&&` `\|\|` `!` |
| Assignment | `=` |
| String | `+` (concatenation) |

## Examples

See the [examples](examples/) directory:

- [hello.mi](examples/hello.mi) - Hello World
- [fibonacci.mi](examples/fibonacci.mi) - Fibonacci sequence
- [struct_example.mi](examples/struct_example.mi) - Struct usage

## License

MIT
