//! Undo the two encodings some sources leave in their values. Each caller names a
//! source known to carry the encoding; elsewhere a backslash or `&` is plain text.
use std::borrow::Cow;

/// Decode HTML character references in one pass, so `&amp;lt;` reads `&lt;`.
///
/// `&nbsp;` becomes an ordinary space with the same meaning. An unknown or invalid
/// reference stays as written, because no character is guessed.
pub(crate) fn decode_entities(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after)) = rest.split_once('&') {
        out.push_str(before);
        let decoded = after
            .split_once(';')
            .and_then(|(name, tail)| Some((entity(name)?, tail)));
        if let Some((decoded, tail)) = decoded {
            out.push(decoded);
            rest = tail;
        } else {
            out.push('&');
            rest = after;
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

fn entity(name: &str) -> Option<char> {
    match name {
        "nbsp" => Some(' '),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => numeric(name),
    }
}

/// A numeric reference; an unparsable one returns None, so its text stays as written.
fn numeric(name: &str) -> Option<char> {
    let (digits, radix) = match name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
        Some(hex) => (hex, 16),
        None => (name.strip_prefix('#')?, 10),
    };
    // A reference is digits alone, with no sign; one too large for a code point stays as written.
    if digits.is_empty() {
        return None;
    }
    let code = digits.chars().try_fold(0u32, |code, ch| {
        code.checked_mul(radix)?.checked_add(ch.to_digit(radix)?)
    })?;
    char::from_u32(code).filter(|ch| *ch != '\0')
}

/// Remove Markdown backslash escapes: a backslash before ASCII punctuation only.
///
/// Any other backslash is literal text, as in Markdown itself.
pub(crate) fn unescape_markdown(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match chars.peek() {
            Some(next) if ch == '\\' && next.is_ascii_punctuation() => {
                out.push(*next);
                chars.next();
            }
            _ => out.push(ch),
        }
    }
    Cow::Owned(out)
}
