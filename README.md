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

### For GitHub Codespaces (Recommended)

**⚠️ Important**: The pre-built binary will not work in GitHub Codespaces due to architecture differences. You must build from source.

```bash
# Clone the repository (if not already cloned)
git clone https://github.com/asp2131/rusty-scv.git
cd rusty-scv

# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build the application
cargo build --release

# Copy to local bin directory
mkdir -p ~/.local/bin
cp target/release/scv ~/.local/bin/

# Add to PATH (add this to your ~/.bashrc for persistence)
export PATH="$HOME/.local/bin:$PATH"

# Run the application
scv
```

### For Local Systems

#### Option 1: Download the pre-built binary

```bash
# Download the release
curl -L https://github.com/asp2131/rusty-scv/releases/download/1.0.1/scv -o scv

# Make it executable
chmod +x scv

# Move to a directory in your PATH
sudo mv scv /usr/local/bin/
```

#### Option 2: Build from source

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- Cargo (comes with Rust)

```bash
# Clone the repository
git clone https://github.com/asp2131/rusty-scv.git
cd rusty-scv

# Build the application
cargo build --release

# Copy to local bin directory
mkdir -p ~/.local/bin
cp target/release/scv ~/.local/bin/

# Add to PATH (add this to your ~/.bashrc for persistence)
export PATH="$HOME/.local/bin:$PATH"

# Run the application
scv
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

## Troubleshooting

### "Exec format error" when running the binary

This error occurs when trying to run a binary compiled for a different architecture. This is common in:
- GitHub Codespaces (uses Linux x86_64, but the binary might be compiled for a different target)
- Docker containers with different architectures
- Cross-platform usage (e.g., trying to run a macOS binary on Linux)

**Solution**: Build from source using the instructions above for your specific environment.

### Rust not found

If you get "cargo: command not found" or "rustc: command not found":

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Permission denied when moving to /usr/local/bin

If you don't have sudo access (common in some environments):

```bash
# Use local bin directory instead
mkdir -p ~/.local/bin
cp target/release/scv ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"

# Make the PATH change permanent
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

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
