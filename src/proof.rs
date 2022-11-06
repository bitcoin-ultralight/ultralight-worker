use btc::{
    btc::{make_header_circuit, HeaderTarget, MultiHeaderTarget},
    l1::{compile_l1_circuit, run_l1_circuit},
};
use plonky2::{
    iop::witness::{PartialWitness, Witness},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputs,
    },
};

use crate::block::BlockHeader;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub struct ReusableProver {
    data: CircuitData<F, C, D>,
    header_target: MultiHeaderTarget,
}

impl ReusableProver {
    pub fn new() -> Self {
        let (data, header_target) = compile_l1_circuit(11).unwrap();

        Self {
            data,
            header_target,
        }
    }

    pub fn prove_headers(&self, headers: &[BlockHeader]) -> Vec<u8> {
        let header_hexs = headers
            .iter()
            .map(|header| hex::encode(&header.0))
            .collect::<Vec<String>>();
        let header_hex_ref = header_hexs
            .iter()
            .map(|hex| hex.as_str())
            .collect::<Vec<&str>>();

        let proof = run_l1_circuit(
            &self.data,
            &self.header_target,
            header_hex_ref.as_slice(),
            headers.len(),
        )
        .unwrap();

        proof.to_bytes().unwrap()
    }
}
