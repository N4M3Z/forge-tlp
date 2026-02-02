---
title: Source Code Samples
---

## Python

```python
# This is a Python comment — not a TLP marker
import os
from pathlib import Path

# Another comment with hash
def process(data):
    """Process data."""
    return {k: v for k, v in data.items() if v is not None}
```

## Rust

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```

## C/C++

```c
#include <stdio.h>
#define MAX_SIZE 100
#ifdef DEBUG
    #pragma message("Debug mode")
#endif
```

## Shell

```bash
#!/usr/bin/env bash
set -euo pipefail
# Deploy script
echo "Deploying..."
```

## CSS

```css
/* Color values with # */
body {
    color: #333;
    background: #f0f0f0;
}
.highlight { border: 1px solid #ff0000; }
```
