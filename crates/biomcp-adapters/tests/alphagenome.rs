//! Feature alphagenome-protocol: one documented request and complete bounded DNASE envelopes.
use biomcp_adapters::alphagenome::DnaseRequest;
use biomcp_fetch::grpc::{Limits, StreamDecoder};
type Check = Result<(), Box<dyn std::error::Error + Send + Sync>>;
fn request() -> Result<DnaseRequest, biomcp_fetch::InputError> {
    DnaseRequest::human_lung_all_folds(
        biodata::Build::GRCh38,
        "chr7",
        140745143..140761527,
        140753336,
        "A",
        "T",
    )
}
fn varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value > 127 {
        bytes.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}
fn number(tag: u8, value: u64) -> Vec<u8> {
    let mut bytes = vec![tag << 3];
    bytes.extend(varint(value));
    bytes
}
fn field(tag: u8, data: &[u8]) -> Vec<u8> {
    let mut bytes = vec![(tag << 3) | 2];
    bytes.extend(varint(data.len() as u64));
    bytes.extend(data);
    bytes
}
fn interval() -> Vec<u8> {
    [
        field(1, b"chr7"),
        number(2, 140745143),
        number(3, 140761527),
        number(4, 3),
    ]
    .concat()
}
fn output(allele: u8, tracks: usize) -> Vec<u8> {
    // A synthetic [2,1] UINT8 tensor in one chunk; no provider shape is assumed.
    let tensor = [field(1, &[2, 1]), number(2, 7), number(4, 1)].concat();
    let mut track = field(1, &tensor);
    for _ in 0..tracks {
        track.extend(field(2, &field(1, b"synthetic label")));
    }
    track.extend(number(3, 1));
    track.extend(field(4, &interval()));
    field(allele, &[number(1, 3), field(2, &track)].concat())
}
fn chunk() -> Vec<u8> {
    field(3, &field(1, &[1, 2]))
}
#[test]
fn canonical_request_preserves_one_based_variant_and_zero_based_interval() -> Check {
    let variant = [
        field(1, b"chr7"),
        number(2, 140753336),
        field(3, b"A"),
        field(4, b"T"),
    ]
    .concat();
    let expected = [
        field(1, &interval()),
        field(2, &variant),
        number(3, 9606),
        field(4, &[3]),
        field(5, &[number(1, 2), number(2, 2048)].concat()),
        field(6, b"ALL_FOLDS"),
    ]
    .concat();
    assert_eq!(request()?.bytes(), expected);
    assert_eq!(request()?.build(), biodata::Build::GRCh38);
    assert!(
        DnaseRequest::human_lung_all_folds(
            biodata::Build::GRCh37,
            "chr7",
            140745143..140761527,
            140753336,
            "A",
            "T"
        )
        .is_err()
    );
    for (interval, position, reference) in [
        (140745143..140761526, 140753336, "A"),
        (140745143..140761527, 0, "A"),
        (140745143..140761527, 140753336, "?"),
    ] {
        assert!(
            DnaseRequest::human_lung_all_folds(
                biodata::Build::GRCh38,
                "chr7",
                interval,
                position,
                reference,
                "T"
            )
            .is_err()
        );
    }
    Ok(())
}
#[test]
fn envelopes_require_both_alleles_exact_chunks_and_metadata_shape() -> Check {
    let req = request()?;
    let mut complete = req
        .decoder(Limits::default())
        .map_err(|_| "decoder refused limits")?;
    for msg in [output(2, 1), chunk(), output(1, 1), chunk()] {
        complete
            .message(&msg)
            .map_err(|_| "complete envelope rejected")?;
    }
    complete.finish().map_err(|_| "complete stream rejected")?;
    for messages in [
        vec![],
        vec![chunk()],
        vec![output(1, 1)],
        vec![output(1, 1), chunk()],
        vec![output(1, 1), output(2, 1)],
        vec![output(1, 1), chunk(), output(1, 1), chunk()],
        vec![output(1, 2), chunk(), output(2, 1), chunk()],
        vec![output(1, 1), chunk(), output(2, 1), chunk(), chunk()],
        vec![vec![10, 255]],
    ] {
        let mut decoder = req
            .decoder(Limits::default())
            .map_err(|_| "decoder refused limits")?;
        let valid = messages
            .iter()
            .try_for_each(|message| decoder.message(message))
            .is_ok();
        assert!(!valid || decoder.finish().is_err());
    }
    let mut bounded = req
        .decoder(Limits {
            expanded_bytes: 3,
            ..Default::default()
        })
        .map_err(|_| "decoder refused bounded limits")?;
    bounded
        .message(&output(1, 1))
        .map_err(|_| "reference envelope rejected")?;
    bounded
        .message(&chunk())
        .map_err(|_| "reference chunk rejected")?;
    assert!(bounded.message(&output(2, 1)).is_err());
    Ok(())
}
