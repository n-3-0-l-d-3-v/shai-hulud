use artifact::{detect, embed, extract, pdf, run, svg, Error, Format};
use proptest::prelude::*;

const SUM: &str = r#"
    block entry:
        loadi r0, 0
        loadi r1, 1
        loadi r2, 5
        jmp loop_check
    block loop_check:
        cmpgt r3, r1, r2
        jnz r3, after_loop
    block loop_body:
        add r0, r0, r1
        loadi r4, 1
        add r1, r1, r4
        jmp loop_check
    block after_loop:
        call print_result
    block done:
        halt
    block print_result:
        syscall 0
        ret
"#;

fn program() -> isa::Program {
    asm::assemble(SUM).unwrap()
}

#[test]
fn both_formats_roundtrip_and_run_identically() {
    let native = run(program(), 1 << 20, 1_000_000);
    for fmt in [Format::Pdf, Format::Svg] {
        let bytes = embed(fmt, &program(), "Sum (1..5)").unwrap();
        assert_eq!(detect(&bytes), Some(fmt));
        let report = run(extract(&bytes).unwrap(), 1 << 20, 1_000_000);
        assert_eq!(report.output, vec!["15".to_string()]);
        assert_eq!(report.output, native.output);
    }
}

#[test]
fn pdf_is_structurally_valid_xref_and_inert() {
    let bytes = embed(Format::Pdf, &program(), "T (with) \\ parens").unwrap();
    let text = bytes
        .iter()
        .map(|&b| if b < 0x80 { b as char } else { '?' })
        .collect::<String>();
    for bad in ["/JavaScript", "/JS ", "/Launch", "/OpenAction", "/AA "] {
        assert!(!text.contains(bad), "active content key {bad}");
    }
    let sx = text.rfind("startxref\n").unwrap() + "startxref\n".len();
    let xref: usize = text[sx..].lines().next().unwrap().parse().unwrap();
    assert!(text[xref..].starts_with("xref\n"));
    for (i, line) in text[xref..]
        .lines()
        .skip(3)
        .take_while(|l| l.ends_with(" n "))
        .enumerate()
    {
        let off: usize = line[..10].parse().unwrap();
        assert!(
            text[off..].starts_with(&format!("{} 0 obj", i + 1)),
            "obj {}",
            i + 1
        );
    }
}

#[test]
fn failure_modes_fail_closed() {
    assert_eq!(extract(b"hello").unwrap_err(), Error::UnknownFormat);
    assert_eq!(extract(b"%PDF-1.7\n%%EOF").unwrap_err(), Error::NotFound);
    let mut pdf = embed(Format::Pdf, &program(), "t").unwrap();
    let pos = pdf.windows(2).position(|w| w == b"{\"").unwrap();
    pdf[pos + 2] ^= 1; // corrupt payload
    assert_eq!(extract(&pdf).unwrap_err(), Error::HashMismatch);
    let svg_bytes = embed(Format::Svg, &program(), "t").unwrap();
    let doubled = [svg_bytes.clone(), svg_bytes].concat();
    assert_eq!(extract(&doubled).unwrap_err(), Error::Multiple);
    let huge = vec![0u8; artifact::MAX_PAYLOAD + 1];
    assert!(matches!(pdf::wrap(&huge, "t"), Err(Error::TooLarge(_))));
}

#[test]
fn step_budget_bounds_a_hostile_program() {
    let looping = asm::assemble("block a:\n  jmp a\n").unwrap();
    let bytes = embed(Format::Pdf, &looping, "loop").unwrap();
    let report = run(extract(&bytes).unwrap(), 1 << 20, 1000);
    assert!(report.exit.is_err(), "infinite loop must trap on budget");
}

proptest! {
    /// Invariant: for ANY single-byte corruption anywhere in a container the
    /// loader returns an error or the *identical* payload — never different bytes.
    #[test]
    fn pdf_corruption_never_yields_different_payload(
        payload in proptest::collection::vec(any::<u8>(), 0..300),
        idx in any::<prop::sample::Index>(),
        flip in 1u8..=255,
    ) {
        let Ok(file) = pdf::wrap(&payload, "t") else { return Ok(()) };
        let mut bad = file.clone();
        let i = idx.index(bad.len());
        bad[i] ^= flip;
        if let Ok(got) = pdf::unwrap(&bad) {
            prop_assert_eq!(got, payload);
        }
    }

    #[test]
    fn svg_corruption_never_yields_different_payload(
        payload in proptest::collection::vec(any::<u8>(), 0..300),
        idx in any::<prop::sample::Index>(),
        flip in 1u8..=255,
    ) {
        let file = svg::wrap(&payload, "t").unwrap();
        let mut bad = file.clone();
        let i = idx.index(bad.len());
        bad[i] ^= flip;
        if let Ok(got) = svg::unwrap(&bad) {
            prop_assert_eq!(got, payload);
        }
    }

    #[test]
    fn pdf_and_svg_roundtrip_arbitrary_bytes(payload in proptest::collection::vec(any::<u8>(), 0..2000)) {
        if let Ok(f) = pdf::wrap(&payload, "t") {
            prop_assert_eq!(pdf::unwrap(&f).unwrap(), payload.clone());
        }
        prop_assert_eq!(svg::unwrap(&svg::wrap(&payload, "t").unwrap()).unwrap(), payload);
    }
}
