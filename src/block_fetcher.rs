use reqwest::{header::RANGE, StatusCode};

pub async fn get_block_headers_for_range(from_height: u64, to_height: u64) -> anyhow::Result<Vec<Vec<u8>>> {
    assert!(from_height <= to_height);

    let client = reqwest::Client::new();

    const HEADER_SIZE: u64 = 80;
    let range = format!(
        "bytes={}-{}",
        from_height * HEADER_SIZE,
        (to_height + 1) * HEADER_SIZE - 1
    );

    const URL: &str = "http://44.202.17.16/output.bin";
    let response = client.get(URL).header(RANGE, range).send().await?;

    //println!("{:?}", response);
    //let content_range_header = response.headers().get("content-range").unwrap().to_str().unwrap();

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        response.content_length(),
        Some(HEADER_SIZE * (to_height - from_height + 1))
    );

    let bytes = response.bytes().await?.to_vec();
    let chunks = bytes.chunks_exact(HEADER_SIZE as usize);
    assert_eq!(chunks.remainder().len(), 0);

    Ok(chunks.map(|chunk| chunk.to_vec()).collect::<Vec<Vec<u8>>>())
}
