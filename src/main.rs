#![feature(array_chunks)]

use anyhow::bail;
use clap::Parser;
use ultralight_worker::block_fetcher::ParentHashAndHeaders;

use ultralight_worker::{
    block_fetcher::get_parent_hash_and_headers, proof::ReusableProver, s3_pusher::S3Pusher,
};

#[derive(Parser, Debug)]
struct Args {
    // #[arg(long)]
    // job_index: u64,
    #[arg(long)]
    total_jobs: u64,

    #[arg(long)]
    from_height: u64,
    #[arg(long)]
    to_height: u64,
}

fn get_job_id() -> anyhow::Result<String> {
    let job_id = std::env::var("AWS_BATCH_JOB_ID").unwrap();
    let split: Vec<&str> = job_id.split(':').collect();
    if split.len() != 2 {
        bail!("Bad job id length");
    }

    Ok(split[0].to_owned())
}
const CHUNK_SIZE: usize = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let job_index: u64 = std::env::var("AWS_BATCH_JOB_ARRAY_INDEX")
        .unwrap()
        .parse()
        .unwrap();

    let job_id = get_job_id()?;
    let s3_prefix = format!("{}/{:0>8}/", job_id, job_index);
    let s3_pusher = S3Pusher::new(s3_prefix).await?;

    let (from_height, to_height) = get_block_range(&args, job_index);

    println!("{} {}", from_height, to_height);

    let ParentHashAndHeaders {
        parent_hash,
        headers,
    } = get_parent_hash_and_headers(from_height, to_height).await?;

    println!("Creating prover");
    let reusable_prover = ReusableProver::new();
    println!("Prover created");

    let chunks = headers.as_slice().array_chunks::<CHUNK_SIZE>();
    assert!(chunks.remainder().is_empty());

    let mut chunk_start_height = from_height;
    for chunk in chunks {
        println!("Proving {}", chunk_start_height);
        let proof = reusable_prover.prove_headers(chunk);
        println!("Proved {}", chunk_start_height);
        let chunk_end_height = chunk_start_height + CHUNK_SIZE as u64 - 1;
        s3_pusher
            .push_bytes(
                &format!("{:0>10}-{:0>10}", chunk_start_height, chunk_end_height),
                proof,
            )
            .await?;
        chunk_start_height += CHUNK_SIZE as u64;
    }

    Ok(())
}

fn get_block_range(args: &Args, job_index: u64) -> (u64, u64) {
    assert!(args.from_height <= args.to_height);
    let num_blocks = args.to_height - args.from_height + 1;
    assert_eq!(num_blocks % CHUNK_SIZE as u64, 0);
    assert_eq!(num_blocks % args.total_jobs, 0);
    assert!(job_index < args.total_jobs);

    let blocks_per_worker = num_blocks / args.total_jobs;

    let start_height = args.from_height + blocks_per_worker * job_index;
    let end_height = std::cmp::min(start_height + blocks_per_worker - 1, args.to_height);

    (start_height, end_height)
}
