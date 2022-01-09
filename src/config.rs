use std::fmt::Debug;

use anyhow::anyhow;
use json_comments::StripComments;
use serde::{Serialize, de::DeserializeOwned};
use structopt::StructOpt;

use crate::json_merge;

pub fn load<'a, T>(
    default_file: Option<String>,
    user_file: Option<String>,
) -> anyhow::Result<T>
    where T: Serialize + DeserializeOwned + StructOpt + Debug {
    println!("default:{:?}, user_file:{:?}", default_file, user_file);

    let cfg_cmdline: T = T::from_args();
    println!("conf from command line:\n{:#?}", cfg_cmdline);

    let mut default_file = default_file;

    if default_file.is_none() {
        let mut path = std::env::current_exe().unwrap();
        let exe_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        path.pop();
        path.push("conf");
        path.push(format!("{}.{}", exe_name, "json.default"));
        default_file = Some(path.to_str().unwrap().to_string());
        println!("conf_default not set, use:{}", default_file.as_ref().unwrap());
    }
    let default_str = std::fs::read_to_string(default_file.as_ref().unwrap());
    let mut cfg;
    match default_str {
        Ok(s) => {
            let stripped = StripComments::new(s.as_bytes());
            let r = serde_json::from_reader::<StripComments<&[u8]>, serde_json::Value>(stripped);
            match r {
                Ok(cfg_default) => {
                    cfg = cfg_default;
                }
                Err(e) => {
                    return Err(anyhow!("decode conf default file FAILED! {}", e));
                }
            }
        }
        Err(e) => {
            // not allow no conf default
            return Err(anyhow!("read conf default FAILED! {}", e));
        }
    }
    println!("conf default:\n{:#?}", serde_json::from_value::<T>(cfg.clone()).unwrap());

    if user_file.is_some() {
        let user_str = std::fs::read_to_string(user_file.as_ref().unwrap());
        match user_str {
            Ok(s) => {
                let stripped = StripComments::new(s.as_bytes());
                let r = serde_json::from_reader::<StripComments<&[u8]>, serde_json::Value>(stripped);
                match r {
                    Ok(cfg_user) => {
                        json_merge::merge(&mut cfg, cfg_user);
                    }
                    Err(e) => {
                        return Err(anyhow!("decode conf user file FAILED! {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(anyhow!("read conf user file FAILED! {}", e));
            }
        }
        println!("conf merge user:\n{:#?}", serde_json::from_value::<T>(cfg.clone()).unwrap());
    } else {
        // allow no conf user, but print warnings
        println!("no conf user specified");
    }

    let cfg_cmdline = serde_json::to_value(cfg_cmdline).unwrap();
    json_merge::merge(&mut cfg, cfg_cmdline);
    let cfg: T = serde_json::from_value(cfg).unwrap();
    println!("conf merge cmdline:\n{:#?}", cfg);

    return Ok(cfg);
}
