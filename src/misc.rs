use anyhow::Context;

pub fn relative2absolute(relative: &str, base: &str) -> anyhow::Result<String> {
    let mut path = std::path::PathBuf::from(base);
    path.pop();
    path.push(relative);
    path = path.canonicalize() .context(format!("canonicalize {:?}", path.to_str()))?;
    Ok(path.to_str().unwrap().to_string())
}
