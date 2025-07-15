# Rusty SCV - Student Code Viewer

A terminal-based application for managing student repositories and tracking GitHub activity, built with Rust.

![Rusty SCV Screenshot](docs/images/screenshot.png)

## Features

- Manage student classes and repositories
- Track GitHub activity and commit history
- View latest student activity across repositories
- Terminal-based UI with keyboard navigation
- Efficient data management and caching

## Installation

### Option 1: Download the pre-built binary

```bash
# Download the latest release
curl -L https://github.com/asp2131/rusty-scv/releases/latest/download/scv-macos -o scv

# Make it executable
chmod +x scv

# Move to a directory in your PATH (optional)
sudo mv scv /usr/local/bin/
```

### Option 2: Build from source

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- Cargo (comes with Rust)

```bash
# Clone the repository
git clone https://github.com/asp2131/rusty-scv.git
cd rusty-scv

# Build the application
cargo build --release

# The binary will be available at target/release/scv
```

## Development

### Building for development

```bash
# Build in debug mode
cargo build

# Run the debug build
./target/debug/scv
```

### Running tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test <test_name>
```

### Cleaning the project

```bash
# Remove all build artifacts
cargo clean
```

### Running in development mode

```bash
# Run directly with cargo
cargo run

# Or build and run manually
cargo build
./target/debug/scv
```

## Usage

- Navigate menus using arrow keys or vim-style keys (j/k)
- Select options with Enter
- Go back with Esc or q
- Refresh data with r
- See on-screen help for additional commands

## Configuration

The application looks for a configuration file at `~/.config/scv/config.toml`. You can set up GitHub tokens and other preferences there.

Example configuration:
```toml
[github]
token = "your_github_token_here"

[database]
path = "~/.local/share/scv/database.sqlite"
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
