use std::{env, process::exit, thread};
use sui_keys::key_derive::generate_new_key;
use sui_types::crypto::SignatureScheme;

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut args = args.into_iter();

    let mut handle_vec = vec![];

    let mut prefix = args
        .next()
        .unwrap_or_else(|| {
            panic!("should define a prefix!");
        })
        .to_owned();
    prefix.insert_str(0, "0x");

    let word_size = args.next().unwrap_or("24".to_string());

    for _i in 0..20 {
        let prefix = prefix.clone();
        let word_size = word_size.clone();

        let handle = thread::spawn(move || loop {
            let (sui_address, _, _, mnemonic) = generate_new_key(
                SignatureScheme::ED25519,
                None,
                Some(format!("word{}", word_size)),
            )
            .unwrap();

            if sui_address.to_string().starts_with(&prefix) {
                println!("Your sui address: {}", sui_address);
                println!("Your mnemonic: {}", mnemonic);
                exit(1);
            };
        });
        handle_vec.push(handle);
    }
    handle_vec
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}
