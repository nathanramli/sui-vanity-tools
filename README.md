## Sui Vanity Tools

A high-performance vanity address generator for Sui blockchain. Generate wallet addresses with custom prefixes using optimized multi-threaded brute force search.

**Key Features:**
- ⚡ **Optimized Performance**: Uses all CPU cores by default
- 📊 **Real-time Progress**: Live statistics showing attempts/second
- 🎯 **Smart Configuration**: Automatic optimization with manual override options
- 🔧 **Flexible Options**: Custom word sizes, thread counts, and case-insensitive matching

**Performance Note**: The longer the prefix, the exponentially more time needed (each character = 16x harder). Choose wisely!

## Quick Start

### Installation

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed, then clone and build:

```bash
git clone <your-repo-url>
cd sui-vanity-tools
cargo build --release
```

### Basic Usage

**IMPORTANT**: Always use `--release` for 10-50x better performance!

```bash
# Find address starting with 0xabc (uses all CPU cores)
cargo run --release -- --prefix abc

# Find address ending with abc
cargo run --release -- --suffix abc

# Find address with both prefix and suffix
cargo run --release -- --prefix cafe --suffix beef

# With custom word size (12, 15, 18, 21, or 24)
cargo run --release -- --prefix abc --word-size 12

# Case-insensitive matching
cargo run --release -- --prefix ABC --case-insensitive

# Custom thread count
cargo run --release -- --prefix abc --threads 8
```

### Example Output

```
🔍 Sui Vanity Address Generator (Optimized)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Target:            Prefix: 0xabc
Word size:         24
Worker threads:    10
Batch size:        1000
Case insensitive:  false
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Estimated attempts needed: ~4,096 (1 in 4,096)
Starting search...

⚡ 50,000 attempts | 25,000/sec | elapsed: 2s | ETA: 1s

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ FOUND MATCHING ADDRESS!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Address:  0xabc1234567890abcdef...
Mnemonic: word1 word2 word3 ... word24
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total attempts: 3,847
```

## Options

```
Options:
  -p, --prefix <PREFIX>              Target prefix (without 0x)
  -s, --suffix <SUFFIX>              Target suffix (last characters of address)
  -w, --word-size <WORD_SIZE>        Mnemonic word count: 12, 15, 18, 21, or 24 [default: 24]
  -t, --threads <THREADS>            Number of worker threads [default: CPU cores]
  -c, --case-insensitive             Enable case-insensitive matching
  -b, --batch-size <BATCH_SIZE>      Keys to generate before checking status [default: 1000]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Performance Guide

### Expected Time by Prefix Length

| Prefix Length | Attempts Needed | Time (on modern CPU) |
|---------------|-----------------|----------------------|
| 1 char        | ~16             | Instant              |
| 2 chars       | ~256            | Instant              |
| 3 chars       | ~4,096          | Seconds              |
| 4 chars       | ~65,536         | Seconds to minutes   |
| 5 chars       | ~1,048,576      | Minutes to hours     |
| 6 chars       | ~16,777,216     | Hours to days        |
| 7+ chars      | ~268,435,456+   | Days to weeks        |

## Security

- Generated mnemonics are cryptographically secure
- Uses system RNG via official Sui SDK
- No shortcuts - every address is genuinely generated
- **Never share your mnemonic** - store it securely offline

## Contributing

Contributions welcome! Please open an issue or PR.