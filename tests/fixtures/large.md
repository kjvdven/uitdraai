# Large fixture

Fixed document of about 1000 lines for `--timing` measurements.

## Section 1

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 1

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 1 | cell text |
| row 2 | 2 | cell text |
| row 3 | 3 | cell text |
| row 4 | 4 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^1]

[^1]: Footnote for section 1.

## Section 2

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 2

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 2 | cell text |
| row 2 | 4 | cell text |
| row 3 | 6 | cell text |
| row 4 | 8 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^2]

[^2]: Footnote for section 2.

## Section 3

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 3

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 3 | cell text |
| row 2 | 6 | cell text |
| row 3 | 9 | cell text |
| row 4 | 12 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^3]

[^3]: Footnote for section 3.

## Section 4

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 4

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 4 | cell text |
| row 2 | 8 | cell text |
| row 3 | 12 | cell text |
| row 4 | 16 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^4]

[^4]: Footnote for section 4.

## Section 5

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 5

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 5 | cell text |
| row 2 | 10 | cell text |
| row 3 | 15 | cell text |
| row 4 | 20 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^5]

[^5]: Footnote for section 5.

## Section 6

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 6

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 6 | cell text |
| row 2 | 12 | cell text |
| row 3 | 18 | cell text |
| row 4 | 24 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^6]

[^6]: Footnote for section 6.

## Section 7

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 7

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 7 | cell text |
| row 2 | 14 | cell text |
| row 3 | 21 | cell text |
| row 4 | 28 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^7]

[^7]: Footnote for section 7.

## Section 8

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 8

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 8 | cell text |
| row 2 | 16 | cell text |
| row 3 | 24 | cell text |
| row 4 | 32 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^8]

[^8]: Footnote for section 8.

## Section 9

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 9

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 9 | cell text |
| row 2 | 18 | cell text |
| row 3 | 27 | cell text |
| row 4 | 36 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^9]

[^9]: Footnote for section 9.

## Section 10

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 10

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 10 | cell text |
| row 2 | 20 | cell text |
| row 3 | 30 | cell text |
| row 4 | 40 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^10]

[^10]: Footnote for section 10.

## Section 11

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 11

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 11 | cell text |
| row 2 | 22 | cell text |
| row 3 | 33 | cell text |
| row 4 | 44 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^11]

[^11]: Footnote for section 11.

## Section 12

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 12

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 12 | cell text |
| row 2 | 24 | cell text |
| row 3 | 36 | cell text |
| row 4 | 48 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^12]

[^12]: Footnote for section 12.

## Section 13

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 13

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 13 | cell text |
| row 2 | 26 | cell text |
| row 3 | 39 | cell text |
| row 4 | 52 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^13]

[^13]: Footnote for section 13.

## Section 14

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 14

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 14 | cell text |
| row 2 | 28 | cell text |
| row 3 | 42 | cell text |
| row 4 | 56 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^14]

[^14]: Footnote for section 14.

## Section 15

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 15

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 15 | cell text |
| row 2 | 30 | cell text |
| row 3 | 45 | cell text |
| row 4 | 60 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^15]

[^15]: Footnote for section 15.

## Section 16

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 16

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 16 | cell text |
| row 2 | 32 | cell text |
| row 3 | 48 | cell text |
| row 4 | 64 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^16]

[^16]: Footnote for section 16.

## Section 17

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 17

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 17 | cell text |
| row 2 | 34 | cell text |
| row 3 | 51 | cell text |
| row 4 | 68 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^17]

[^17]: Footnote for section 17.

## Section 18

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 18

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 18 | cell text |
| row 2 | 36 | cell text |
| row 3 | 54 | cell text |
| row 4 | 72 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^18]

[^18]: Footnote for section 18.

## Section 19

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 19

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 19 | cell text |
| row 2 | 38 | cell text |
| row 3 | 57 | cell text |
| row 4 | 76 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^19]

[^19]: Footnote for section 19.

## Section 20

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 20

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 20 | cell text |
| row 2 | 40 | cell text |
| row 3 | 60 | cell text |
| row 4 | 80 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^20]

[^20]: Footnote for section 20.

## Section 21

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 21

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 21 | cell text |
| row 2 | 42 | cell text |
| row 3 | 63 | cell text |
| row 4 | 84 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^21]

[^21]: Footnote for section 21.

## Section 22

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 22

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 22 | cell text |
| row 2 | 44 | cell text |
| row 3 | 66 | cell text |
| row 4 | 88 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^22]

[^22]: Footnote for section 22.

## Section 23

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 23

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 23 | cell text |
| row 2 | 46 | cell text |
| row 3 | 69 | cell text |
| row 4 | 92 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^23]

[^23]: Footnote for section 23.

## Section 24

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 24

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 24 | cell text |
| row 2 | 48 | cell text |
| row 3 | 72 | cell text |
| row 4 | 96 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^24]

[^24]: Footnote for section 24.

## Section 25

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 25

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 25 | cell text |
| row 2 | 50 | cell text |
| row 3 | 75 | cell text |
| row 4 | 100 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^25]

[^25]: Footnote for section 25.

## Section 26

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 26

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 26 | cell text |
| row 2 | 52 | cell text |
| row 3 | 78 | cell text |
| row 4 | 104 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^26]

[^26]: Footnote for section 26.

## Section 27

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 27

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 27 | cell text |
| row 2 | 54 | cell text |
| row 3 | 81 | cell text |
| row 4 | 108 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^27]

[^27]: Footnote for section 27.

## Section 28

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 28

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 28 | cell text |
| row 2 | 56 | cell text |
| row 3 | 84 | cell text |
| row 4 | 112 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^28]

[^28]: Footnote for section 28.

## Section 29

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 29

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 29 | cell text |
| row 2 | 58 | cell text |
| row 3 | 87 | cell text |
| row 4 | 116 | cell text |

```python
def fib(n: int) -> int:
    """Naive Fibonacci."""
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

> A quote to exercise blockquotes.[^29]

[^29]: Footnote for section 29.

## Section 30

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 30

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 30 | cell text |
| row 2 | 60 | cell text |
| row 3 | 90 | cell text |
| row 4 | 120 | cell text |

```javascript
const fib = (n) => (n < 2 ? n : fib(n - 1) + fib(n - 2));
console.log(fib(10)); // 55
```

> A quote to exercise blockquotes.[^30]

[^30]: Footnote for section 30.

## Section 31

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 31

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 31 | cell text |
| row 2 | 62 | cell text |
| row 3 | 93 | cell text |
| row 4 | 124 | cell text |

```bash
for f in *.md; do
  uitdraai export "$f" --pdf
done
```

> A quote to exercise blockquotes.[^31]

[^31]: Footnote for section 31.

## Section 32

Uitdraai renders Markdown with **bold**, *italic*, `inline code` and [links](https://example.com). This paragraph exists to give the renderer ordinary prose to chew on, the bulk of any real document.

### Details 32

- Item 1 with `code` and ~~strike~~
- Item 2 with `code` and ~~strike~~
- Item 3 with `code` and ~~strike~~
- Item 4 with `code` and ~~strike~~

- [x] done
- [ ] todo

| Name | Value | Note |
|------|------:|------|
| row 1 | 32 | cell text |
| row 2 | 64 | cell text |
| row 3 | 96 | cell text |
| row 4 | 128 | cell text |

```rust
fn fib(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

> A quote to exercise blockquotes.[^32]

[^32]: Footnote for section 32.

