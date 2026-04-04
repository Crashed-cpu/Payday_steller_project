<div align="center">

# 💸 PaydayAdvance

### On-chain Earned-Wage Access · Powered by Stellar Soroban

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Soroban SDK](https://img.shields.io/badge/Soroban_SDK-20.0.0-blueviolet)](https://soroban.stellar.org)
[![Network](https://img.shields.io/badge/Network-Stellar_Testnet-green)](https://stellar.expert/explorer/testnet)

</div>

---

## 📖 Project Description

**PaydayAdvance** is a trustless, transparent earned-wage access (EWA) protocol built as a Soroban smart contract on the Stellar blockchain.

Traditional payday-advance services charge employees predatory fees and rely on opaque, centralised intermediaries. PaydayAdvance replaces that model with a self-custodial, on-chain escrow where:

- **Employers** lock payroll tokens into the contract as wages are earned.
- **Employees** can withdraw any portion of their earned (but not yet paid) wages at any time, paying only a flat, employer-defined fee.
- **Settlement** at payday pushes the remaining balance directly to the employee.

No banks. No apps. No waiting. Just code.

---

## ⚙️ What It Does

```
Employer funds escrow                Employee requests advance
─────────────────────                ──────────────────────────
  Employer Wallet                       Employee Wallet
       │                                     ▲
       │  register_employee(                 │  request_advance(
       │    earned_wages = 2000 USDC,        │    amount = 300 USDC
       │    fee_per_advance = 5 USDC         │  )
       │  )                                  │
       ▼                                     │
  ┌──────────────────────────────────────────┴──────┐
  │            PaydayAdvance Contract               │
  │  ┌──────────────────────────────────────────┐   │
  │  │  Employee Record                         │   │
  │  │  earned_wages:    2 000 USDC             │   │
  │  │  advanced_amount:   305 USDC  ◄── grows  │   │
  │  │  fee_per_advance:     5 USDC             │   │
  │  └──────────────────────────────────────────┘   │
  └─────────────────────────────────────────────────┘
       │
       │  settle_payroll()   ← called by employer on payday
       │  sends remaining 1 695 USDC to employee
       ▼
  Employee Wallet  (+1 695 USDC)
```

1. **Employer** calls `register_employee` → locks earned wages into contract escrow.
2. **Employee** calls `request_advance` → receives net amount instantly; fee stays in contract.
3. **Employer** calls `settle_payroll` on payday → remaining wages sent to employee, record reset.
4. **Admin** calls `collect_fees` → sweeps accumulated flat fees to treasury.

---

## ✨ Features

| Feature | Details |
|---|---|
| 🔐 **Role-based auth** | `employer`, `employee`, and `admin` roles enforced via `require_auth()` |
| 💰 **Flexible fee model** | Per-employee flat fee configured by the employer at registration |
| 📊 **Transparent balances** | `get_employee` & `available_to_advance` views on-chain at all times |
| 🔄 **Pay-period reset** | `settle_payroll` clears advances and re-opens the next cycle |
| 🛡️ **Overflow protection** | Rust's checked arithmetic + Soroban `i128` prevents integer exploits |
| 📢 **Event emission** | `register`, `advance`, `settle` events for off-chain indexing |
| ⚡ **Any Stellar token** | Works with USDC, EURC, or any SEP-41 / SAC-compatible asset |
| 🧪 **Unit-tested** | Full test suite with happy-path and panic cases |

---

## 🗂️ Project Structure

```
payday-advance/
├── Cargo.toml                          # Workspace manifest
├── deploy.sh                           # Build & deploy helper script
├── README.md
└── contracts/
    └── payday_advance/
        ├── Cargo.toml
        └── src/
            └── lib.rs                  # Contract source + tests
```

---

## 🚀 Quick Start

### 1 · Prerequisites

```bash
# Install Rust
curl https://sh.rustup.rs -sSf | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli
```

### 2 · Build

```bash
cargo build \
  --manifest-path contracts/payday_advance/Cargo.toml \
  --target wasm32-unknown-unknown \
  --release
```

### 3 · Run Tests

```bash
cargo test \
  --manifest-path contracts/payday_advance/Cargo.toml \
  --features testutils
```

### 4 · Deploy to Testnet

```bash
# Fund a testnet account first (Friendbot)
curl "https://friendbot.stellar.org?addr=YOUR_PUBLIC_KEY"

# Deploy
ADMIN_SECRET=YOUR_SECRET_KEY bash deploy.sh
```

The script prints the Contract ID and a direct Stellar Expert explorer link.

---

## 📡 Contract Interface

### Initialisation
```
initialize(admin: Address, token: Address)
```

### Employer
```
register_employee(employer, employee, earned_wages, fee_per_advance)
settle_payroll(employer, employee)
set_employee_status(employer, employee, active)
```

### Employee
```
request_advance(employee, amount)
```

### Admin
```
collect_fees(to: Address)
transfer_admin(new_admin: Address)
```

### View (read-only)
```
get_employee(employee)        → Employee
available_to_advance(employee) → i128
get_admin()                   → Address
get_token()                   → Address
```

---

## 🔗 Deployed Smart Contract

| Network | Contract ID |
|---|---|
| **Stellar Testnet** | `CCHTFW3WHYOFIXHCLV2P2Y6GDRD7IESNZWBWQL5IETJKPNEP4A2T64JS` |
| **Stellar Mainnet** | _Not yet deployed_ |

> 🔍 View Contract on Stellar Expert:  
> https://stellar.expert/explorer/testnet/contract/CCHTFW3WHYOFIXHCLV2P2Y6GDRD7IESNZWBWQL5IETJKPNEP4A2T64JS

### 👤 Admin / Deployer Wallet

| Role | Address |
|---|---|
| **Admin Wallet** | `GAE3XJ5Y5XFPRG2ZXR4RDOIY7B7RUAMFROFYWQL37FRU5PDYUAXKXU63` |

> 🔍 View Wallet on Stellar Expert:  
> https://stellar.expert/explorer/testnet/account/GAE3XJ5Y5XFPRG2ZXR4RDOIY7B7RUAMFROFYWQL37FRU5PDYUAXKXU63

---

## 🗺️ Roadmap

- [ ] Multi-advance per pay period (already supported, UI pending)
- [ ] Employer dashboard (React + Soroban JS SDK)
- [ ] Percentage-based fee option alongside flat fee
- [ ] Payroll oracle integration for automatic wage updates
- [ ] Mainnet deployment + audit

---

## 🤝 Contributing

Pull requests are welcome! Please open an issue first to discuss significant changes.

```bash
git clone https://github.com/your-org/payday-advance.git
cd payday-advance
cargo test --features testutils
```

---

## 📜 License

MIT © 2024 Your Name
