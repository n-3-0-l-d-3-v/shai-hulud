//! SVG container: the payload is base64 text inside a namespaced
//! `<metadata>` element, which renderers ignore. No `<script>`, no handlers.

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{sha256_hex, Error, MAX_PAYLOAD};

const OPEN: &str = "<imc:program ";
const CLOSE: &str = "</imc:program>";

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            c if c.is_control() => "?".to_string(),
            c => c.to_string(),
        })
        .collect()
}

pub fn wrap(payload: &[u8], title: &str) -> Result<Vec<u8>, Error> {
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    let t = xml_escape(title);
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:imc=\"urn:impossible-computer:artifact:1\" width=\"480\" height=\"120\" viewBox=\"0 0 480 120\">\n<title>{t}</title>\n<text x=\"20\" y=\"60\" font-family=\"sans-serif\" font-size=\"18\">{t}</text>\n<metadata>{OPEN}encoding=\"base64\" sha256=\"{}\">{}{CLOSE}</metadata>\n</svg>\n",
        sha256_hex(payload),
        STANDARD.encode(payload)
    )
    .into_bytes())
}

pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Malformed("not utf-8".into()))?;
    let at = text.find(OPEN).ok_or(Error::NotFound)?;
    if text[at + 1..].contains(OPEN) {
        return Err(Error::Multiple);
    }
    let rest = &text[at + OPEN.len()..];
    let gt = rest
        .find('>')
        .ok_or_else(|| Error::Malformed("unterminated tag".into()))?;
    let attrs = &rest[..gt];
    let body_end = rest
        .find(CLOSE)
        .ok_or_else(|| Error::Malformed("no close tag".into()))?;
    if body_end < gt {
        return Err(Error::Malformed("bad element".into()));
    }
    let body = &rest[gt + 1..body_end];

    let want = attrs
        .split("sha256=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .ok_or_else(|| Error::Malformed("no sha256 attribute".into()))?;
    if !attrs.contains("encoding=\"base64\"") {
        return Err(Error::Malformed("unsupported encoding".into()));
    }
    if body.len() > MAX_PAYLOAD * 4 / 3 + 4 {
        return Err(Error::TooLarge(body.len()));
    }
    let payload = STANDARD
        .decode(body.trim())
        .map_err(|e| Error::Malformed(format!("base64: {e}")))?;
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    if sha256_hex(&payload) != want {
        return Err(Error::HashMismatch);
    }
    Ok(payload)
}
