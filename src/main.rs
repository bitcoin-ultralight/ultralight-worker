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
    let job_index: u64 = std::env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .unwrap()
        .parse()
        .unwrap();

    let job_id = get_job_id()?;
    let s3_prefix = format!("{}/{:0>8}/", job_id, job_index);
    let s3_pusher = S3Pusher::new(s3_prefix).await?;

    println!("Creating prover");
    let reusable_prover = ReusableProver::new();
    println!("Prover created");


    let nb_proofs = 10usize;
    let nb_blocks_per_proof = 10usize;
    let offset = (job_index as usize) * nb_proofs * nb_blocks_per_proof;

    for i in 0..nb_proofs {
        let start_block_idx = offset + i * nb_blocks_per_proof; 
        let end_block_idx = offset + (i+1) * (nb_blocks_per_proof);
        let ParentHashAndHeaders {
            parent_hash,
            headers,
        } = get_parent_hash_and_headers(start_block_idx as u64, end_block_idx as u64).await?;

        println!("Proving {}", start_block_idx);
        let proof = reusable_prover.prove_headers(headers.as_slice());
        println!("Proved {}", start_block_idx);
        // s3_pusher
        //     .push_bytes(
        //         &format!("{:0>10}-{:0>10}", start_block_idx, end_block_idx),
        //         proof,
        //     )
        //     .await?;
    }

    Ok(())
}
