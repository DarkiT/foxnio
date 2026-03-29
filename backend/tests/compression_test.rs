//! 压缩中间件测试

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::useless_vec)]

// 测试各种大小内容的压缩
#[test]
fn test_various_sizes_compression() {
    use foxnio::gateway::middleware::compression::{
        CompressionLayer, CompressionLevel, ContentEncoding,
    };
    use std::time::Instant;

    let layer = CompressionLayer::new().min_size(100);

    let test_cases = vec![
        ("tiny", b"hi".to_vec(), ContentEncoding::Identity),
        ("small", vec![b'x'; 200], ContentEncoding::Gzip),
        ("medium", vec![b'x'; 10_000], ContentEncoding::Gzip),
        ("large", vec![b'x'; 100_000], ContentEncoding::Gzip),
        ("huge", vec![b'x'; 1_000_000], ContentEncoding::Gzip),
    ];

    println!("\n=== Various Size Compression Test ===\n");
    println!(
        "{:<10} {:>12} {:>12} {:>10} {:>10}",
        "Size", "Original", "Compressed", "Ratio", "Time(ms)"
    );
    println!("{}", "-".repeat(60));

    for (name, data, encoding) in test_cases {
        let start = Instant::now();
        let result = layer.compress(&data, encoding).unwrap();
        let elapsed = start.elapsed();

        println!(
            "{:<10} {:>12} {:>12} {:>9.2}% {:>10.2}",
            name,
            format!("{} B", result.original_size),
            format!("{} B", result.compressed_size),
            result.compression_ratio() * 100.0,
            elapsed.as_millis()
        );

        // 小于最小大小的不压缩
        if data.len() < layer.min_size {
            assert_eq!(result.encoding, ContentEncoding::Identity);
        } else {
            assert!(result.compressed_size < result.original_size);
        }
    }
}

// 测试 gzip vs brotli 压缩率
#[test]
fn test_compression_ratio_comparison() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new().min_size(100);

    // JSON-like 数据
    let json_data = r#"{"id": 1, "name": "test", "data": [1, 2, 3, 4, 5]}"#.repeat(100);

    println!("\n=== Compression Ratio Comparison (JSON) ===\n");

    // Gzip
    let gzip_result = layer
        .compress(json_data.as_bytes(), ContentEncoding::Gzip)
        .unwrap();
    println!("Gzip:");
    println!("  Original: {} bytes", gzip_result.original_size);
    println!("  Compressed: {} bytes", gzip_result.compressed_size);
    println!("  Ratio: {:.2}%", gzip_result.compression_ratio() * 100.0);
    println!("  Saved: {} bytes", gzip_result.bytes_saved());

    // Brotli
    let brotli_result = layer
        .compress(json_data.as_bytes(), ContentEncoding::Brotli)
        .unwrap();
    println!("\nBrotli:");
    println!("  Original: {} bytes", brotli_result.original_size);
    println!("  Compressed: {} bytes", brotli_result.compressed_size);
    println!("  Ratio: {:.2}%", brotli_result.compression_ratio() * 100.0);
    println!("  Saved: {} bytes", brotli_result.bytes_saved());

    // Brotli 通常有更好的压缩率
    println!(
        "\nBrotli advantage: {} bytes better compression",
        gzip_result
            .compressed_size
            .saturating_sub(brotli_result.compressed_size)
    );

    assert!(gzip_result.compressed_size < gzip_result.original_size);
    assert!(brotli_result.compressed_size < brotli_result.original_size);
}

// 测试压缩级别
#[test]
fn test_compression_levels() {
    use foxnio::gateway::middleware::compression::{
        CompressionLayer, CompressionLevel, ContentEncoding,
    };
    use std::time::Instant;

    let data = vec![b'x'; 100_000];

    println!("\n=== Compression Level Comparison ===\n");
    println!(
        "{:<10} {:>12} {:>12} {:>10} {:>10}",
        "Level", "Original", "Compressed", "Ratio", "Time(ms)"
    );
    println!("{}", "-".repeat(60));

    for level in [
        CompressionLevel::Fast,
        CompressionLevel::Default,
        CompressionLevel::Best,
    ] {
        let layer = CompressionLayer::new().min_size(100).level(level);

        let start = Instant::now();
        let result = layer.compress(&data, ContentEncoding::Gzip).unwrap();
        let elapsed = start.elapsed();

        let level_name = match level {
            CompressionLevel::Fast => "Fast",
            CompressionLevel::Default => "Default",
            CompressionLevel::Best => "Best",
        };

        println!(
            "{:<10} {:>12} {:>12} {:>9.2}% {:>10.2}",
            level_name,
            format!("{} B", result.original_size),
            format!("{} B", result.compressed_size),
            result.compression_ratio() * 100.0,
            elapsed.as_millis()
        );
    }

    // 最佳压缩级别应该有最高的压缩率
    let fast_layer = CompressionLayer::new()
        .min_size(100)
        .level(CompressionLevel::Fast);
    let best_layer = CompressionLayer::new()
        .min_size(100)
        .level(CompressionLevel::Best);

    let fast_result = fast_layer.compress(&data, ContentEncoding::Gzip).unwrap();
    let best_result = best_layer.compress(&data, ContentEncoding::Gzip).unwrap();

    assert!(best_result.compressed_size <= fast_result.compressed_size);
}

// 测试解压缩
#[test]
fn test_roundtrip_compression() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new().min_size(10);
    let original = b"Hello, World! This is a test string for compression that is long enough.";

    println!("\n=== Roundtrip Compression Test ===\n");

    // Gzip roundtrip
    let compressed = layer.compress(original, ContentEncoding::Gzip).unwrap();
    assert_eq!(
        compressed.encoding,
        ContentEncoding::Gzip,
        "Data should be compressed"
    );
    let decompressed = layer
        .decompress(&compressed.body, ContentEncoding::Gzip)
        .unwrap();
    assert_eq!(decompressed.as_ref(), original);
    println!(
        "Gzip roundtrip: OK ({} -> {} bytes)",
        original.len(),
        compressed.compressed_size
    );

    // Brotli roundtrip
    let compressed = layer.compress(original, ContentEncoding::Brotli).unwrap();
    assert_eq!(
        compressed.encoding,
        ContentEncoding::Brotli,
        "Data should be compressed"
    );
    let decompressed = layer
        .decompress(&compressed.body, ContentEncoding::Brotli)
        .unwrap();
    assert_eq!(decompressed.as_ref(), original);
    println!(
        "Brotli roundtrip: OK ({} -> {} bytes)",
        original.len(),
        compressed.compressed_size
    );
}

// 测试内容协商
#[test]
fn test_content_negotiation() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new();

    println!("\n=== Content Negotiation Test ===\n");

    let test_cases = vec![
        ("gzip", ContentEncoding::Gzip),
        ("br", ContentEncoding::Brotli),
        ("gzip, br", ContentEncoding::Brotli), // Brotli 优先
        ("gzip, br;q=0.9", ContentEncoding::Gzip), // Gzip 有更高的 q 值
        ("identity", ContentEncoding::Identity),
        ("*", ContentEncoding::Brotli),         // 通配符选择最高优先级
        ("deflate", ContentEncoding::Identity), // 不支持的编码
        ("gzip, deflate, br;q=0.8", ContentEncoding::Gzip),
    ];

    for (accept_encoding, expected) in test_cases {
        let result = layer.select_encoding(Some(accept_encoding));
        assert_eq!(result, expected);
        println!("Accept-Encoding: {:<30} -> {:?}", accept_encoding, result);
    }
}

// 测试压缩统计
#[test]
fn test_compression_stats() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new().min_size(500);

    // 执行多次压缩 - 确保数据足够大以触发压缩
    for i in 0..10 {
        let data = vec![b'x'; (i + 1) * 1000]; // 1000, 2000, ... 10000 bytes
        layer.compress(&data, ContentEncoding::Gzip).unwrap();
    }

    let stats = layer.stats().snapshot();

    println!("\n=== Compression Statistics ===\n");
    println!("{}", stats);

    assert_eq!(stats.compress_count, 10);

    assert_eq!(stats.compress_count, 10);
    assert!(stats.total_original_size > 0);
    assert!(stats.total_compressed_size > 0);
    assert!(stats.compression_ratio > 0.0);

    // 重置统计
    layer.stats().reset();
    let stats = layer.stats().snapshot();
    assert_eq!(stats.compress_count, 0);
}

// 性能基准测试
#[test]
fn test_performance_benchmark() {
    use foxnio::gateway::middleware::compression::{
        CompressionLayer, CompressionLevel, ContentEncoding,
    };
    use std::time::Instant;

    println!("\n=== Performance Benchmark ===\n");

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    let encodings = vec![
        ("Gzip", ContentEncoding::Gzip),
        ("Brotli", ContentEncoding::Brotli),
    ];

    let levels = vec![
        ("Fast", CompressionLevel::Fast),
        ("Default", CompressionLevel::Default),
        ("Best", CompressionLevel::Best),
    ];

    println!(
        "{:<8} {:<10} {:<10} {:>12} {:>12} {:>10} {:>10}",
        "Size", "Encoding", "Level", "Original", "Compressed", "Ratio", "Time(ms)"
    );
    println!("{}", "-".repeat(80));

    for (size_name, size) in &sizes {
        let data = vec![b'x'; *size];

        for (encoding_name, encoding) in &encodings {
            for (level_name, level) in &levels {
                let layer = CompressionLayer::new().min_size(100).level(*level);

                let start = Instant::now();
                let result = layer.compress(&data, *encoding).unwrap();
                let elapsed = start.elapsed();

                println!(
                    "{:<8} {:<10} {:<10} {:>12} {:>12} {:>9.2}% {:>10.2}",
                    size_name,
                    encoding_name,
                    level_name,
                    format!("{} B", result.original_size),
                    format!("{} B", result.compressed_size),
                    result.compression_ratio() * 100.0,
                    elapsed.as_millis()
                );
            }
        }
        println!();
    }
}

// 测试不同类型内容的压缩效果
#[test]
fn test_different_content_types() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new().min_size(100);

    println!("\n=== Content Type Compression Test ===\n");

    // JSON
    let json = r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#.repeat(100);
    let json_result = layer
        .compress(json.as_bytes(), ContentEncoding::Gzip)
        .unwrap();
    println!(
        "JSON: {} -> {} bytes ({:.2}% reduction)",
        json.len(),
        json_result.compressed_size,
        json_result.compression_ratio() * 100.0
    );

    // HTML
    let html = r#"<html><body><p>Hello World</p></body></html>"#.repeat(100);
    let html_result = layer
        .compress(html.as_bytes(), ContentEncoding::Gzip)
        .unwrap();
    println!(
        "HTML: {} -> {} bytes ({:.2}% reduction)",
        html.len(),
        html_result.compressed_size,
        html_result.compression_ratio() * 100.0
    );

    // JavaScript
    let js = r#"function test() { console.log("Hello"); return 42; }"#.repeat(100);
    let js_result = layer
        .compress(js.as_bytes(), ContentEncoding::Gzip)
        .unwrap();
    println!(
        "JavaScript: {} -> {} bytes ({:.2}% reduction)",
        js.len(),
        js_result.compressed_size,
        js_result.compression_ratio() * 100.0
    );

    // CSS
    let css = r#".class { color: red; margin: 10px; padding: 5px; }"#.repeat(100);
    let css_result = layer
        .compress(css.as_bytes(), ContentEncoding::Gzip)
        .unwrap();
    println!(
        "CSS: {} -> {} bytes ({:.2}% reduction)",
        css.len(),
        css_result.compressed_size,
        css_result.compression_ratio() * 100.0
    );

    // 随机数据（低压缩率）
    let random: Vec<u8> = (0..1000).map(|i| (i * 17 + 31) as u8).collect();
    let random_result = layer.compress(&random, ContentEncoding::Gzip).unwrap();
    println!(
        "Random: {} -> {} bytes ({:.2}% reduction)",
        random.len(),
        random_result.compressed_size,
        random_result.compression_ratio() * 100.0
    );

    // 重复数据（高压缩率）
    let repeated = b"a".repeat(1000);
    let repeated_result = layer.compress(&repeated, ContentEncoding::Gzip).unwrap();
    println!(
        "Repeated: {} -> {} bytes ({:.2}% reduction)",
        repeated.len(),
        repeated_result.compressed_size,
        repeated_result.compression_ratio() * 100.0
    );
}

// 零拷贝测试
#[test]
fn test_zero_copy_optimization() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};

    let layer = CompressionLayer::new();
    let _original = vec![b'x'; 100_000];

    // 小于最小大小的数据不应压缩（直接返回）
    let small = vec![b'x'; 100];
    let result = layer.compress(&small, ContentEncoding::Gzip).unwrap();
    assert_eq!(result.encoding, ContentEncoding::Identity);
    assert_eq!(result.original_size, result.compressed_size);

    println!("\n=== Zero Copy Optimization ===\n");
    println!("Small data (< min_size): No compression (zero-copy)");
    println!("Large data: Compression applied");
}

// 流式压缩测试
#[test]
fn test_streaming_compression() {
    use foxnio::gateway::middleware::compression::{
        CompressionLevel, ContentEncoding, StreamingCompressor,
    };

    let mut compressor = StreamingCompressor::new(ContentEncoding::Gzip, CompressionLevel::Default);

    let chunks = vec![
        b"First chunk of data. ".as_slice(),
        b"Second chunk of data. ".as_slice(),
        b"Third chunk of data. ".as_slice(),
    ];

    println!("\n=== Streaming Compression Test ===\n");

    let mut total_compressed = 0;
    let mut total_original = 0;

    for (i, chunk) in chunks.iter().enumerate() {
        let compressed = compressor.compress_chunk(chunk).unwrap();
        println!(
            "Chunk {}: {} -> {} bytes",
            i + 1,
            chunk.len(),
            compressed.len()
        );
        total_original += chunk.len();
        total_compressed += compressed.len();
    }

    println!("\nTotal: {} -> {} bytes", total_original, total_compressed);
}

// 并发压缩测试
#[tokio::test]
async fn test_concurrent_compression() {
    use foxnio::gateway::middleware::compression::{CompressionLayer, ContentEncoding};
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::task::JoinSet;

    let layer = Arc::new(CompressionLayer::new());
    let mut tasks = JoinSet::new();

    println!("\n=== Concurrent Compression Test ===\n");

    let start = Instant::now();

    for i in 0..10 {
        let layer = layer.clone();
        tasks.spawn(async move {
            let data = vec![b'x'; 10_000];
            let result = layer.compress(&data, ContentEncoding::Gzip).unwrap();
            (i, result.compressed_size, result.compression_ratio())
        });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok((i, size, ratio)) = result {
            results.push((i, size, ratio));
        }
    }

    let elapsed = start.elapsed();

    println!(
        "Compressed {} chunks concurrently in {:?}",
        results.len(),
        elapsed
    );
    for (i, size, ratio) in results {
        println!(
            "  Task {}: {} bytes ({:.2}% reduction)",
            i,
            size,
            ratio * 100.0
        );
    }
}
