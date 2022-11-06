use reqwest::{header::RANGE, StatusCode};

use crate::block::{BlockHash, BlockHeader};

pub async fn get_block_headers_for_range(
    from_height: u64,
    to_height: u64,
) -> anyhow::Result<Vec<BlockHeader>> {
    assert!(from_height <= to_height);

    let client = reqwest::Client::new();

    const HEADER_SIZE: u64 = 80;
    let range = format!(
        "bytes={}-{}",
        from_height * HEADER_SIZE,
        (to_height + 1) * HEADER_SIZE - 1
    );

    const URL: &str = "http://44.202.17.16/output.bin";

    let mut i = 0;
    let response = loop {
        let request = client.get(URL).header(RANGE, &range).send().await;
        if request.is_ok() || i >= 5 {
            break request;
        } else {
            println!("Retrying due to error {:?}", request);
        }
        i += 1;
    }?;

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

    Ok(chunks
        .map(|chunk| BlockHeader(chunk.to_vec()))
        .collect::<Vec<BlockHeader>>())
}

pub struct ParentHashAndHeaders {
    pub parent_hash: BlockHash,
    pub headers: Vec<BlockHeader>,
}

pub async fn get_parent_hash_and_headers(
    from_height: u64,
    to_height: u64,
) -> anyhow::Result<ParentHashAndHeaders> {
    let fetch_from_height = if from_height > 0 { from_height - 1 } else { 0 };

    let mut headers = get_block_headers_for_range(fetch_from_height, to_height).await?;

    let parent_hash = if from_height > 0 {
        let parent_header = headers.remove(0);
        parent_header.compute_hash()
    } else {
        BlockHash(vec![0u8; 32])
    };

    Ok(ParentHashAndHeaders {
        parent_hash,
        headers,
    })
}
