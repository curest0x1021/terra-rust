use std::io::{self, BufRead};
use structopt::StructOpt;
use terra_rust_api::{PrivateKey, Terra};

use crate::errors::Result;
use bitcoin::secp256k1::All;
use bitcoin::secp256k1::Secp256k1;

#[derive(StructOpt)]
pub enum KeysCommand {
    #[structopt(name = "parse", about = "parse a hex key into a terra account")]
    Parse {
        #[structopt(name = "hex")]
        /// hex public key to convert
        hex: String,
    },
    /// Create a new key to the wallet
    New {
        #[structopt(name = "name", help = "a memorable name to use in this client")]
        name: String,
    },
    /// Recover an existing key to the wallet using the recovery words
    Recover {
        #[structopt(name = "name", help = "a memorable name to use in this client")]
        name: String,
    },
    /// Delete a key from the wallet
    Delete {
        #[structopt(name = "name", help = "delete the private key with this name.")]
        name: String,
    },
    /// Get Public Key representation of the private key
    ///
    Get {
        #[structopt(name = "name", help = "the key with this name.")]
        name: String,
    },
    /// List keys in the wallet
    List,
}

macro_rules! key_format {
    () => {
        "TERRA-RUST-{}-{}"
    };
}
pub fn key_cmd_parse(
    _terra: &Terra,
    wallet: &str,
    seed: Option<&str>,
    key_cmd: KeysCommand,
) -> Result<()> {
    match key_cmd {
        KeysCommand::Parse { hex } => {
            println!("{}", hex)
        }
        KeysCommand::Recover { name } => {
            let secp = Secp256k1::new();
            let keyname = format!(key_format!(), wallet, name);

            println!("Please input the set of the recovery words, followed by the passphrase (which is passed via --seed)");
            if seed.is_some() {
                println!("Your Passphrase is {}", seed.unwrap());
            } else {
                println!("No Passphrase is being used");
            }
            println!("These will be stored in your computer's vault (win10)/secret service (linux)/keyring (os/x)");
            println!();
            println!("We take NO responsibility for the safety/security of this.");
            println!("This software is ALPHA and has not undergone a security audit");
            println!(
                "For high value keys, we suggest you always use a hardware wallet, like ledger"
            );
            println!();
            println!("Recovery words:");
            let stdin = io::stdin();
            let mut iterator = stdin.lock().lines();

            let words = iterator.next().unwrap().unwrap();

            let pk = match seed {
                Some(seed_str) => PrivateKey::from_words_seed(&secp, &words, seed_str)?,
                None => PrivateKey::from_words(&secp, &words)?,
            };
            let keyring = keyring::Keyring::new(&wallet, &keyname);
            keyring.set_password(pk.words().unwrap())?;
        }

        KeysCommand::New { name } => {
            let secp = Secp256k1::new();

            println!("This key will be stored in your computer's vault (win10)/secret service (linux)/keyring (os/x)");

            let pk = match seed {
                None => PrivateKey::new(&secp)?,
                Some(seed_str) => PrivateKey::new_seed(&secp, seed_str)?,
            };
            println!("Please write these down and save these in a secure location.");
            println!("These words can be used to transfer all your coins out of your account");
            println!("NO ONE has a need for these keys except you. If they are asking for them it is a scam.");
            println!();
            println!("Your recovery words are:");
            println!("{}", pk.words().unwrap());
            if seed.is_some() {
                println!("Please also take note of your seed phrase");
            }

            let keyname = format!(key_format!(), wallet, name);
            let keyring = keyring::Keyring::new(&wallet, &keyname);
            keyring.set_password(pk.words().unwrap())?;
        }
        KeysCommand::Delete { name } => {
            let keyname = format!(key_format!(), wallet, name);
            let keyring = keyring::Keyring::new(&wallet, &keyname);
            keyring.delete_password()?;
        }
        KeysCommand::Get { name } => {
            let secp = Secp256k1::new();
            let priv_key = get_private_key(&secp, wallet, &name, seed)?;
            let pub_key = priv_key.public_key(&secp);
            println!("{}", pub_key.account()?)
        }
        KeysCommand::List => {
            todo!()
        }
    }
    Ok(())
}

pub fn get_private_key(
    secp: &Secp256k1<All>,
    wallet: &str,
    name: &str,
    seed: Option<&str>,
) -> Result<PrivateKey> {
    let keyname = format!(key_format!(), wallet, name);
    let keyring = keyring::Keyring::new(&wallet, &keyname);
    let phrase = keyring.get_password()?;
    log::error!("{}", &phrase);
    match seed {
        None => Ok(PrivateKey::from_words(secp, &phrase)?),
        Some(seed_str) => Ok(PrivateKey::from_words_seed(secp, &phrase, seed_str)?),
    }
}
