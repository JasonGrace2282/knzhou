use reqwest::blocking::Client;
use std::{fs, io::Write, path::PathBuf};

pub fn fetch_handout(client: &Client, handout: &str, output: PathBuf) {
    let mut response = client
        .get(format!("https://knzhou.github.io/handouts/{}.pdf", handout))
        .send();
    if response.is_err() {
        eprintln!(
            "Error fetching handout {}. Try checking your internet connection?",
            handout
        );
        std::process::exit(1);
    }
    response = response.unwrap().error_for_status();
    match response {
        Ok(r) => {
            let mut file =
                fs::File::create(output).expect("Should be able to create file in current dir");
            file.write_all(&r.bytes().unwrap())
                .expect("Should be able to write to file");
        }
        Err(r) if matches!(r.status(), Some(reqwest::StatusCode::NOT_FOUND)) => {
            eprintln!("Handout {} not found.", handout);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error fetching handout {}: {}", handout, e);
            std::process::exit(1);
        }
    }
}

pub fn fetch_handouts(client: &Client) -> WebsiteTree {
    let mut response = client
        .get("http://api.github.com/repos/knzhou/knzhou.github.io/git/trees/master")
        .query(&[("recursive", "1")])
        .header(reqwest::header::USER_AGENT, "knzhou-cli")
        .send();

    if response.is_err() {
        eprintln!("Could not access knzhou's website. Try checking your internet connection?");
        std::process::exit(1);
    }

    response = response.unwrap().error_for_status();
    if response.is_err() {
        eprintln!("Error fetching handouts: {}", response.unwrap_err());
        std::process::exit(1);
    }

    let tree = serde_json::from_str::<WebsiteTree>(&response.unwrap().text().unwrap());
    if tree.is_err() {
        let err = tree.unwrap_err();
        if err.is_data() {
            println!("{}", err);
            eprintln!("Outdated cli - the github api has been updated. Please reinstall knzhou.");
            std::process::exit(1);
        }
        panic!("Error parsing website tree: {}", err);
    }
    tree.unwrap()
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WebsiteTree {
    pub sha: String,
    pub url: String,
    pub tree: Vec<WebsiteTreeEntry>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WebsiteTreeEntry {
    pub path: PathBuf,
    pub size: Option<u64>,
}
