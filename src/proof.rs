use btc::btc::{make_header_circuit, HeaderTarget};
use plonky2::{
    iop::witness::{PartialWitness, Witness},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig}, proof::ProofWithPublicInputs,
    },
};

use crate::block::BlockHeader;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub struct ReusableProver {
    data: CircuitData<F, C, D>,
    header_target: HeaderTarget,
}

impl ReusableProver {
    pub fn new() -> Self {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
        let header_target = make_header_circuit(&mut builder);
        let data = builder.build::<C>();

        Self {
            data,
            header_target,
        }
    }

    pub fn prove_header(&self, header: BlockHeader) -> Vec<u8> {
        let header_bits = to_bits(header.0);

        let mut pw = PartialWitness::new();

        for i in 0..header_bits.len() {
            pw.set_bool_target(self.header_target.header_bits[i], header_bits[i]);
        }

        let (exp, mantissa) = compute_exp_and_mantissa(header_bits);

        println!("exp: {}, mantissa: {}", exp, mantissa);

        for i in 0..256 {
            if i < 256 - exp && mantissa & (1 << (255 - exp - i)) != 0 {
                pw.set_bool_target(self.header_target.threshold_bits[i as usize], true);
            } else {
                pw.set_bool_target(self.header_target.threshold_bits[i as usize], false);
            }
        }

        let proof = self.data.prove(pw).unwrap();
        let bytes = proof.to_bytes().unwrap();

        let deserialized_proof = ProofWithPublicInputs::from_bytes(bytes.clone(), &self.data.common).unwrap();
        self.data.verify(deserialized_proof).unwrap();

        bytes
    }
}

// TODO move to btc repo
fn compute_exp_and_mantissa(header_bits: Vec<bool>) -> (u32, u64) {
    let mut d = 0;
    for i in 600..608 {
        d += ((header_bits[i]) as u32) << (608 - i - 1);
    }
    let exp = 8 * (d - 3);
    let mut mantissa = 0;
    for i in 576..584 {
        mantissa += ((header_bits[i]) as u64) << (584 - i - 1);
    }
    for i in 584..592 {
        mantissa += ((header_bits[i]) as u64) << (592 - i - 1 + 8);
    }
    for i in 592..600 {
        mantissa += ((header_bits[i]) as u64) << (600 - i - 1 + 16);
    }

    (exp, mantissa)
}

fn to_bits(msg: Vec<u8>) -> Vec<bool> {
    let mut res = Vec::new();
    for i in 0..msg.len() {
        let char = msg[i];
        for j in 0..8 {
            if (char & (1 << 7 - j)) != 0 {
                res.push(true);
            } else {
                res.push(false);
            }
        }
    }
    res
}
