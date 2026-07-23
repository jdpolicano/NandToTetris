//! Shared source-location types for Hack language tooling.

use std::ops::Range;

/// A Unicode-aware, one-based location in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    /// Zero-based byte offset from the beginning of the source.
    pub offset: usize,
    /// One-based line number.
    pub line: usize,
    /// One-based Unicode scalar-value column.
    pub column: usize,
}

/// A half-open byte range and its corresponding source positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    /// Inclusive start position of the span.
    pub start: SourcePosition,
    /// Exclusive end position of the span.
    pub end: SourcePosition,
}

impl SourceSpan {
    /// Maps a half-open byte range to Unicode-aware line and column positions.
    ///
    /// The offsets must lie on UTF-8 character boundaries in `source`.
    pub fn new(source: &str, start: usize, end: usize) -> Self {
        assert!(start <= end && end <= source.len());
        assert!(source.is_char_boundary(start) && source.is_char_boundary(end));
        Self {
            start: position_at(source, start),
            end: position_at(source, end),
        }
    }

    /// Returns the zero-width span at end of input.
    pub fn eof(source: &str) -> Self {
        Self::new(source, source.len(), source.len())
    }

    /// Returns a zero-width span at `position`.
    pub const fn at(position: SourcePosition) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Returns the half-open byte range represented by this span.
    pub fn byte_range(self) -> std::ops::Range<usize> {
        self.start.offset..self.end.offset
    }

    /// Creates the smallest span covering both spans.
    pub fn cover(self, other: Self) -> Self {
        let start = if self.start.offset <= other.start.offset {
            self.start
        } else {
            other.start
        };
        let end = if self.end.offset >= other.end.offset {
            self.end
        } else {
            other.end
        };
        Self { start, end }
    }
}

/// Monotonically tracks Unicode-aware positions while a lexer advances.
///
/// A tracker is suitable for use as a Logos lexer's `Extras` value. Calling
/// [`span_for`](Self::span_for) with successive lexer ranges advances across
/// both emitted tokens and any skipped text exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionTracker {
    position: SourcePosition,
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self {
            position: SourcePosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        }
    }
}

impl PositionTracker {
    /// Returns the position immediately after the text processed so far.
    pub const fn position(&self) -> SourcePosition {
        self.position
    }

    /// Advances through skipped text and the supplied token range.
    ///
    /// Ranges must be supplied in nondecreasing order and must lie on UTF-8
    /// character boundaries.
    pub fn span_for(&mut self, source: &str, range: Range<usize>) -> SourceSpan {
        assert!(self.position.offset <= range.start);
        assert!(range.start <= range.end && range.end <= source.len());
        assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));

        self.advance_to(source, range.start);
        let start = self.position;
        self.advance_to(source, range.end);
        SourceSpan {
            start,
            end: self.position,
        }
    }

    /// Advances through all source text remaining after the last token.
    pub fn finish(&mut self, source: &str) -> SourcePosition {
        self.advance_to(source, source.len());
        self.position
    }

    fn advance_to(&mut self, source: &str, end: usize) {
        assert!(self.position.offset <= end && end <= source.len());
        assert!(source.is_char_boundary(end));

        for ch in source[self.position.offset..end].chars() {
            self.position.offset += ch.len_utf8();
            if ch == '\n' {
                self.position.line += 1;
                self.position.column = 1;
            } else {
                self.position.column += 1;
            }
        }
    }
}

fn position_at(source: &str, offset: usize) -> SourcePosition {
    let mut line = 1;
    let mut column = 1;
    for ch in source[..offset].chars() {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    SourcePosition {
        offset,
        line,
        column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_unicode_and_newlines() {
        let source = "αβ\r\n💥x";
        let bang = source.find('💥').unwrap();
        assert_eq!(
            SourceSpan::new(source, bang, bang + '💥'.len_utf8()),
            SourceSpan {
                start: SourcePosition {
                    offset: bang,
                    line: 2,
                    column: 1
                },
                end: SourcePosition {
                    offset: bang + 4,
                    line: 2,
                    column: 2
                },
            }
        );
        assert_eq!(SourceSpan::eof(source).end.line, 2);
        assert_eq!(SourceSpan::eof(source).end.column, 3);
    }

    #[test]
    fn tracker_matches_spans_while_advancing_once_through_skipped_text() {
        let source = "α  // comment\r\n💥x // trailing";
        let alpha_end = 'α'.len_utf8();
        let bang_start = source.find('💥').unwrap();
        let bang_end = bang_start + '💥'.len_utf8();
        let x_start = bang_end;

        let mut tracker = PositionTracker::default();
        assert_eq!(
            tracker.span_for(source, 0..alpha_end),
            SourceSpan::new(source, 0, alpha_end)
        );
        assert_eq!(
            tracker.span_for(source, bang_start..bang_end),
            SourceSpan::new(source, bang_start, bang_end)
        );
        assert_eq!(
            tracker.span_for(source, x_start..x_start + 1),
            SourceSpan::new(source, x_start, x_start + 1)
        );

        let eof = tracker.finish(source);
        assert_eq!(SourceSpan::at(eof), SourceSpan::eof(source));
        assert_eq!(eof.offset, source.len());
    }
}
