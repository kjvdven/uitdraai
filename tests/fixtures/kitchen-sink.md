# Kitchen sink

Every element once, to check a theme in the preview and the PDF. Open it with your theme and save the CSS to see it change.

## Text

A paragraph with **bold**, *italic*, ***both***, ~~strike~~, `inline code`, a [link](https://example.com), an autolink https://example.com and a footnote.[^1]

A second paragraph, long enough to wrap over more than one line in the preview column, so line height and paragraph spacing are visible next to each other.

A hard line break\
right here, and raw HTML: <kbd>Ctrl</kbd>+<kbd>R</kbd> (stripped unless `--allow-html`).

### Heading 3

Text right under a third-level heading. Most documents live at this depth, so the gap between heading and paragraph matters more here than anywhere else. A few sentences make the paragraph wrap, which shows the line height against the heading's own spacing.

A follow-up paragraph, then the next heading straight after it.

#### Heading 4

Text under a fourth-level heading, with **bold** and a [link](https://example.com) to see whether they still stand out at a smaller heading size.

##### Heading 5

Text under a fifth-level heading. At this size the heading can get close to body text; it should still read as a heading.

###### Heading 6

Text under the smallest heading.

## Headings and blocks

Every block element straight under a heading, the spacing a theme gets wrong most often.

### Heading then list

- First item
- Second item

### Heading then code

```sh
cargo run -- notes.md
```

### Heading then table

| Key | Value |
|-----|-------|
| a | 1 |

### Heading then quote

> A quote right under a heading.

### Heading then heading

#### Straight into a subheading

And only then a paragraph.

## A longer section

This section reads like a real document, so the rhythm of prose and headings is visible over more than one screen. Uitdraai is a Markdown previewer: you write in your own editor, and every save updates the preview in place without losing the scroll position.

The paragraphs are of different lengths on purpose. A short one.

Then a longer one again, with a sentence that goes on for a while, mentions `inline code` and *emphasis* along the way, and only ends after it has wrapped over at least two lines in the preview column, which is about 48rem wide.

### Why themes matter

A theme decides more than colours. It sets the measure, the line height, the spacing between blocks and the way headings group the text below them. When that spacing is off, the reader loses track of which paragraph belongs to which heading.

Good spacing keeps a heading closer to the text below it than to the text above it. This paragraph and the one above should clearly belong together, under the heading "Why themes matter".

### Printing

The PDF uses the same theme, plus `print.css` for the page size, margins and page numbers. Headings shouldn't end up alone at the bottom of a page, and code blocks shouldn't break in half.

Export this file to PDF and check the page breaks: this longer section is there to push some headings close to a page boundary.

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
