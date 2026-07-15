# TODO

Condensed from `code-review.md` and updated for the current implementation.

## Correctness and robustness

- [x] Make signed `gt` and `lt` comparisons overflow-safe by branching on operand signs before subtraction.
- [x] Add exact codegen tests for overflow-safe signed comparisons and internal comparison labels.
- [ ] Add execution-level comparison tests for signed boundary cases such as `32767 > -1` and `-32768 < 1`.
- [x] Prefix generated comparison labels with an internal namespace.
- [ ] Guarantee that internal labels cannot collide with static symbols derived from filenames.
- [ ] Make the VM tokenizer Unicode-safe; malformed non-ASCII input must return an error rather than panic.
- [ ] Enforce one VM command per line and reject trailing arguments.
- [ ] Stop assembler variable allocation before the `SCREEN` memory range and add a dedicated exhaustion error.
- [ ] Validate or collision-safely encode file stems used for static symbols.

## Diagnostics and I/O

- [ ] Add byte spans plus line/column information to assembler and VM tokens and parse errors.
- [ ] Include source locations in CLI diagnostics.
- [ ] Write outputs atomically and reject accidental source-path overwrites.
- [ ] Add tests for write failures, missing output directories, and source/output path equality.

## Testing

- [ ] Add a small Hack execution harness or integrate an emulator for behavioral VM translation tests.
- [ ] Add fuzz/property tests asserting that tokenizers and parsers never panic.
- [ ] Cover Unicode input, CRLF, long identifiers, comments at EOF, ROM limits, and large symbol tables.
- [ ] Add a regression test proving static symbols cannot collide with internal comparison labels.

## Maintainability

- [x] Replace positional template mutation in static push/pop generation with helpers accepting an A-instruction operand directly.
- [x] Propagate generated-label construction errors instead of using `expect` in comparison codegen.
- [ ] Choose and document a clearly one-shot or incremental `Codegen` API.
- [ ] Centralize internal symbol allocation before adding Project 8 commands or multi-file translation.
- [ ] Extract shared character-aware cursor/span logic while keeping crate-specific token classification.
- [ ] Consider separating comments and labels from executable Hack instructions if semantic passes are added.
- [ ] Narrow public module exposure and add rustdoc for supported entry points and VM scope.

## Cleanup

- [x] Rename `CodeGen` to `Codegen`.
- [ ] Use consistent `index`/`base` terminology in memory-access helpers.
- [ ] Remove the commented-out parser stub.
- [ ] Improve vague error names such as `AssemblyGenError`.
- [x] Keep the workspace manifest and lockfile committed for reproducible CLI builds.
