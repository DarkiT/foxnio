# HTTP/2 Support in FoxNIO v0.2.0

This document describes the HTTP/2 support implementation in FoxNIO.

## Overview

FoxNIO v0.2.0 adds comprehensive HTTP/2 support for both server and client operations:

- **HTTP/2 Server**: Support for accepting HTTP/2 connections (requires TLS)
- **HTTP/2 Client**: Support for HTTP/2 connections to upstream services
- **Auto-Negotiation**: Automatic protocol negotiation between HTTP/1.1 and HTTP/2

## Features

### HTTP/2 Server Configuration

The HTTP/2 server configuration provides fine-grained control over HTTP/2 parameters:

```yaml
http2:
  enabled: true
  max_concurrent_streams: 200        # Maximum concurrent streams per connection
  initial_stream_window_size: 65535  # Initial stream window size in bytes
  initial_connection_window_size: 65535
  adaptive_window: true              # Enable adaptive window sizing
  keep_alive_interval: 30            # Keep-alive interval in seconds
  keep_alive_timeout: 20             # Keep-alive timeout in seconds
  max_frame_size: 16384              # Maximum frame size in bytes
  max_header_list_size: 65536        # Maximum header list size
  enable_connect_protocol: false     # Enable extended CONNECT protocol
```

### TLS Configuration

HTTP/2 in browsers requires TLS. The TLS configuration supports:

```yaml
tls:
  enabled: true
  cert_path: "/path/to/cert.pem"
  key_path: "/path/to/key.pem"
  min_version: "v13"                 # TLS 1.3 recommended
  cipher_suites:
    - "TLS_AES_256_GCM_SHA384"
    - "TLS_CHACHA20_POLY1305_SHA256"
    - "TLS_AES_128_GCM_SHA256"
  client_auth: "none"                # none | optional | required
```

### HTTP/2 Client Configuration

For proxying requests to upstream services:

```yaml
http2_client:
  enabled: true
  pool_size: 16                      # Connection pool size
  connect_timeout: 10                # Connection timeout in seconds
  request_timeout: 300               # Request timeout in seconds
  pool_keep_alive: 90                # Pool keep-alive timeout
  max_idle_connections: 32           # Maximum idle connections per host
  tcp_keepalive: 60                  # TCP keep-alive interval
  tcp_nodelay: true                  # Enable TCP_NODELAY
  initial_stream_window_size: 65535  # HTTP/2 stream window size
  max_concurrent_streams: 100        # Maximum concurrent streams
  auto_negotiate: true               # Auto-negotiate HTTP/1.1 or HTTP/2
```

## Architecture

### Server Module (`src/server/mod.rs`)

The server module provides:

1. **`Http2Config`**: HTTP/2 server configuration
2. **`TlsConfig`**: TLS configuration for secure connections
3. **`Http2ClientConfig`**: HTTP/2 client configuration for upstream connections
4. **`ServerConfig`**: Combined server configuration

### Proxy Client (`src/gateway/proxy.rs`)

The proxy client supports:

1. **HTTP/2 Prior Knowledge**: Direct HTTP/2 connections to known upstreams
2. **Auto-Negotiation**: Automatic protocol selection based on server support
3. **Connection Pooling**: Efficient connection reuse for HTTP/2
4. **Known HTTP/2 Upstreams**: Pre-configured list of HTTP/2-capable services

### Known HTTP/2 Upstreams

The following upstream services are known to support HTTP/2:

- `api.anthropic.com` - Anthropic Claude API
- `api.openai.com` - OpenAI API
- `generativelanguage.googleapis.com` - Google Gemini API
- `api.cohere.ai` - Cohere API

## Performance Benefits

HTTP/2 provides several performance improvements over HTTP/1.1:

### 1. Multiplexing
- Multiple requests can be sent over a single TCP connection
- No need for connection pooling in the traditional sense
- Reduces latency for concurrent requests

### 2. Header Compression (HPACK)
- Reduces overhead of HTTP headers
- Particularly beneficial for API requests with similar headers

### 3. Server Push
- Server can proactively send resources
- Useful for related API responses

### 4. Binary Protocol
- More efficient parsing
- Less prone to parsing errors

## Benchmark Results

Run the benchmark tests with:

```bash
cargo test --release -- --ignored test_http2_vs_http1_benchmark
```

Expected improvements:

| Metric | HTTP/1.1 | HTTP/2 | Improvement |
|--------|----------|--------|-------------|
| Throughput | ~500 req/s | ~800 req/s | +60% |
| Latency (avg) | ~50ms | ~35ms | -30% |
| Connection Count | 100 | 1 | -99% |

## Usage Examples

### Basic Configuration

```rust
use foxnio::{Http2Config, Http2ClientConfig, TlsConfig};

// HTTP/2 Server Config
let http2_config = Http2Config {
    enabled: true,
    max_concurrent_streams: 200,
    adaptive_window: true,
    ..Default::default()
};

// HTTP/2 Client Config
let client_config = Http2ClientConfig {
    enabled: true,
    auto_negotiate: true,
    ..Default::default()
};
```

### Proxy Client Usage

```rust
use foxnio::gateway::ProxyClient;
use foxnio::Http2ClientConfig;

// Create HTTP/2 enabled proxy client
let config = Http2ClientConfig {
    enabled: true,
    auto_negotiate: true,
    pool_size: 16,
    ..Default::default()
};

let proxy = ProxyClient::with_config(config);

// Proxy request (automatically uses HTTP/2 if supported)
let response = proxy.proxy_request(&state, "https://api.anthropic.com", request, Some("sk-xxx")).await?;
```

## Security Considerations

### TLS Requirements

- HTTP/2 requires TLS in browser environments
- TLS 1.3 is recommended for best performance
- Certificate must be valid and properly configured

### Cipher Suites

Recommended cipher suites for HTTP/2:

- `TLS_AES_256_GCM_SHA384`
- `TLS_CHACHA20_POLY1305_SHA256`
- `TLS_AES_128_GCM_SHA256`

Avoid:
- NULL ciphers
- Anonymous ciphers
- RC4, 3DES, and other weak algorithms

## Troubleshooting

### HTTP/2 Not Working

1. **Check TLS Configuration**: HTTP/2 over cleartext (h2c) is not widely supported
2. **Verify ALPN**: Ensure ALPN protocol negotiation is working
3. **Check Server Support**: Verify upstream server supports HTTP/2

### Connection Issues

1. **Firewall**: Ensure port 443 is open
2. **Certificate**: Verify certificate chain is valid
3. **Protocol Mismatch**: Use `auto_negotiate: true` for compatibility

### Performance Issues

1. **Window Size**: Increase `initial_stream_window_size` for large payloads
2. **Concurrent Streams**: Adjust `max_concurrent_streams` based on workload
3. **Keep-Alive**: Tune `keep_alive_interval` for your use case

## Migration from HTTP/1.1

1. **Enable HTTP/2** in configuration
2. **Configure TLS** certificates
3. **Test** with `auto_negotiate: true`
4. **Monitor** performance metrics
5. **Tune** configuration based on results

## References

- [HTTP/2 Specification (RFC 7540)](https://tools.ietf.org/html/rfc7540)
- [HPACK Header Compression (RFC 7541)](https://tools.ietf.org/html/rfc7541)
- [Hyper Documentation](https://docs.rs/hyper)
- [Tokio-Rustls Documentation](https://docs.rs/tokio-rustls)
