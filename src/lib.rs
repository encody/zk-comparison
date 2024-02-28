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
    sequence_number: u32,
    secret_identifier: Vec<u8>,
}

impl HashHint {
    pub fn hash(&self) -> [u8; 32] {
        let payload = [
            &self.sequence_number.to_be_bytes()[..],
            &self.secret_identifier[..],
        ]
        .concat();
        sha256(&payload)
    }
}

#[test]
fn calc_hash() {
    let h = HashHint {
        sequence_number: 123,
        secret_identifier: vec![22; 256],
    };

    println!("{:?}", h.hash());
}

#[test]
fn test_hash() {
    let preimage = HashHint {
        sequence_number: 123,
        secret_identifier: vec![22; 256],
    };

    impl<L: PlonkParameters<D>, const D: usize> Hint<L, D> for HashHint {
        fn hint(
            &self,
            _input_stream: &mut ValueStream<L, D>,
            output_stream: &mut ValueStream<L, D>,
        ) {
            output_stream.write_value::<Bytes32Variable>(self.hash().into());
        }
    }

    let mut builder = DefaultBuilder::new();

    let output_stream = builder.hint(VariableStream::new(), preimage);
    let calculated_hash = output_stream.read::<Bytes32Variable>(&mut builder);
    let input_hash = builder.read::<Bytes32Variable>();
    builder.assert_is_equal(input_hash, calculated_hash);

    let circuit = builder.build();

    let mut input = circuit.input();
    input.write::<Bytes32Variable>(
        [
            216, 107, 140, 71, 138, 50, 175, 97, 144, 26, 143, 64, 19, 118, 124, 228, 99, 71, 25,
            252, 217, 236, 133, 254, 140, 172, 180, 216, 110, 137, 3, 35,
        ]
        .into(),
    );

    let (proof, output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);
}
