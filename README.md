## Sui Vanity Tools

If you want to have an address with specified prefix that's in hexadecimal character, you can use this tool. It will bruteforce until it found the specified address. **The longer the specified characters are—the more the time needed.** Because the probability is becomes lower. Hence it's recommended to choose the character wisely.
  

## Quick Start

Make sure you have installed [Rust](https://www.rust-lang.org/tools/install) and then run `cargo run [prefix]`. e.g I want to get an address with prefix `0x123` then I will run `cargo run 123`. When the specified address found, it will show you the *address* and the *mnemonic*.

You could also specified the number of words in the mnemonic. There are 12, 15, 18, 21, and 24. To specify this number of words, you can run it as `cargo run [prefix] [number of word]`. e.g I want to get an address with prefix `0xabc` and have 12 words of mnemonic, then I will run `cargo run abc 12`. 
The number of word in the mnemonic is optional, the default number of words is 24.