use btc::{
    btc::{make_header_circuit, HeaderTarget, MultiHeaderTarget},
    l1::{compile_and_run_ln_circuit, compile_l1_circuit, run_l1_circuit},
};
use plonky2::{
    iop::witness::{PartialWitness, Witness},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputs,
    },
};

use crate::block::BlockHeader;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub struct ReusableProver {
    // data: CircuitData<F, C, D>,
    header_target: MultiHeaderTarget,
    layer_num: usize,
    last_circuit_data: CircuitData<F, C, D>,
    factors: Vec<usize>,
}

impl ReusableProver {
    pub fn new(layer_num: usize, FACTORS: &[usize]) -> Self {
        let (data, header_target) = compile_l1_circuit(FACTORS[0]).unwrap();
        let mut last_circuit_data = data;

        if layer_num > 1 {
            for i in 1..layer_num {
                let proof_res = compile_and_run_ln_circuit(
                    0, // UNUSED
                    Vec::new(),
                    &last_circuit_data.verifier_only,
                    &last_circuit_data.common,
                    FACTORS[i],
                    true,
                );
                let t = proof_res.unwrap();
                last_circuit_data = t.1;
            }
        }

        let mut vec_factors = Vec::new();
        for i in 0..FACTORS.len() {
            vec_factors.push(FACTORS[i].clone());
        }

        Self {
            header_target,
            layer_num,
            last_circuit_data: last_circuit_data,
            factors: vec_factors,
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
            &self.last_circuit_data,
            &self.header_target,
            header_hex_ref.as_slice(),
            headers.len(),
        )
        .unwrap();

        proof.to_bytes().unwrap()
    }

    pub fn prove_headers_layer(&self, proofs: Vec<Vec<u8>>) -> Vec<u8> {
        let (data, _) = compile_and_run_ln_circuit(
            0,
            proofs
                .into_iter()
                .map(|p| {
                    let t = ProofWithPublicInputs::from_bytes(p, &self.last_circuit_data.common)
                        .unwrap();
                    t
                })
                .collect(),
            &self.last_circuit_data.verifier_only,
            &self.last_circuit_data.common,
            self.factors[self.layer_num],
            false,
        )
        .unwrap();

        return data.unwrap().to_bytes().unwrap();
    }
}
