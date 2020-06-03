use anyhow::Result;
use clap::{Arg, App};
use log::warn;
use regex::{Captures, Regex};
use std::path::Path;
use std::fs;
use tokio::runtime::Runtime;
use rusoto_core::Region;
use rusoto_secretsmanager::{GetSecretValueRequest, SecretsManager, SecretsManagerClient};

fn get_secret(s: &str) -> Result<String> {
    let client = SecretsManagerClient::new(Region::UsEast1);
    let request = GetSecretValueRequest {
        secret_id: s.to_string(),
        ..Default::default()
    };
    let result = async {
        client.get_secret_value(request).await
    };
    let secret = Runtime::new()?.block_on(result)?.secret_string;
    let v: serde_json::Value = serde_json::from_str(&secret.unwrap())?;
    Ok(v[s].as_str().unwrap().to_string())
}


fn run(path: &Path) -> Result<()> {
    assert!(path.exists(), "Given path does not exist");
    let contents = fs::read_to_string(path)?;
    let re = Regex::new(r"<<(?P<s>.*?)>>").unwrap();
    for line in contents.lines() {
        if re.is_match(line) {
            let all_secrets_applied = re.replace_all(line, |cap: &Captures| {
                let secret = get_secret(&cap[1]);
                match secret {
                    Ok(s) => s,
                    Err(_) => {warn!("Secret {} not found.", &cap[1]); format!("<<{}>>", cap[1].to_string())}
            }
        });
            println!("{}", all_secrets_applied)
        } else {
            println!("{}", line)
        }
    };
    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("Secret Populator")
        .version("1.0")
        .author("Sebastian Rollen")
        .about("Populates secrets within a .tpl template file")
        .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .help("Path to template file")
             .required(true)
             .takes_value(true))
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let path = Path::new(file);
    if !path.is_file() {
        eprintln!("No file exists at {}", file);
        std::process::exit(1)
    }
    run(path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_secret() {
        assert_eq!(get_secret("test/secret").unwrap(), "abcd")
    }
}
