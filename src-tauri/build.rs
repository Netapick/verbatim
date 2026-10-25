/// Horodatage UTC « AAAAMMJJ-HHMM » (sans dépendance : calendrier grégorien à partir de l'époque Unix).
fn build_stamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (h, m) = (rem / 3600, rem % 3600 / 60);
    // jour civil depuis l'époque (algorithme de H. Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if mo <= 2 { 1 } else { 0 };
    format!("{y:04}{mo:02}{d:02}-{h:02}{m:02}")
}

fn main() {
    // étiquette de build « v{version}-{horodatage} » : la CI fixe VERBATIM_BUILD_TS pour que l'app et la release portent la même valeur
    println!("cargo:rerun-if-env-changed=VERBATIM_BUILD_TS");
    let stamp = std::env::var("VERBATIM_BUILD_TS").ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(build_stamp);
    println!("cargo:rustc-env=VERBATIM_BUILD_TS={}", stamp.trim());
    tauri_build::build()
}
