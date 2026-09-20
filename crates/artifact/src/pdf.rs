//! PDF container: the payload is a standard embedded-file attachment
//! (`/EmbeddedFiles` name tree -> `/Filespec` -> `/EmbeddedFile` stream).
//! No JavaScript, no actions, no launch: a viewer just shows a page of text.

use crate::{sha256_hex, Error, MAX_PAYLOAD};

const MARKER: &[u8] = b"/Type /EmbeddedFile";
const HASH_KEY: &[u8] = b"/IMC_SHA256 (";

fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}

pub fn wrap(payload: &[u8], title: &str) -> Result<Vec<u8>, Error> {
    if payload.len() > MAX_PAYLOAD {
        return Err(Error::TooLarge(payload.len()));
    }
    if payload.windows(MARKER.len()).any(|w| w == MARKER) {
        return Err(Error::Malformed("payload contains container marker".into()));
    }
    let content = format!("BT /F1 18 Tf 72 720 Td ({}) Tj ET", escape(title));
    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R /Names << /EmbeddedFiles << /Names [(program.imcp) 5 0 R] >> >> >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 7 0 R >> >> >>".to_vec(),
        format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content).into_bytes(),
        b"<< /Type /Filespec /F (program.imcp) /UF (program.imcp) /EF << /F 6 0 R >> >>".to_vec(),
    ];
    let mut emb = format!(
        "<< /Type /EmbeddedFile /Subtype /application#2Fjson /Length {n} /Params << /Size {n} /IMC_SHA256 ({h}) >> >>\nstream\n",
        n = payload.len(),
        h = sha256_hex(payload)
    )
    .into_bytes();
    emb.extend_from_slice(payload);
    emb.extend_from_slice(b"\nendstream");
    objects.push(emb);
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());

    let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    Ok(out)
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if from > hay.len() {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

pub fn unwrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let at = find(bytes, MARKER, 0).ok_or(Error::NotFound)?;
    if find(bytes, MARKER, at + 1).is_some() {
        return Err(Error::Multiple);
    }
    let stream =
        find(bytes, b"stream\n", at).ok_or_else(|| Error::Malformed("no stream".into()))?;
    let dict = &bytes[at..stream];

    let len_at = find(dict, b"/Length ", 0).ok_or_else(|| Error::Malformed("no /Length".into()))?
        + b"/Length ".len();
    let digits: Vec<u8> = dict[len_at..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .copied()
        .collect();
    let len: usize = std::str::from_utf8(&digits)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::Malformed("bad /Length".into()))?;
    if len > MAX_PAYLOAD {
        return Err(Error::TooLarge(len));
    }

    let h_at = find(dict, HASH_KEY, 0).ok_or_else(|| Error::Malformed("no checksum".into()))?
        + HASH_KEY.len();
    let want = dict
        .get(h_at..h_at + 64)
        .ok_or_else(|| Error::Malformed("short checksum".into()))?;

    let start = stream + b"stream\n".len();
    let end = start
        .checked_add(len)
        .filter(|e| *e <= bytes.len())
        .ok_or_else(|| Error::Malformed("stream exceeds file".into()))?;
    let payload = &bytes[start..end];
    if sha256_hex(payload).as_bytes() != want {
        return Err(Error::HashMismatch);
    }
    Ok(payload.to_vec())
}
