# HTTP Debug Proxy

A powerful HTTP debugging proxy server built in Rust that helps developers inspect, monitor, and debug HTTP/HTTPS traffic between clients and upstream servers.

## Features

- **Multiple Upstream Support**: Route requests to different backend servers based on URL prefixes
- **Full HTTP Method Support**: Handles GET, POST, PUT, PATCH, and DELETE methods
- **Comprehensive Request Logging**: 
  - Detailed logging of request/response headers
  - Automatic pretty-printing of JSON payloads
  - Binary and UTF-8 content support
  - Request timing information
- **Header Forwarding**: Automatically forwards client headers to upstream servers
- **Smart Content Handling**:
  - JSON auto-formatting for better readability
  - Support for binary data
  - UTF-8 string handling
- **Request Tracking**: Unique ID for each request for easy tracking
- **Flexible Configuration**: Support for multiple upstream servers with default fallback

## Installation

```bash
# Clone the repository
git clone https://github.com/DirectX/http-debug-proxy.git
cd http-debug-proxy

# Copy configs from references
cp config.example.yaml config.yaml
cp .example.env .env

# Replace config params with relevant upstream links

# Build the project
cargo build --release

# Run the proxy
cargo run --release
```

## Configuration

The proxy can be configured with multiple upstream servers. Example configuration:

```yaml
# config.yaml

server:
  host: 127.0.0.1
  port: 8001
upstreams:
  example_upstream: http://example.com
  another_upstream: http://another-example.com
default_upstream: example_upstream
```

## Usage

### Basic Request Routing

The proxy automatically routes requests based on the URL prefix:

```
# Routes to example_upstream
GET http://localhost:8081/example_upstream/api/users

# Same as above if it is the only upstream or `example_upstream` is set as default
GET http://localhost:8081/api/users
```

### Request Logging

For each request, the proxy logs:
- Unique request ID
- HTTP method and URL
- Request headers
- Request body (with JSON formatting if applicable)
- Response status code
- Response headers
- Response body (with JSON formatting if applicable)
- Request timing

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Actix-web](https://actix.rs/)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client functionality
- JSON handling powered by [serde_json](https://github.com/serde-rs/json)