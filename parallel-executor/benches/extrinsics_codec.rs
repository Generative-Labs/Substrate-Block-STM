/// Benchmark for Substrate extrinsic codec.
///
/// In the parallel execution approach, a batch of transactions is passed from the block
/// builder to the runtime API, and then transmitted to the parallel executor. This process
/// involves one encoding and one decoding operation. Hence, considering the impact of such
/// encoding and decoding on performance is crucial.
///
/// Currently, we have only conducted encoding and decoding performance tests for simple
/// transfer transactions. The observed time consumption at the transaction boundary is very
/// short, which might be due to the simplicity of transfer transactions—quite small, around
/// 130 bytes in size.
///
/// In the future, we may need to conduct encoding and decoding checks for complex
/// transactions to ensure that transaction encoding and decoding do not become performance
/// bottlenecks.
use criterion::{criterion_group, criterion_main, Criterion};
use sp_keyring::AccountKeyring;
use substrate_test_runtime_client::runtime::{Extrinsic, Transfer};

fn benchmark_extrinsic_codec(c: &mut Criterion) {
    let mut transfer_extrinsics = Vec::new();
    for _i in 0..100000 {
        transfer_extrinsics.push(
            Transfer { from: AccountKeyring::Alice.into(), to: AccountKeyring::Bob.into(), amount: 0, nonce: 0 }
                .into_unchecked_extrinsic(),
        )
    }

    c.bench_function("benchmark extrinsic encode", |b| {
        b.iter(|| {
            let _ = sp_core::Encode::encode(&transfer_extrinsics);
        })
    });

    let encode_extrinsic = sp_core::Encode::encode(&transfer_extrinsics);
    println!("size {}", encode_extrinsic.len());

    c.bench_function("benchmark extrinsic decode", |b| {
        b.iter(|| {
            let extrinsics_decode: Result<Vec<Extrinsic>, _> = sp_core::Decode::decode(&mut &encode_extrinsic[..]);
            assert!(matches!(extrinsics_decode, Ok(_)));
        })
    });
}

criterion_group!(benches, benchmark_extrinsic_codec);
criterion_main!(benches);
