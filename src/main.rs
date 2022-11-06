use std::time::Instant;

use anyhow::bail;
use clap::Parser;
use rayon::prelude::{ParallelIterator, IndexedParallelIterator};
use rayon::{prelude::IntoParallelIterator, ThreadPoolBuilder};
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

    let reusable_prover = ReusableProver::new();

    let header_pool = ThreadPoolBuilder::new().num_threads(4).build()?;

    let pool = ThreadPoolBuilder::new().num_threads(24).build()?;

    header_pool.scope(|_| {
        headers.into_par_iter().enumerate().for_each(|(idx, header)| {
            //let hash = header.compute_hash();
            let start = Instant::now();
            let bytes = pool.install(|| reusable_prover.prove_header(header));
            let elapsed = start.elapsed();
            println!("{} {} {}", from_height + idx as u64, bytes.len(), elapsed.as_micros());
            // TODO retry
            /*s3_pusher
                .push_bytes(&format!("{:0>10}-{}", block_height, hash.human()), bytes)
                .await?;*/
            //block_height += 1;
        });
    });
    //let mut block_height = from_height;

    Ok(())
}

fn get_block_range(args: &Args, job_index: u64) -> (u64, u64) {
    assert!(args.from_height <= args.to_height);
    assert!(job_index < args.total_jobs);

    let num_blocks = args.to_height - args.from_height + 1;
    let blocks_per_worker = (args.total_jobs + num_blocks - 1) / args.total_jobs;

    let start_height = args.from_height + blocks_per_worker * job_index;
    let end_height = std::cmp::min(start_height + blocks_per_worker - 1, args.to_height);

    (start_height, end_height)
}
