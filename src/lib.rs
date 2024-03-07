use ethers::types::H256;
use plonky2x::{
    frontend::hint::simple::hint::Hint,
    prelude::{
        plonky2::field::types::{Field64, PrimeField},
        *,
    },
    utils::hash::sha256,
};
use serde::{Deserialize, Serialize};

#[test]
fn test() {
    let mut cb = DefaultBuilder::new();

    let a = cb.read::<Variable>();
    let b = cb.read::<Variable>();
    let c = cb.read::<Variable>();

    let d = cb.add(a, b);
    let e = cb.mul(d, c);
    cb.write(e);

    let circuit = cb.build();

    let mut input = circuit.input();
    input.write::<Variable>(GoldilocksField::TWO);
    input.write::<Variable>(GoldilocksField::TWO);
    input.write::<Variable>(GoldilocksField::TWO.add_one());

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let sum = output.read::<Variable>();
    println!("{}", sum.0);
    for i in &proof.public_inputs {
        println!("public input: {}", i.0);
    }
}

#[test]
fn test_sqrt() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SqrtHint;

    impl<L: PlonkParameters<D>, const D: usize> Hint<L, D> for SqrtHint {
        fn hint(
            &self,
            input_stream: &mut ValueStream<L, D>,
            output_stream: &mut ValueStream<L, D>,
        ) {
            let value = input_stream.read_value::<Variable>();

            output_stream.write_value::<Variable>(value.sqrt().unwrap());
        }
    }

    let mut cb = DefaultBuilder::new();

    let value_squared = cb.read::<Variable>();

    let mut input_stream = VariableStream::new();
    input_stream.write(&value_squared);
    let output_stream = cb.hint(input_stream, SqrtHint);
    let answer = output_stream.read::<Variable>(&mut cb);
    let answer_squared = cb.mul(answer, answer);
    cb.assert_is_equal(answer_squared, value_squared);

    let circuit = cb.build();

    let mut input = circuit.input();
    input.write::<Variable>(GoldilocksField::from_canonical_u32(16));

    let (proof, output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    for i in &proof.public_inputs {
        println!("public input: {}", i.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HashHint {
    sequence_number: u128,
    secret_identifier: u128,
}

impl HashHint {
    pub fn hash(&self) -> [u8; 32] {
        let payload = [
            u128_to_32_be(self.sequence_number),
            u128_to_32_be(self.secret_identifier),
        ]
        .concat();
        sha256(&payload)
    }
}

impl<L: PlonkParameters<D>, const D: usize> Hint<L, D> for HashHint {
    fn hint(&self, _input_stream: &mut ValueStream<L, D>, output_stream: &mut ValueStream<L, D>) {
        output_stream.write_value::<Bytes32Variable>(u128_to_32_be(self.sequence_number).into());
        output_stream.write_value::<Bytes32Variable>(u128_to_32_be(self.secret_identifier).into());
    }
}

fn u128_to_32_be(u: u128) -> [u8; 32] {
    let mut v = [0u8; 32];
    v[16..32].copy_from_slice(&u.to_be_bytes());
    v
}

#[test]
fn calc_hash() {
    let h = HashHint {
        sequence_number: 123,
        secret_identifier: 456,
    };

    assert_eq!(
        H256::from(h.hash()),
        bytes32!("0xe03e1ee464b067e1fd0570cd3ca6829cf5041843ec151dffbc3b29340ee77045"),
    );
}

// 387.41s
#[test]
fn test_hash() {
    let h = HashHint {
        sequence_number: 123,
        secret_identifier: 456,
    };
    let mut builder = DefaultBuilder::new();
    let output_stream = builder.hint(VariableStream::new(), h.clone());
    let sequence_number = output_stream
        .read::<Bytes32Variable>(&mut builder);
    let secret_identifier = output_stream
        .read::<Bytes32Variable>(&mut builder);
    let actual_hash = builder
        .curta_sha256_pair(sequence_number, secret_identifier);
    let expected_hash = builder.read::<Bytes32Variable>();
    builder.assert_is_equal(actual_hash, expected_hash);

    let circuit = builder.build();

    let mut input = circuit.input();
    input.write::<Bytes32Variable>(H256::from(h.hash()));

    let (proof, output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);
}
