# Research: Using nom for .smsg Parsing

## Overview

Research on using nom parser combinator library to rewrite the .smsg file parser.

## Decision: Use nom 7.x

### Rationale

- nom is explicitly required by the feature specification
- Provides composable parser combinators
- Better error reporting with position information
- Well-maintained and widely used

### Alternatives Considered

1. **Manual string parsing** - Rejected: Already implemented, violates spec requirement
2. **lalrpop** - Rejected: Heavy for this use case, more complex setup
3. **pest** - Rejected: Not as commonly used in Rust ecosystem

### Implementation Notes

The new parser uses the following nom combinators:
- `tag` - Match exact strings like "message", "{", "}"
- `is_not` - Match until whitespace
- `multispace0/multispace1` - Match whitespace
- `alt` - Try multiple alternatives
- `map` - Transform parser output
- `opt` - Make parser optional

The parser handles:
- Primitive types (string, int8, int64, etc.)
- Array types (int32[], float64[3])
- Nested types (user-defined message references)
- Comments (lines starting with #)
- Multiple message definitions per file

## References

- nom crate: https://crates.io/crates/nom
- nom docs: https://docs.rs/nom/latest/nom/
