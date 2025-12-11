# Aliter Remaining Work

## Current Status
- **874 tests passing** across 14 test files
- **0 ignored tests**
- **0 failing tests**

## Completed Work
- Fixed nested tests in `spec.rs` (9 tests now running)
- Fixed `the_mathchoice_function` - added `allowed_in_argument: true`
- Fixed `symbols_2` - test was comparing parse trees, not rendered output
- Fixed `unicode_accents` - use Unicode dotless-i `ı` instead of `\i`
- Fixed combining character test - was using double accents instead of single

## Remaining Work

### 1. TODOs (197 in src/, 8 in tests/)
**High Priority:**
- **12 "Don't panic" TODOs** - Functions that panic instead of returning errors
- **Error handling** - Many places use `.unwrap()` or `panic!` instead of proper error propagation
- **Strict mode warnings** - Console warnings not implemented (only errors)

**Medium Priority:**
- Memory/performance optimizations (clones, allocations)
- Parser position info in errors
- Optional argument support for `\newcommand`

**Low Priority:**
- Code cleanup (unused variables, better naming)
- Documentation

### 2. Commented Out Test Assertions (2)
```
spec.rs:1820 - \url with newline handling
spec.rs:2081 - \def with \expandafter (needs proper macro expansion)
```

### 3. Wide Characters (Mathematical Alphanumeric Symbols)
Characters in U+1D400-U+1D7FF range (bold/italic/script letters) not supported.
See `// FIXME: Wide characters` comment in symbols.rs.

### 4. Test Framework Limitations
Several tests use `to_build_like` / `to_parse_like` which compare parse trees.
Some equivalences only hold at render time (e.g., `\ae` vs `æ`, `\mathchoice`).
TODO: Add rendered output comparison for these tests.

### 5. Test Coverage Gaps (from TODOs in tests/)
- Atom type verification tests (ord, bin, rel, punct, open, close)
- Supsub node structure tests

### 6. Feature Gaps (matches KaTeX)
- `\strut` (only `\mathstrut` supported - same as KaTeX)
- `\mspace` (not supported - same as KaTeX)
- `\cancelto` (not supported - same as KaTeX)

## Summary Table
| Category | Count |
|----------|-------|
| Passing tests | 874 |
| Failing tests | 0 |
| TODOs | ~200 |
| Panic points to fix | 12 |
| Critical missing features | 0 (matches KaTeX) |
