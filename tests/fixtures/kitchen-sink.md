# Kitchen sink

Every element once, to check a theme in the preview and the PDF. Open it with your theme and save the CSS to see it change.

## Text

A paragraph with **bold**, *italic*, ***both***, ~~strike~~, `inline code`, a [link](https://example.com), an autolink https://example.com and a footnote.[^1]

A second paragraph, long enough to wrap over more than one line in the preview column, so line height and paragraph spacing are visible next to each other.

A hard line break\
right here, and raw HTML: <kbd>Ctrl</kbd>+<kbd>R</kbd> (stripped unless `--allow-html`).

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6

## Quotes

> A quote with **bold** and `code`.
>
> > A nested quote.

## Lists

- Unordered
- With a nested list
  - Second level
    - Third level
- Back to the first level

1. Ordered
2. With a nested list
   1. Second level
3. Back to the first level

- Loose list item

  With a second paragraph.

- List item with a code block:

  ```sh
  uitdraai notes.md
  ```

- [x] Done task
- [ ] Open task

## Table

| Left | Center | Right |
|:-----|:------:|------:|
| text | text | 1 |
| longer text | `code` | 1000 |

## Code

```rust
/// Doc comment.
fn main() {
    let greeting = "hello";
    println!("{greeting}, {}", 42);
}
```

```python
def fib(n: int) -> int:
    # A comment.
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

```
Plain code block without a language, with a line that is long enough to need horizontal scrolling in the preview.
```

## Rule and image

---

![A screenshot of uitdraai](../../data/screenshot.png)

## Math

Inline $E = mc^2$ and display:

$$
x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}
$$

Broken on purpose: $\frac{1}$

## Chords

```chordpro
{title: Amazing Grace}
{artist: John Newton}
{key: G}
{time: 3/4}
{define: G base-fret 1 frets 3 2 0 0 0 3}

{start_of_verse: Verse 1}
A[G]mazing [G7]grace, how [C]sweet the [G]sound
That [G]saved a wretch like [D]me
{end_of_verse}

{start_of_tab}
e|-----3-----|
B|-------0---|
{end_of_tab}
```

[^1]: The footnote text.
