# Kaspa Vanity Address Generator

Kaspa vanity address generator. Uses brute-force search on mnemonic combinations to find addresses matching specific prefixes or suffixes.

## Features

- **High Performance**: Parallel processing via Rayon.
- **Standard BIP-39**: 12 or 24 word mnemonics.
- **Flexible Matching**: Prefix, suffix, or both.

## Installation

```bash
git clone https://github.com/kryonkas/kas-vanity.git
cd kas-vanity
cargo build --release
```

## Usage

### Examples

```bash
# Search for prefix "test"
./target/release/kas-vanity --prefix test

# Search for suffix "2025"
./target/release/kas-vanity --suffix 2025

# Search for both
./target/release/kas-vanity --prefix test --suffix 2025

# 12-word mnemonic
./target/release/kas-vanity --prefix test --words 12

# Specific thread count
./target/release/kas-vanity --prefix test --threads 8

# Case sensitive
./target/release/kas-vanity --prefix Test --case-sensitive

# Scan limit
./target/release/kas-vanity --prefix test --scan-limit 50
```

### Parameters

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--prefix` | `-p` | Target prefix | None |
| `--suffix` | `-s` | Target suffix | None |
| `--words` | `-w` | Mnemonic length (12/24) | 24 |
| `--threads` | `-t` | Thread count | All cores |
| `--scan-limit` | | Addresses per mnemonic | 1 |
| `--case-sensitive` | | Case sensitivity | false |

**Note**: At least one of `--prefix` or `--suffix` is required.

### Output Example

**Success**:
```
Searching for prefix: "test"
Difficulty: 1 in 1048576 (approx)
Scan limit: 10 addresses per mnemonic
Using 10 threads...
Checked 0 addresses... (0.00% chance)
Checked 1000 addresses... (0.10% chance)
...
Checked 40000 addresses... (3.74% chance)

[MATCH FOUND]
Address: kaspa:qqtest...
Mnemonic: iron repair cabbage hood jewel deer title nest elder dance angry goose
Path Index: 3 (m/44'/111111'/0'/0/3)
Time taken: 4.20s
```

☕️ https://kas.coffee/kryon