//! Minimal XML tokenizer for MSBuild project files.
//!
//! MSBuild files use a small, regular slice of XML: elements, quoted
//! attributes, text, comments, an XML declaration and the occasional CDATA
//! section. This scanner turns that slice into a flat event list; it does
//! not validate nesting, and markup it does not model (DOCTYPE, processing
//! instructions) is skipped rather than rejected.

/// One token of a scanned document, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum XmlEvent {
    Open {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    Close {
        name: String,
    },
    Text(String),
}

/// Scans `document` into events. Whitespace-only text is dropped; text and
/// attribute values have the predefined entities decoded.
pub(super) fn scan(document: &str) -> Vec<XmlEvent> {
    let mut events = Vec::new();
    let mut rest = document;
    while let Some(tag_start) = rest.find('<') {
        push_text(&mut events, &rest[..tag_start]);
        rest = consume_markup(&rest[tag_start..], &mut events);
    }
    push_text(&mut events, rest);
    events
}

fn push_text(events: &mut Vec<XmlEvent>, raw: &str) {
    if !raw.trim().is_empty() {
        events.push(XmlEvent::Text(decode_entities(raw)));
    }
}

/// Consumes one markup construct at the head of `markup` (which starts with
/// `<`) and returns the remainder after it.
fn consume_markup<'a>(markup: &'a str, events: &mut Vec<XmlEvent>) -> &'a str {
    if let Some(body) = markup.strip_prefix("<!--") {
        return skip_past(body, "-->");
    }
    if let Some(body) = markup.strip_prefix("<![CDATA[") {
        let end = body.find("]]>").unwrap_or(body.len());
        events.push(XmlEvent::Text(body[..end].to_string()));
        return skip_past(body, "]]>");
    }
    if markup.starts_with("<?") || markup.starts_with("<!") {
        return skip_past(markup, ">");
    }
    let tag_end = find_tag_end(markup);
    let inner = &markup[1..tag_end];
    let remainder = markup.get(tag_end + 1..).unwrap_or("");
    if let Some(closing) = inner.strip_prefix('/') {
        events.push(XmlEvent::Close {
            name: closing.trim().to_string(),
        });
    } else {
        events.push(open_event(inner));
    }
    remainder
}

fn skip_past<'a>(text: &'a str, terminator: &str) -> &'a str {
    match text.find(terminator) {
        Some(position) => &text[position + terminator.len()..],
        None => "",
    }
}

/// Byte index of the `>` closing the tag that starts `markup`, ignoring any
/// `>` inside quoted attribute values; the end of input if unterminated.
fn find_tag_end(markup: &str) -> usize {
    let mut open_quote: Option<char> = None;
    for (index, character) in markup.char_indices() {
        match (open_quote, character) {
            (None, '"' | '\'') => open_quote = Some(character),
            (Some(quote), _) if character == quote => open_quote = None,
            (None, '>') => return index,
            _ => {}
        }
    }
    markup.len()
}

fn open_event(inner: &str) -> XmlEvent {
    let trimmed = inner.trim_end();
    let self_closing = trimmed.ends_with('/');
    let body = trimmed.trim_end_matches('/');
    let name_end = body
        .find(|character: char| character.is_whitespace())
        .unwrap_or(body.len());
    XmlEvent::Open {
        name: body[..name_end].to_string(),
        attributes: parse_attributes(&body[name_end..]),
        self_closing,
    }
}

fn parse_attributes(mut text: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    while let Some(equals) = text.find('=') {
        let key = text[..equals].trim().to_string();
        let after_equals = text[equals + 1..].trim_start();
        let Some(quote) = after_equals.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            break;
        };
        let value_body = &after_equals[1..];
        let value_end = value_body.find(quote).unwrap_or(value_body.len());
        attributes.push((key, decode_entities(&value_body[..value_end])));
        text = value_body.get(value_end + 1..).unwrap_or("");
    }
    attributes
}

/// Decodes the five predefined XML entities (`&amp;` last so an escaped
/// entity such as `&amp;lt;` stays literal).
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(name: &str, attributes: &[(&str, &str)], self_closing: bool) -> XmlEvent {
        XmlEvent::Open {
            name: name.to_string(),
            attributes: attributes
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            self_closing,
        }
    }

    fn close(name: &str) -> XmlEvent {
        XmlEvent::Close {
            name: name.to_string(),
        }
    }

    #[test]
    fn scans_elements_attributes_and_text() {
        let events = scan(
            "<?xml version=\"1.0\"?>\n<Project a='1' b=\"x>y\">\n\
             <Import Project=\"p.props\" />\n<Name>A &amp; B</Name>\n</Project>",
        );
        assert_eq!(
            events,
            vec![
                open("Project", &[("a", "1"), ("b", "x>y")], false),
                open("Import", &[("Project", "p.props")], true),
                open("Name", &[], false),
                XmlEvent::Text("A & B".to_string()),
                close("Name"),
                close("Project"),
            ]
        );
    }

    #[test]
    fn skips_comments_and_keeps_cdata_verbatim() {
        let events = scan("<A><!-- <B>hidden</B> --><![CDATA[a&amp;<b>]]></A>");
        assert_eq!(
            events,
            vec![
                open("A", &[], false),
                XmlEvent::Text("a&amp;<b>".to_string()),
                close("A"),
            ]
        );
    }

    #[test]
    fn truncated_input_does_not_panic() {
        for document in ["<", "<A", "<A b=\"", "<!-- open", "<![CDATA[x", "</"] {
            let _ = scan(document);
        }
    }
}
