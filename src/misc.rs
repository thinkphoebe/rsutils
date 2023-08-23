use std::net::UdpSocket;

use anyhow::{anyhow, Context};

pub fn relative2absolute(relative: &str, base: &str) -> anyhow::Result<String> {
    let mut path = std::path::PathBuf::from(base);
    if !path.is_dir() {
        path.pop();
    }
    path.push(relative);
    path = path.canonicalize()
            .context(format!("canonicalize {:?}", path.to_str()))?;
    Ok(path.to_str().unwrap().to_string())
}

pub fn get_local_ip() -> anyhow::Result<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            return Err(anyhow!("socket bind got err:{:?}", e));
        }
    };

    // 这里并不需要真正连接成功，只是系统会根据这个 IP 选择对应的网卡，后面获得的也是对应网卡的地址
    if let Err(e) = socket.connect("8.8.8.8:80") {
        return Err(anyhow!("socket connect got err:{:?}", e));
    }

    match socket.local_addr() {
        Ok(addr) => {
            Ok(addr.ip().to_string())
        }
        Err(e) => {
            Err(anyhow!("socket local_addr got err:{:?}", e))
        }
    }
}
