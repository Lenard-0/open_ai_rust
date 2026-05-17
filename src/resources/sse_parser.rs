//! Pure (no I/O) Server-Sent Events parser used by the streaming HTTP code path.
//!
//! Extracted from [`super::stream::post_sse_stream`] so the parsing logic — which is the
//! most-likely-to-break part of streaming — has direct unit-test coverage independent of
//! the HTTP layer.
//!
//! References: [WHATWG SSE spec](https://html.spec.whatwg.org/multipage/server-sent-events.html).

/// Outcome of pulling one logical event off the buffer.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SseEvent {
    /// A `data:` event with the concatenated payload (excluding the field name).
    Data(String),
    /// `data: [DONE]` terminator — the consumer should stop yielding.
    Done,
}

/// Try to pull one event off the front of `buffer`. Returns `Some(event)` and drains the
/// consumed bytes; returns `None` if the buffer doesn't yet contain a complete event.
///
/// Skips comment lines (`:foo`), `event:`/`id:`/`retry:` fields, and empty events.
/// Honours both `\n\n` and `\r\n\r\n` event terminators, choosing whichever appears first
/// in the buffer.
///
/// Multi-line `data:` fields are concatenated with `\n` per spec. A single optional space
/// after `data:` is stripped (and only one — `"data:  x"` keeps one leading space).
pub(crate) fn parse_next_event(buffer: &mut String) -> Option<SseEvent> {
    loop {
        // SSE events end with a blank line. Look for either CR LF CR LF or LF LF; pick
        // whichever appears earliest in the buffer.
        let crlf = buffer.find("\r\n\r\n");
        let lf = buffer.find("\n\n");
        let (idx, sep_len) = match (crlf, lf) {
            (Some(c), Some(l)) if c <= l => (c, 4),
            (Some(c), None) => (c, 4),
            (_, Some(l)) => (l, 2),
            (None, None) => return None,
        };

        let raw_event = buffer[..idx].to_string();
        buffer.drain(..idx + sep_len);

        let mut data = String::new();
        for line in raw_event.split('\n').map(strip_trailing_cr) {
            if line.is_empty() {
                continue;
            }
            if line.starts_with(':') {
                // Comment line — ignore.
                continue;
            }
            // Split into field name + value at the first colon.
            let (field, value) = match line.split_once(':') {
                Some((f, v)) => (f, strip_one_leading_space(v)),
                None => (line, ""),
            };
            if field == "data" {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(value);
            }
            // Other field names (`event`, `id`, `retry`) — ignored. OpenAI doesn't use them.
        }

        if data.is_empty() {
            // Empty event (e.g. comment-only or all unknown fields) — keep scanning.
            continue;
        }
        if data == "[DONE]" {
            return Some(SseEvent::Done);
        }
        return Some(SseEvent::Data(data));
    }
}

fn strip_trailing_cr(s: &str) -> &str {
    s.strip_suffix('\r').unwrap_or(s)
}

fn strip_one_leading_space(s: &str) -> &str {
    s.strip_prefix(' ').unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain_all(mut buf: String) -> (Vec<SseEvent>, String) {
        let mut out = Vec::new();
        while let Some(ev) = parse_next_event(&mut buf) {
            out.push(ev);
        }
        (out, buf)
    }

    #[test]
    fn simple_single_event() {
        let mut buf = "data: hello\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("hello".into()))
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn crlf_separator() {
        let mut buf = "data: hi\r\n\r\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("hi".into()))
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn crlf_separator_does_not_leak_stray_cr_lf() {
        // Regression guard for the pre-extraction bug where `find("\n\n")` matched
        // inside `\r\n\r\n` (1 byte past the start) and left `\r\n` in the buffer.
        let buf = "data: a\r\n\r\ndata: b\r\n\r\n".to_string();
        let (events, rest) = drain_all(buf);
        assert_eq!(
            events,
            vec![SseEvent::Data("a".into()), SseEvent::Data("b".into()),]
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn lf_picked_when_earlier_than_crlf() {
        let buf = "data: short\n\ndata: long-with-cr\r\n\r\n".to_string();
        let (events, _) = drain_all(buf);
        assert_eq!(
            events,
            vec![
                SseEvent::Data("short".into()),
                SseEvent::Data("long-with-cr".into()),
            ]
        );
    }

    #[test]
    fn crlf_picked_when_earlier_than_lf() {
        // Regression for a mutation-testing-discovered gap: when CRLF appears before LF
        // in the buffer (event 1 ends with CRLF, event 2 ends with LF), the first event
        // must consume the 4-byte separator, not 2. Otherwise stray `\r\n` leaks into
        // event 2.
        let buf = "data: first\r\n\r\ndata: second\n\n".to_string();
        let (events, rest) = drain_all(buf);
        assert_eq!(
            events,
            vec![
                SseEvent::Data("first".into()),
                SseEvent::Data("second".into()),
            ]
        );
        assert!(rest.is_empty(), "trailing bytes leaked: {rest:?}");
    }

    #[test]
    fn multi_line_data_concatenated_with_newline() {
        let mut buf = "data: line1\ndata: line2\ndata: line3\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("line1\nline2\nline3".into()))
        );
    }

    #[test]
    fn comment_lines_ignored() {
        let buf = ":keep-alive\n:another\n\ndata: real\n\n".to_string();
        let (events, _) = drain_all(buf);
        assert_eq!(events, vec![SseEvent::Data("real".into())]);
    }

    #[test]
    fn event_and_id_fields_ignored() {
        let mut buf = "event: foo\nid: 42\nretry: 1000\ndata: payload\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("payload".into()))
        );
    }

    #[test]
    fn done_terminator() {
        let mut buf = "data: [DONE]\n\n".to_string();
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Done));
    }

    #[test]
    fn done_followed_by_trailing_garbage_keeps_garbage_in_buffer() {
        let mut buf = "data: [DONE]\n\nextra".to_string();
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Done));
        assert_eq!(buf, "extra");
    }

    #[test]
    fn partial_buffer_returns_none_without_draining() {
        let mut buf = "data: partial".to_string();
        assert_eq!(parse_next_event(&mut buf), None);
        assert_eq!(buf, "data: partial");
    }

    #[test]
    fn partial_buffer_with_one_newline_returns_none() {
        let mut buf = "data: nearly\n".to_string();
        assert_eq!(parse_next_event(&mut buf), None);
        assert_eq!(buf, "data: nearly\n");
    }

    #[test]
    fn multiple_events_in_one_buffer() {
        let buf = "data: 1\n\ndata: 2\n\ndata: 3\n\n".to_string();
        let (events, rest) = drain_all(buf);
        assert_eq!(
            events,
            vec![
                SseEvent::Data("1".into()),
                SseEvent::Data("2".into()),
                SseEvent::Data("3".into()),
            ]
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn no_space_after_data_colon() {
        let mut buf = "data:no-space\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("no-space".into()))
        );
    }

    #[test]
    fn only_one_space_after_data_colon_stripped() {
        // Per spec, exactly ONE optional space is stripped. Extra spaces are preserved.
        let mut buf = "data:  two-spaces\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data(" two-spaces".into()))
        );
    }

    #[test]
    fn empty_event_skipped_then_real_event_returned() {
        let mut buf = "event: foo\n\ndata: actual\n\n".to_string();
        // First "event" has no `data:` field → empty, skip, fall through to next.
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("actual".into()))
        );
    }

    #[test]
    fn buffer_emptied_after_full_consumption() {
        let mut buf = "data: x\n\n".to_string();
        parse_next_event(&mut buf);
        assert!(buf.is_empty());
        assert_eq!(parse_next_event(&mut buf), None);
    }

    #[test]
    fn unicode_payload_preserved() {
        let mut buf = "data: héllo 🦀\n\n".to_string();
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("héllo 🦀".into()))
        );
    }

    #[test]
    fn json_payload_with_embedded_colons_intact() {
        let mut buf = r#"data: {"k": "v: still works"}"#.to_string() + "\n\n";
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data(r#"{"k": "v: still works"}"#.into()))
        );
    }

    #[test]
    fn split_mid_event_then_completed_in_next_call() {
        // Simulates the byte stream arriving in chunks: parser sees half an event first,
        // returns None, then sees the rest and parses successfully.
        let mut buf = String::new();
        buf.push_str("data: par");
        assert_eq!(parse_next_event(&mut buf), None);
        assert_eq!(buf, "data: par");
        buf.push_str("tial\n\n");
        assert_eq!(
            parse_next_event(&mut buf),
            Some(SseEvent::Data("partial".into()))
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn split_inside_crlf_terminator() {
        // Buffer contains `data: x\r\n\r` — looks like the start of CRLF CRLF but
        // missing the final `\n`. Should return None.
        let mut buf = "data: x\r\n\r".to_string();
        assert_eq!(parse_next_event(&mut buf), None);
        assert_eq!(buf, "data: x\r\n\r");
        // Once the final \n arrives the event is complete.
        buf.push('\n');
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Data("x".into())));
    }

    #[test]
    fn done_with_extra_whitespace_around() {
        // `data: [DONE]` is exact-match; `data:[DONE]` also works because we strip one
        // optional space. But `data:  [DONE]` (two spaces) would leave one space and
        // fail the equality check. Document behaviour.
        let mut buf = "data:[DONE]\n\n".to_string();
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Done));

        let mut buf2 = "data:  [DONE]\n\n".to_string();
        // Two spaces → one stripped → " [DONE]" → not the terminator, surfaces as Data.
        assert_eq!(
            parse_next_event(&mut buf2),
            Some(SseEvent::Data(" [DONE]".into()))
        );
    }

    #[test]
    fn comment_only_buffer_returns_none_after_drain() {
        // A buffer that contains only a comment event has no data → should return None,
        // leaving the buffer empty (the comment was consumed).
        let mut buf = ":heartbeat\n\n".to_string();
        assert_eq!(parse_next_event(&mut buf), None);
        assert!(
            buf.is_empty(),
            "comment-only event should be drained, got: {buf:?}"
        );
    }

    #[test]
    fn leading_newline_in_event_skipped_via_empty_line_branch() {
        // A buffer that begins with `\n` produces an empty first line after split('\n').
        // The empty-line `continue` branch should fire, then the real `data:` line
        // is parsed normally.
        let mut buf = "\ndata: x\n\n".to_string();
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Data("x".into())));
    }

    #[test]
    fn line_without_colon_is_treated_as_field_without_value() {
        // A line like `weird` has no `:` separator. `split_once(':')` returns None →
        // the `None => (line, "")` arm fires → field is "weird", value is "". Since
        // field != "data", the line is ignored, leaving only the proper data line.
        let mut buf = "weird\ndata: real\n\n".to_string();
        assert_eq!(parse_next_event(&mut buf), Some(SseEvent::Data("real".into())));
    }

    #[test]
    fn back_to_back_data_then_done() {
        let buf = "data: hi\n\ndata: [DONE]\n\n".to_string();
        let (events, rest) = drain_all(buf);
        assert_eq!(events, vec![SseEvent::Data("hi".into()), SseEvent::Done]);
        assert!(rest.is_empty());
    }
}
