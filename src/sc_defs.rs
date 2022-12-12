use std::{fs, path};

use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub followers_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Follower {
    pub avatar_url: String,
    pub username: String,
}

impl Follower {
    pub fn have_default_avatar(&self) -> bool {
        return self.avatar_url.contains("default_avatar");
    }

    pub fn download_avatar(&self, to_path: &path::Path) -> Result<String, String> {
        let mut rsp: reqwest::blocking::Response = match reqwest::blocking::get(&self.avatar_url) {
            Ok(rsp) => rsp,
            Err(err) => {
                debug!("{:?}", err);
                return Err("failed to request data".to_string());
            }
        };

        let file_name: String = random_string::generate(7, "abcdefghijklmnopqrstuvwxyz");

        let mut file: fs::File =
            match fs::File::create(to_path.join(format!("{}.jpg", file_name)).as_path()) {
                Ok(file) => file,
                Err(err) => {
                    debug!("{:?}", err);
                    return Err(format!("failed to create file with name: {}", file_name));
                }
            };

        match rsp.copy_to(&mut file) {
            Ok(copied_bytes) => {
                debug!("copied {}B to {}.jpg", copied_bytes, file_name);
                return Ok(file_name);
            }
            Err(err) => {
                debug!("{:?}", err);
                return Err(format!("failed to write content to file {}.jpg", file_name));
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub id: i32,
    pub kind: String,
    pub permalink: String,
}

impl SearchResult {
    pub fn is_an_user(&self) -> bool {
        return self.kind == "user";
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection<T> {
    pub collection: Vec<T>,
    pub next_href: Option<String>,
}

impl<T> Collection<T> {
    pub fn have_next_href(&self) -> bool {
        return match &self.next_href {
            Some(v) => !v.is_empty(),
            None => false,
        };
    }

    pub fn is_first(&self) -> bool {
        return match &self.next_href {
            Some(v) => v == "st",
            None => false,
        };
    }

    pub fn get_next_href_offset(&self) -> String {
        return match &self.next_href {
            Some(v) => v.split('?').collect::<Vec<&str>>()[1]
                .split('&')
                .collect::<Vec<&str>>()[0]
                .split('=')
                .collect::<Vec<&str>>()[1]
                .to_string(),
            _ => "no more".to_string(),
        };
    }
}
