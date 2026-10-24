//! Rééchantillonnage de qualité (sinus cardinal fenêtré, polyphase) pour l'enregistrement et l'import.
//! L'interpolation linéaire utilisée auparavant ne filtre pas les fréquences au-dessus de la moitié de la fréquence
//! d'arrivée : à 16 kHz, les aigus (sifflantes, bruit, musique jusqu'à 22 kHz) se repliaient dans la bande de la voix.
//! Ici un filtre passe-bas précède chaque réduction de fréquence ; le signal n'est ni décalé ni déformé en durée.

use std::f64::consts::PI;

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 { 1.0 } else { (PI * x).sin() / (PI * x) }
}

/// Rééchantillonneur en flux : s'utilise bloc par bloc (rappel audio) ou sur un fichier entier.
pub struct Stream {
    from: u64,
    to: u64,
    half: i64,
    taps: usize,
    /// un noyau de `taps` coefficients par phase fractionnaire
    table: Vec<f32>,
    grid: u64,
    exact: bool,
    g: u64,
    buf: Vec<f32>,
    /// indice, dans le flux d'entrée, de `buf[0]`
    base: i64,
    n_out: u64,
}

impl Stream {
    pub fn new(from: u32, to: u32) -> Stream {
        let (from, to) = (from as u64, to as u64);
        let g = gcd(from, to).max(1);
        let phases = to / g;
        let (grid, exact) = if phases <= 1024 { (phases, true) } else { (1024, false) };
        // coupure à 96 % de la moitié de la plus basse des deux fréquences, 12 passages par zéro de chaque côté
        let fc = 0.5 * (to as f64 / from as f64).min(1.0) * 0.96;
        let half = (12.0 / (2.0 * fc)).ceil() as i64;
        let taps = (2 * half) as usize;
        let mut table = vec![0.0f32; grid as usize * taps];
        for p in 0..grid as usize {
            let frac = p as f64 / grid as f64;
            let mut k_w = vec![0.0f64; taps];
            let mut sum = 0.0;
            for (k, w) in k_w.iter_mut().enumerate() {
                let t = (1 - half + k as i64) as f64 - frac;
                let u = t / half as f64;
                let win = if u.abs() < 1.0 { 0.42 + 0.5 * (PI * u).cos() + 0.08 * (2.0 * PI * u).cos() } else { 0.0 };
                *w = 2.0 * fc * sinc(2.0 * fc * t) * win;
                sum += *w;
            }
            for (k, w) in k_w.iter().enumerate() {
                table[p * taps + k] = (w / sum) as f32; // gain unitaire en continu, pour chaque phase
            }
        }
        // historique de silence : le premier échantillon de sortie coïncide avec le premier d'entrée
        Stream { from, to, half, taps, table, grid, exact, g, buf: vec![0.0; half as usize], base: -half, n_out: 0 }
    }

    /// Nombre d'échantillons d'entrée à fournir en fin de flux pour vider le filtre.
    pub fn flush_len(&self) -> usize {
        self.half as usize + 2
    }

    /// Ajoute des échantillons d'entrée ; les échantillons de sortie qui en résultent sont ajoutés à `out`.
    pub fn push(&mut self, x: &[f32], out: &mut Vec<f32>) {
        if self.from == self.to {
            out.extend_from_slice(x);
            return;
        }
        self.buf.extend_from_slice(x);
        loop {
            let num = self.n_out * self.from;
            let ipos = (num / self.to) as i64;
            let r = num % self.to;
            if ipos + self.half - self.base >= self.buf.len() as i64 {
                break;
            }
            let idx = if self.exact { (r / self.g) as usize } else { ((r * self.grid + self.to / 2) / self.to).min(self.grid - 1) as usize };
            let start = (ipos + 1 - self.half - self.base) as usize;
            let kern = &self.table[idx * self.taps..(idx + 1) * self.taps];
            let sl = &self.buf[start..start + self.taps];
            let mut acc = 0.0f32;
            for (a, b) in sl.iter().zip(kern) {
                acc += a * b;
            }
            out.push(acc);
            self.n_out += 1;
        }
        let next_start = ((self.n_out * self.from) / self.to) as i64 + 1 - self.half - self.base;
        if next_start > 8192 {
            self.buf.drain(..next_start as usize);
            self.base += next_start;
        }
    }
}

/// Rééchantillonne un signal entier. La durée est conservée (même nombre d'échantillons à la fréquence d'arrivée qu'avant,
/// à un échantillon près).
#[cfg(test)]
pub fn resample(x: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || x.is_empty() {
        return x.to_vec();
    }
    let n = (x.len() as f64 * to as f64 / from as f64) as usize;
    let mut s = Stream::new(from, to);
    let mut out = Vec::with_capacity(n + 16);
    for chunk in x.chunks(1 << 18) {
        s.push(chunk, &mut out);
    }
    s.push(&vec![0.0; s.flush_len()], &mut out);
    out.resize(n, 0.0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(f: f64, rate: u32, secs: f64) -> Vec<f32> {
        (0..(rate as f64 * secs) as usize).map(|i| (2.0 * PI * f * i as f64 / rate as f64).sin() as f32 * 0.5).collect()
    }
    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    #[test]
    fn voix_conservee_et_aigus_filtres() {
        for from in [44_100u32, 48_000, 22_050, 32_000] {
            let ok = resample(&tone(1000.0, from, 2.0), from, 16_000);
            assert!((rms(&ok[2000..28000]) - 0.5 / 2f64.sqrt()).abs() < 0.01, "1 kHz altéré depuis {from} Hz");
            // 10 kHz > 8 kHz (Nyquist de 16 kHz) : doit disparaître, pas se replier à 6 kHz
            if from > 20_000 {
                let bad = resample(&tone(10_000.0, from, 2.0), from, 16_000);
                assert!(rms(&bad[2000..28000]) < 0.005, "10 kHz replié depuis {from} Hz : {}", rms(&bad[2000..28000]));
            }
        }
    }

    #[test]
    fn duree_conservee() {
        for (from, n) in [(44_100u32, 441_000usize), (48_000, 48_000 * 7 + 13), (16_000, 1234)] {
            let y = resample(&vec![0.1; n], from, 16_000);
            assert_eq!(y.len(), (n as f64 * 16_000.0 / from as f64) as usize);
        }
    }

    #[test]
    fn flux_identique_au_bloc_entier() {
        let x: Vec<f32> = (0..50_000).map(|i| ((i * 7919) % 2000) as f32 / 1000.0 - 1.0).collect();
        let tout = resample(&x, 44_100, 16_000);
        let mut s = Stream::new(44_100, 16_000);
        let mut out = Vec::new();
        for c in x.chunks(441) {
            s.push(c, &mut out);
        }
        for (a, b) in out.iter().zip(&tout) {
            assert!((a - b).abs() < 1e-5);
        }
        assert!(out.len() + 40 >= tout.len());
    }
}
