# 🚀 Miden MASM Rust Starter Template

This repository is a **starter template** for building and testing **Miden Assembly (MASM)** smart contracts with Rust.  
It comes with a simple **Calculator contract** and a script to **calculate value using public and private inputs**, plus Rust utilities to **deploy an account, deploy the contract, and interact with it**.

---

## ⚡ Features

- ✅ **MASM contract**: A simple `Calculator` contract.  
- ✅ **Calculate script**: Examples showing how to pass public and private inputs into Miden transactions and storing the result.
- ✅ **Rust client**: Utilities for deploying accounts and contracts.  
- ✅ **Async tests**: Tests using `tokio`.
- ✅ **Ready-to-extend**: Add your own MASM programs and Rust bindings easily.  

---
## 🛠️ Prerequisites

- [Rust](https://www.rust-lang.org/) (nightly recommended)
- [Miden](https://github.com/0xMiden) toolchain installed  
- Cargo for package management

---

## 🚀 Getting Started

### 1. Clone the repo
```bash
git clone https://github.com/novara-fi/masm-starter.git
cd masm-starter
```

### 2. Build the project
```bash
cargo build
```

### 3. Run deploy script
```bash
cargo run --release
```

### 4. Run tests
```bash
cargo test
```

# 📖 Resources

A collection of references to help you build with **Miden Assembly (MASM)**.

- [📚 Miden Book](https://0xmiden.github.io/miden-docs/index.html)  
  Official docs for the Miden VM, assembly language, and ecosystem.  

- [🛠️ Miden GitHub](https://github.com/0xMiden)  
  Source code, compiler, and tooling maintained by the Miden team.  

---