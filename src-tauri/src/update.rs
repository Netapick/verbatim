//! Mise à jour automatique : l'app lit la dernière release GitHub (`latest`), télécharge l'installateur,
//! vérifie son SHA-256 puis l'exécute en silence et se relance.

use crate::R;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::PathBuf, time::Duration};

const REPO: &str = "Netapick/verbatim";
const DL_PREFIX: &str = "https://github.com/Netapick/verbatim/releases/download/";

fn ver_tuple(v: &str) -> Vec<u64> {
    v.trim().trim_start_matches(['v', 'V']).split('.').map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)).collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let (mut a, mut b) = (ver_tuple(latest), ver_tuple(current));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

/// Cherche une version plus récente dans la dernière release publiée.
#[tauri::command(async)]
pub fn update_check() -> R<Value> {
    let current = env!("CARGO_PKG_VERSION");
    let rel: Value = ureq::get(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .set("User-Agent", &format!("Verbatim/{current}"))
        .set("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Recherche de mise à jour impossible : {e}"))?
        .into_json()
        .map_err(|e| e.to_string())?;
    let latest = rel["tag_name"].as_str().unwrap_or("").trim_start_matches(['v', 'V']).to_string();
    let asset = rel["assets"].as_array().and_then(|a| a.iter().find(|x| x["name"].as_str().map_or(false, |n| n.ends_with("_x64-setup.exe"))));
    let (url, sha) = match asset {
        Some(a) => (
            a["browser_download_url"].as_str().unwrap_or("").to_string(),
            a["digest"].as_str().unwrap_or("").trim_start_matches("sha256:").to_lowercase(),
        ),
        None => (String::new(), String::new()),
    };
    let available = !latest.is_empty() && is_newer(&latest, current) && url.starts_with(DL_PREFIX) && sha.len() == 64;
    Ok(json!({ "available": available, "current": current, "latest": latest, "notes": rel["body"].as_str().unwrap_or(""), "url": url, "sha": sha }))
}

fn q(s: &str) -> String {
    s.replace('\'', "''")
}

/// Télécharge l'installateur, vérifie son intégrité, puis le lance en silence et relance l'app.
#[tauri::command(async)]
pub fn update_install(app: tauri::AppHandle, url: String, sha: String) -> R<()> {
    if !url.starts_with(DL_PREFIX) || sha.len() != 64 {
        return Err("Mise à jour refusée : source ou somme de contrôle invalide.".into());
    }
    let dir: PathBuf = std::env::temp_dir().join("verbatim-update");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let setup = dir.join("verbatim-setup.exe");
    let mut hasher = Sha256::new();
    {
        let resp = ureq::get(&url).timeout(Duration::from_secs(300)).call().map_err(|e| format!("Téléchargement impossible : {e}"))?;
        let mut r = resp.into_reader();
        let mut f = fs::File::create(&setup).map_err(|e| e.to_string())?;
        let mut buf = [0u8; 65536];
        loop {
            let n = r.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            std::io::Write::write_all(&mut f, &buf[..n]).map_err(|e| e.to_string())?;
        }
    }
    let got = format!("{:x}", hasher.finalize());
    if got != sha.to_lowercase() {
        let _ = fs::remove_file(&setup);
        return Err("Mise à jour annulée : somme de contrôle invalide (fichier corrompu ou modifié).".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let inst_dir = exe.parent().ok_or("Dossier d'installation introuvable.")?.to_path_buf();
    // Script détaché : attend la fermeture de l'app, installe en silence dans le même dossier, relance.
    let script = format!(
        "Start-Sleep -Seconds 2; Start-Process -FilePath '{}' -ArgumentList '/S /D={}' -Wait; Start-Process -FilePath '{}'",
        q(&setup.to_string_lossy()), q(&inst_dir.to_string_lossy()), q(&exe.to_string_lossy())
    );
    let mut c = std::process::Command::new("powershell");
    c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-Command", &script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000 | 0x0000_0008); // CREATE_NO_WINDOW | DETACHED_PROCESS
    }
    c.spawn().map_err(|e| format!("Lancement de l'installateur impossible : {e}"))?;
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(is_newer("0.2.1", "0.2"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.9", "0.2.0"));
    }
}
