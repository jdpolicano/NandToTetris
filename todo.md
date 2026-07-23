# TODO

Condensed from `code-review.md` and updated for the current implementation.

## Correctness and robustness

- [x] Make signed `gt` and `lt` comparisons overflow-safe by branching on operand signs before subtraction.
- [x] Add exact codegen tests for overflow-safe signed comparisons and internal comparison labels.
- [x] Prefix generated comparison labels with an internal namespace.
- [x] Guarantee that internal labels cannot collide with static symbols derived from filenames.
- [x] Make the VM tokenizer Unicode-safe; malformed non-ASCII input must return an error rather than panic.
- [x] Enforce one VM command per line and reject trailing arguments.
- [x] Stop assembler variable allocation before the `SCREEN` memory range and add a dedicated exhaustion error.
- [x] Validate file stems used for static symbols and reject the reserved internal namespace.

## Diagnostics and I/O

- [x] Add byte spans plus line/column information to assembler and VM tokens and parse errors.
- [x] Include source locations in CLI diagnostics.
- [x] Write outputs atomically and reject accidental source-path overwrites.
- [x] Add tests for write failures, missing output directories, and source/output path equality.

## Testing

- [x] Add bounded property tests asserting that tokenizers and parsers never panic.
- [x] Cover Unicode input, CRLF, long identifiers, comments at EOF, ROM limits, and large symbol tables.
- [x] Add a regression test proving static symbols cannot collide with internal comparison labels.

## Maintainability

- [x] Replace positional template mutation in static push/pop generation with helpers accepting an A-instruction operand directly.
- [x] Propagate generated-label construction errors instead of using `expect` in comparison codegen.
- [x] Choose and document an incremental `Codegen` API with a one-shot translation convenience function.
- [x] Centralize internal symbol allocation before adding Project 8 commands or multi-file translation.
- [x] Extract shared character-aware cursor/span logic while keeping crate-specific token classification.
- [x] Narrow public module exposure and add rustdoc for supported entry points and VM scope.

## Cleanup

- [x] Rename `CodeGen` to `Codegen`.
- [x] Use consistent `index`/`base` terminology in memory-access helpers.
- [x] Remove the commented-out parser stub.
- [x] Improve vague generated-instruction error names.
- [x] Keep the workspace manifest and lockfile committed for reproducible CLI builds.

## Deferred

- Add a Hack execution harness or integrate an emulator. This is valuable for behavioral translation testing but is too large for the current phase.
- Add execution-level signed-boundary comparison tests such as `32767 > -1` and `-32768 < 1`; these depend on the deferred execution harness or emulator integration.
- Separate comments and labels from executable Hack instructions only if a future semantic analysis or optimization pass needs an executable-only representation. The current ordered representation remains useful for parsing and formatting.
