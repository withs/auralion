use env_logger::Builder;
use log::LevelFilter;
use log::{debug, error, info, warn};

use std::process::exit;
use std::{env, fs, path, thread};

use crate::config::load_config;
use crate::sc_defs::{Collection, Follower};
mod config;
mod sc_defs;

const SC_API_BASE: &str = "https://api-v2.soundcloud.com";

struct App {
    cfg: config::Config,
}

impl App {
    fn new(with_config: config::Config) -> Self {
        return Self { cfg: with_config };
    }

    fn build_endpoint(
        &self,
        with_ressources: Vec<(&str, String)>,
        and_args: Vec<(&str, String)>,
    ) -> String {
        let mut constructed_endp: String = String::new();
        constructed_endp.push_str(SC_API_BASE);

        let constructed_rss: String = with_ressources
            .iter()
            .map(|(k, v)| {
                if v.is_empty() {
                    return format!("/{}", k);
                }
                return format!("/{}/{}", k, v);
            })
            .collect::<Vec<String>>()
            .join("");
        constructed_endp.push_str(&constructed_rss);

        constructed_endp.push_str(&format!("?client_id={}", &self.cfg.client_token));

        let constructed_args: String = and_args
            .iter()
            .map(|(k, v)| {
                return format!("&{}={}", k, v);
            })
            .collect::<Vec<String>>()
            .join("");
        constructed_endp.push_str(&constructed_args);

        return constructed_endp;
    }

    fn get_user_by_name(&self, name: &str) -> Result<sc_defs::User, String> {
        let search_endp: String = self.build_endpoint(
            vec![("search", "".to_string())],
            vec![("q", name.to_string())],
        );

        let rsp: reqwest::blocking::Response = match reqwest::blocking::get(search_endp) {
            Ok(rsp) => rsp,
            Err(err) => {
                debug!("{:?}", err);
                return Err("failed to request data".to_string());
            }
        };

        let sc_usrs: sc_defs::Collection<sc_defs::SearchResult> =
            match serde_json::from_str(&rsp.text().unwrap()) {
                Ok(usr) => usr,
                Err(err) => {
                    debug!("{:?}", err);
                    return Err("failed to parse search result".to_string());
                }
            };

        let filtered_rslt: Vec<sc_defs::SearchResult> = sc_usrs
            .collection
            .into_iter()
            .filter(|s| {
                return s.is_an_user() && s.permalink == name;
            })
            .collect();

        if filtered_rslt.is_empty() {
            return Err(format!("could not found any user with name: {}", name));
        }

        return self.get_user(&filtered_rslt[0].id.to_string());
    }

    fn get_user(&self, from_id: &str) -> Result<sc_defs::User, String> {
        let usr_endp: String = self.build_endpoint(vec![("users", from_id.to_string())], vec![]);

        let rsp: reqwest::blocking::Response = match reqwest::blocking::get(usr_endp) {
            Ok(rsp) => rsp,
            Err(err) => {
                debug!("{:?}", err);
                return Err("failed to request data".to_string());
            }
        };

        let sc_usr: sc_defs::User = match serde_json::from_str(&rsp.text().unwrap()) {
            Ok(usr) => usr,
            Err(err) => {
                debug!("{:?}", err);
                return Err("failed to parse user".to_string());
            }
        };

        return Ok(sc_usr);
    }

    fn get_folower_for_user(&self, with_user: &sc_defs::User) -> Vec<Follower> {
        let mut followers_endp: String = self.build_endpoint(
            vec![
                ("users", with_user.id.to_string()),
                ("followers", "".to_string()),
            ],
            vec![("limit", "200".to_string())],
        );
        let mut follower_col: Collection<Follower> = Collection {
            collection: Vec::new(),
            next_href: Some("st".to_string()),
        };
        let mut all_followers: Vec<Follower> = Vec::new();

        while follower_col.have_next_href() {
            // for unwraping since we cannot enter the loop if next_reft is None
            // if collection is not the first request add offset param
            if !follower_col.is_first() {
                followers_endp = self.build_endpoint(
                    vec![
                        ("users", with_user.id.to_string()),
                        ("followers", "".to_string()),
                    ],
                    vec![
                        ("limit", "200".to_string()),
                        ("offset", follower_col.get_next_href_offset()),
                    ],
                );
            }

            // http request
            let rsp = match reqwest::blocking::get(&followers_endp) {
                Ok(rsp) => rsp,
                Err(err) => {
                    warn!("request failed err: {:?} (stoping loop)", err);
                    break;
                }
            };

            let rsp_text: String = rsp.text().unwrap();

            follower_col = match serde_json::from_str(&rsp_text) {
                Ok(col) => col,
                Err(err) => {
                    warn!("json decode failed: {:?} (stoping loop)", err);
                    break;
                }
            };

            all_followers.append(&mut follower_col.collection);
        }

        return all_followers;
    }
}

fn main() {
    let mut builder = Builder::from_default_env();
    let log_lvl: LevelFilter = if env::args().into_iter().any(|x| x == "--debug".to_string()) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    builder.filter(None, log_lvl).init();

    // load config
    let config: config::Config = match load_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("an error occured loading config err: {}", err);
            exit(1);
        }
    };

    // create our App
    let app: App = App::new(config);

    // check & create imgs out folder
    let out_fold: path::PathBuf = path::PathBuf::from(&app.cfg.img_out_path);
    info!("out folder exist: {}", out_fold.exists());
    if !out_fold.exists() {
        fs::create_dir_all(&out_fold).unwrap();
        info!("creating out folder ({})", &app.cfg.img_out_path);
    }

    // get user specidied in config from permalink
    /*
    let sc_user: sc_defs::User = match app.get_user_by_name("take") {
        Ok(usr) => { usr },
        Err(err) => {
            error!("and error occured while retrieving user: {}, err: {}", &app.cfg.user_to_fetch, err);
            exit(1);
        }
    };
    info!("user: {}", &sc_user.username);
    */

    // get user specified in config
    let sc_user: sc_defs::User = match app.get_user(&app.cfg.user_to_fetch) {
        Ok(usr) => usr,
        Err(err) => {
            error!(
                "and error occured while retrieving user: {}, err: {}",
                &app.cfg.user_to_fetch, err
            );
            exit(1);
        }
    };
    info!("user: {}", &sc_user.username);

    // get followers of this user
    let sc_followers: Vec<Follower> = app.get_folower_for_user(&sc_user);
    info!(
        "collected followers: {} ({})",
        &sc_followers.len(),
        &sc_user.followers_count
    );

    // split followers into multiple vec
    let tasks: Vec<Vec<Follower>> = sc_followers
        .chunks(sc_followers.len() / app.cfg.threads)
        .map(|x| x.to_vec())
        .collect();
    debug!("{:#?}", tasks.len());

    let mut threads = Vec::new();

    for tsk in 0..app.cfg.threads {
        let ts = tasks[tsk].clone();
        let p = out_fold.clone();
        threads.push(thread::spawn(move || {
            for follower in ts.iter() {
                if follower.have_default_avatar() {
                    continue;
                }

                match follower.download_avatar(p.as_path()) {
                    Ok(filename) => {
                        info!(
                            "(t:{}) downloaded {}'s avatars to {}.jpg",
                            tsk, &follower.username, filename
                        );
                    }
                    Err(err) => {
                        error!(
                            "(t:{}) an error occured while downloading {}'s avatars: {}",
                            tsk, &follower.username, err
                        );
                    }
                }
            }
        }));
    }
    for child in threads {
        child.join();
    }
}
