---
title: Complex Markdown Test
tags:
  - test
  - markdown
---

# Heading 1

## Table

| Column A | Column B | Column C |
|----------|----------|----------|
| cell 1   | cell 2   | cell 3   |
| `code`   | **bold** | *italic* |

## Code Block

```rust
#[derive(Debug)]
struct Config {
    // This # is not a TLP marker
    name: String, // #tag in comment
}

fn main() {
    println!("Hello, world!");
}
```

## Nested Lists

- Item 1
  - Sub-item 1a
    - Sub-sub-item
  - Sub-item 1b
- Item 2
  1. Ordered sub
  2. Another ordered

## Horizontal Rules

---

***

## Links and Images

[Example](https://example.com)
![Alt text](image.png "title")

## Blockquote

> This is a quote
> with multiple lines
>
> > Nested quote

## Footnotes

Text with footnote[^1].

[^1]: This is the footnote content.
