use ultralight_worker::block_fetcher::get_block_headers_for_range;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let headers = get_block_headers_for_range(0, 128).await?;
    let body = headers
        .iter()
        .map(|header| format!("\"{}\"", hex::encode(&header.0)))
        .collect::<Vec<String>>()
        .join(",\n");
    println!("{}", body);
    Ok(())
}
