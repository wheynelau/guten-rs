# guten-rs

## Description

Small toy project to crawl, download and clean the Gutenberg project.

## Usage

### Use pre-built binaries

The binaries have been built with cargo-dist, so you can download them from the [tags page](https://github.com/wheynelau/guten-rs/tags)

However to get pass the openssl issues, the binaries are built with the `openssl-vendored` feature. 

You can remove crate and rebuild the binaries following the below instructions.  

### Step 0: Install Rust

1. Follow the instructions from the [Rust installation page](https://www.rust-lang.org/tools/install)

### Step 1: Clone the Repository

```bash
git clone https://github.com/username/project-name.git
cd project-name
```

### Step 2: Configure the Build (if needed)

Configure the toml file

### Step 3: Build and Install

```bash
cargo build --release
```

### Step 4: Run the Project

This project has multiple binaries, so you can run them with:

```bash
./target/release/crawl
./target/release/download
./target/release/process
```

## Personal learning points

- Finally ventured in async rust
- Multiple binaries
