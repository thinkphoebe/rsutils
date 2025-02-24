use anyhow::anyhow;
use json_comments::StripComments;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::json_merge;

// default 和 user 可传入 json 文件路径或 json 字符串
// cmdline 传入 structopt 解析的命令行结构
// 优先级：cmdline > user > default
pub fn load<'a, T>(
    default: Option<String>,
    user: Option<String>,
    cmdline: Option<T>,
) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    eprintln!("default:{:?}", default);
    eprintln!("user:{:?}", user);

    let default = if let Some(v) = default {
        v
    } else {
        let mut path = std::env::current_exe().unwrap();
        let exe_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        path.pop();
        path.push("conf");
        path.push(format!("{}.{}", exe_name, "json.default"));
        let v = path.to_str().unwrap().to_string();
        eprintln!("conf_default not set, use:{}", v);
        v
    };

    let mut cfg;
    let mut default_str = match std::fs::read_to_string(&default) {
        Ok(s) => {
            eprintln!("read default as file OK:{}", s);
            s
        }
        Err(e) => {
            eprintln!("read default as file err, try parse as json content:{:?}", e);
            default
        }
    };
    let stripped = StripComments::new(default_str.as_bytes());
    let r = serde_json::from_reader::<StripComments<&[u8]>, serde_json::Value>(stripped);
    match r {
        Ok(cfg_default) => {
            cfg = cfg_default;
        }
        Err(e) => {
            return Err(anyhow!("decode conf default FAILED! {}", e));
        }
    }
    eprintln!("================================> conf default:\n{:#?}", cfg);

    if let Some(user) = user {
        let mut user_str = match std::fs::read_to_string(&user) {
            Ok(s) => {
                eprintln!("read user as file OK:{}", s);
                s
            }
            Err(e) => {
                eprintln!("read user as file err, try parse as json content:{:?}", e);
                user
            }
        };
        let stripped = StripComments::new(user_str.as_bytes());
        let r = serde_json::from_reader::<StripComments<&[u8]>, serde_json::Value>(stripped);
        match r {
            Ok(cfg_user) => {
                eprintln!("================================> conf user:\n{:#?}", cfg_user);
                json_merge::merge(&mut cfg, cfg_user);
            }
            Err(e) => {
                return Err(anyhow!("decode conf user FAILED! {}", e));
            }
        }
        eprintln!("================================> conf merge user:\n{:#?}", cfg);
    } else {
        // allow no conf user, but print warnings
        eprintln!("no conf user specified");
    }

    if let Some(c) = cmdline {
        let cfg_cmdline = serde_json::to_value(c).unwrap();
        eprintln!("================================> conf cmdline:\n{:#?}", cfg_cmdline);

        json_merge::merge(&mut cfg, cfg_cmdline);
        eprintln!("================================> conf merge cmdline:\n{:#?}", cfg);
    }

    let cfg: T = serde_json::from_value(cfg).unwrap();
    eprintln!("================================> conf final:\n{:#?}", cfg);

    Ok(cfg)
}
