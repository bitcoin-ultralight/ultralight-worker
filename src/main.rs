#![feature(array_chunks)]

use anyhow::bail;
use ultralight_worker::block_fetcher::ParentHashAndHeaders;

use ultralight_worker::{
    block_fetcher::get_parent_hash_and_headers, proof::ReusableProver, s3_pusher::S3Pusher,
};

fn get_job_id() -> anyhow::Result<String> {
    let job_id = std::env::var("AWS_BATCH_JOB_ID").unwrap();
    let split: Vec<&str> = job_id.split(':').collect();
    if split.len() != 2 {
        bail!("Bad job id length");
    }

    Ok(split[0].to_owned())
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let job_index: usize = std::env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .unwrap()
        .parse()
        .unwrap();

    let args: Vec<String> = std::env::args().collect();
    let layer_num: usize = args[1].parse::<usize>().unwrap();
    let bucket_name: usize = args[2].parse::<usize>().unwrap(); // This will be a specific number, like 6969

    let S3_ENABLED: bool = true;
    let FACTORS = [3, 9, 3];
    let B = [11, 1, 1];
    // B Implicitly defines M
    // N_i is defined by N_{i-1} and F_{i-1}
    let N_0 = 100;

    let job_id = get_job_id()?;
    let s3_prefix = format!("{}/", bucket_name);
    let s3_pusher = S3Pusher::new(s3_prefix).await?;

    println!("Creating prover");
    let reusable_prover = ReusableProver::new(layer_num, &FACTORS);
    println!("Prover created");

    if (layer_num > 0) {
        // We read in proofs from AWS

        // Your total start and end block will be

        // let baby_proof_step = FACTORS[layer_num - 1];
        // let final_step = FACTORS[layer_num] * baby_proof_step;
        // let start_block = job_index * final_step;
        let nb_proofs = B[layer_num];
        let child_proofs_per_proof = FACTORS[layer_num];
        let offset = (job_index as usize) * nb_proofs * (child_proofs_per_proof - 1);
        for i in 0..nb_proofs {
            let start_proof_idx = offset + i * (child_proofs_per_proof - 1);
            let end_proof_idx = offset + (i + 1) * (child_proofs_per_proof - 1);

            // Get the child proofs
            let mut child_proofs = Vec::new();
            for i in start_proof_idx..end_proof_idx {
                let proof = s3_pusher
                    .pull_bytes(&format!("{:0>10}-{:0>10}", layer_num - 1, i))
                    .await?;
                child_proofs.push(proof);
            }
            println!(
                "job_id={} job_index={} start_proof_idx={} end_proof_idx={}",
                job_id, job_index, start_proof_idx, end_proof_idx,
            );
            println!("Proving {}", start_proof_idx);
            let proof = reusable_prover.prove_headers_layer(child_proofs);
            println!("Proved {}", start_proof_idx);
            if (S3_ENABLED) {
                s3_pusher
                    .push_bytes(
                        &format!("{:0>10}-{:0>10}", layer_num, job_index * nb_proofs + i),
                        proof,
                    )
                    .await?;
            }
        }
    } else {
        // This is the base case with the leafs
        // Number of jobs we launch for layer zero = total number of leaves / (FACTOR[0] * nb_proofs)
        let nb_proofs = B[0]; // number of proofs per machine so that we parallelize the layer across the machines
        let nb_blocks_per_proof = FACTORS[0];
        let offset = (job_index as usize) * nb_proofs * (nb_blocks_per_proof - 1);

        for i in 0..nb_proofs {
            let start_block_idx = offset + i * (nb_blocks_per_proof - 1);
            let end_block_idx = offset + (i + 1) * (nb_blocks_per_proof - 1);
            let ParentHashAndHeaders {
                parent_hash,
                headers,
            } = get_parent_hash_and_headers(start_block_idx as u64, end_block_idx as u64).await?;

            println!(
                "job_id={} job_index={} start_block_idx={} end_block_idx={}",
                job_id, job_index, start_block_idx, end_block_idx,
            );
            println!("Proving {}", start_block_idx);
            let proof = reusable_prover.prove_headers(headers.as_slice());
            println!("Proved {}", start_block_idx);
            if (S3_ENABLED) {
                s3_pusher
                    .push_bytes(
                        &format!("{:0>10}-{:0>10}", layer_num, job_index * nb_proofs + i),
                        proof,
                    )
                    .await?;
            }
        }
    }

    Ok(())
}
