//! Synthetic protobuf fixtures prove the bounded tensor codec through its public surface.
use biomcp_adapters::alphagenome::TensorStream;
use biomcp_fetch::grpc::{CodecError, Limits};

fn chunk(data: &[u8], compression: u8) -> Vec<u8> {
    let mut bytes = vec![10];
    let mut length = data.len();
    loop {
        bytes.push((length & 127) as u8 | if length > 127 { 128 } else { 0 });
        length >>= 7;
        if length == 0 {
            break;
        }
    }
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(&[16, compression]);
    bytes
}

#[test]
fn stream_tensors_checks_count_order_shape_and_expansion() {
    // feature: stream-tensors
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    // Tensor(shape=[2], dtype=float32, chunk_count=2).
    assert!(
        stream
            .descriptor(&[10, 1, 2, 16, 2, 32, 2])
            .unwrap()
            .is_none()
    );
    assert!(
        stream
            .chunk(&chunk(&1f32.to_le_bytes(), 0))
            .unwrap()
            .is_none()
    );
    let tensor = stream
        .chunk(&chunk(&2f32.to_le_bytes(), 0))
        .unwrap()
        .unwrap();
    assert_eq!(tensor.shape(), &[2]);
    assert_eq!(
        tensor.bytes(),
        [1f32.to_le_bytes(), 2f32.to_le_bytes()].concat()
    );
    assert!(stream.finish().is_ok());
}

#[test]
fn stream_tensors_refuses_malformed_and_out_of_order_payloads() {
    // feature: stream-tensors
    for descriptor in [
        vec![],
        vec![10, 1, 2, 16, 99, 32, 1],
        vec![10, 1, 2, 16, 2, 32, 0],
        vec![10, 1, 2, 16, 2, 40, 1],
    ] {
        let mut stream = TensorStream::new(Limits::default()).unwrap();
        assert_eq!(
            stream.descriptor(&descriptor).unwrap_err(),
            CodecError::BadReply
        );
    }
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    assert_eq!(
        stream.chunk(&chunk(&[], 0)).unwrap_err(),
        CodecError::BadReply
    );
    assert_eq!(stream.finish().unwrap_err(), CodecError::BadReply);
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    stream.descriptor(&[10, 1, 2, 16, 2, 32, 2]).unwrap();
    assert_eq!(stream.finish().unwrap_err(), CodecError::BadReply);
}

#[test]
fn stream_tensors_refuses_wire_and_shape_limits_before_decode() {
    // feature: stream-tensors
    let limits = Limits {
        message_bytes: 6,
        ..Limits::default()
    };
    let mut stream = TensorStream::new(limits).unwrap();
    assert_eq!(
        stream.descriptor(&[10, 1, 2, 16, 2, 32, 2]).unwrap_err(),
        CodecError::BadReply
    );
    let limits = Limits {
        expanded_bytes: 4,
        ..Limits::default()
    };
    let mut stream = TensorStream::new(limits).unwrap();
    assert_eq!(
        stream.descriptor(&[10, 1, 2, 16, 2, 32, 2]).unwrap_err(),
        CodecError::BadReply
    );
}

#[test]
fn stream_tensors_caps_zstd_output_and_window() {
    // feature: stream-tensors
    let compressed = vec![40, 181, 47, 253, 0, 72, 3, 32, 0, 0];
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
    assert_eq!(
        stream.chunk(&chunk(&compressed, 1)).unwrap_err(),
        CodecError::BadReply
    );
    let data = 7f32.to_le_bytes();
    let compressed = vec![40, 181, 47, 253, 0, 72, 33, 0, 0, 0, 0, 224, 64];
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
    assert_eq!(
        stream
            .chunk(&chunk(&compressed, 1))
            .unwrap()
            .unwrap()
            .bytes(),
        data
    );
    assert!(stream.finish().is_ok());
    // The same valid frame must fail when its declared decoding window exceeds policy.
    let limits = Limits {
        zstd_window_log: 10,
        ..Limits::default()
    };
    let mut stream = TensorStream::new(limits).unwrap();
    stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
    assert_eq!(
        stream.chunk(&chunk(&compressed, 1)).unwrap_err(),
        CodecError::BadReply
    );
}

#[test]
fn stream_tensors_enforces_each_allocation_boundary() {
    // feature: stream-tensors
    let fixtures = [
        (
            Limits {
                rank: 1,
                ..Limits::default()
            },
            vec![10, 2, 1, 1, 16, 2, 32, 1],
        ),
        (
            Limits {
                dimension: 1,
                ..Limits::default()
            },
            vec![10, 1, 2, 16, 2, 32, 1],
        ),
        (
            Limits {
                chunks: 1,
                ..Limits::default()
            },
            vec![10, 1, 2, 16, 2, 32, 2],
        ),
    ];
    for (limits, bytes) in fixtures {
        assert_eq!(
            TensorStream::new(limits)
                .unwrap()
                .descriptor(&bytes)
                .unwrap_err(),
            CodecError::BadReply
        );
    }
    for limits in [
        Limits {
            wire_bytes: 8,
            ..Limits::default()
        },
        Limits {
            messages: 1,
            ..Limits::default()
        },
    ] {
        let mut stream = TensorStream::new(limits).unwrap();
        stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
        assert_eq!(
            stream.chunk(&chunk(&1f32.to_le_bytes(), 0)).unwrap_err(),
            CodecError::BadReply
        );
    }
    assert_eq!(
        TensorStream::new(Limits {
            rank: 0,
            ..Limits::default()
        })
        .unwrap_err(),
        CodecError::Limits
    );
    let limits = Limits {
        request_bytes: 1,
        ..Limits::default()
    };
    assert!(limits.check_request(&[0]).is_ok());
    assert_eq!(limits.check_request(&[0, 0]), Err(CodecError::BadReply));
}

#[test]
fn stream_tensors_checks_inline_scalars_and_per_stream_totals() {
    // feature: stream-tensors
    let limits = Limits {
        expanded_bytes: 4,
        ..Limits::default()
    };
    let mut stream = TensorStream::new(limits).unwrap();
    // Empty shape is a scalar. Tensor inline chunk contains exactly one float32.
    let mut bytes = vec![16, 2, 26, 8];
    bytes.extend_from_slice(&chunk(&3f32.to_le_bytes(), 0));
    let tensor = stream.descriptor(&bytes).unwrap().unwrap();
    assert_eq!(tensor.shape(), &[] as &[i32]);
    assert_eq!(tensor.data_type().width(), 4);
    assert_eq!(tensor.data_type().number(), 2);
    assert_eq!(stream.descriptor(&bytes).unwrap_err(), CodecError::BadReply);
}

#[test]
fn stream_tensors_refuses_duplicate_unknown_truncated_and_interleaved_messages() {
    // feature: stream-tensors
    for bytes in [
        vec![16, 2, 32, 1, 16, 2],
        vec![16, 2, 32, 1, 80, 1],
        vec![10, 8, 1],
        vec![16, 2, 32, 128],
    ] {
        let mut stream = TensorStream::new(Limits::default()).unwrap();
        assert_eq!(stream.descriptor(&bytes).unwrap_err(), CodecError::BadReply);
    }
    let mut stream = TensorStream::new(Limits::default()).unwrap();
    stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
    assert_eq!(
        stream.descriptor(&[16, 2, 32, 1]).unwrap_err(),
        CodecError::BadReply
    );
    for bytes in [chunk(&[], 99), vec![10, 1], vec![24, 1], chunk(&[], 0)] {
        let mut stream = TensorStream::new(Limits::default()).unwrap();
        stream.descriptor(&[10, 1, 1, 16, 2, 32, 1]).unwrap();
        assert_eq!(stream.chunk(&bytes).unwrap_err(), CodecError::BadReply);
    }
}

#[test]
fn stream_tensors_checks_all_scalar_widths_and_shape_overflow() {
    // feature: stream-tensors
    for (dtype, width) in [
        (1, 2),
        (2, 4),
        (3, 8),
        (4, 1),
        (5, 4),
        (6, 8),
        (7, 1),
        (8, 4),
        (9, 8),
        (10, 1),
        (11, 2),
    ] {
        let mut stream = TensorStream::new(Limits::default()).unwrap();
        stream.descriptor(&[10, 1, 1, 16, dtype, 32, 1]).unwrap();
        let tensor = stream.chunk(&chunk(&vec![0; width], 0)).unwrap().unwrap();
        assert_eq!(tensor.data_type().width(), width);
        assert!(stream.finish().is_ok());
    }
    let mut bytes = vec![10, 40];
    for _ in 0..8 {
        bytes.extend_from_slice(&[255, 255, 255, 255, 7]);
    }
    bytes.extend_from_slice(&[16, 3, 32, 1]);
    let limits = Limits {
        dimension: i32::MAX as usize,
        ..Limits::default()
    };
    let mut stream = TensorStream::new(limits).unwrap();
    assert_eq!(stream.descriptor(&bytes).unwrap_err(), CodecError::BadReply);
}
