//! Processus enfants (llama.cpp, whisper.cpp) : ils sont placés dans un « job » Windows qui les tue
//! quand Verbatim se ferme, y compris en cas de plantage ou de fermeture forcée.
use std::process::Child;

#[cfg(windows)]
mod job {
    use std::ffi::c_void;
    use std::sync::OnceLock;

    #[repr(C)]
    #[derive(Default)]
    struct BasicLimits {
        per_process_user_time: i64,
        per_job_user_time: i64,
        flags: u32,
        min_working_set: usize,
        max_working_set: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct ExtendedLimits {
        basic: BasicLimits,
        io: [u64; 6],
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory: usize,
        peak_job_memory: usize,
    }
    extern "system" {
        fn CreateJobObjectW(attrs: *mut c_void, name: *const u16) -> *mut c_void;
        fn SetInformationJobObject(job: *mut c_void, class: i32, info: *mut c_void, len: u32) -> i32;
        fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
        fn TerminateJobObject(job: *mut c_void, code: u32) -> i32;
    }

    const KILL_ON_JOB_CLOSE: u32 = 0x2000;
    const EXTENDED_LIMIT_INFORMATION: i32 = 9;

    /// Poignée du job (jamais fermée : Windows la ferme à la fin du processus, ce qui tue les enfants).
    static JOB: OnceLock<usize> = OnceLock::new();

    fn handle() -> Option<*mut c_void> {
        let h = *JOB.get_or_init(|| unsafe {
            let h = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if h.is_null() {
                return 0;
            }
            let mut info = ExtendedLimits::default();
            info.basic.flags = KILL_ON_JOB_CLOSE;
            let ok = SetInformationJobObject(h, EXTENDED_LIMIT_INFORMATION, &mut info as *mut _ as *mut c_void, std::mem::size_of::<ExtendedLimits>() as u32);
            if ok == 0 { 0 } else { h as usize }
        });
        (h != 0).then_some(h as *mut c_void)
    }

    pub fn adopt(process: *mut c_void) {
        if let Some(j) = handle() {
            unsafe { AssignProcessToJobObject(j, process) };
        }
    }

    pub fn kill_all() {
        if let Some(j) = handle() {
            unsafe { TerminateJobObject(j, 1) };
        }
    }
}

/// À appeler juste après le lancement d'un moteur : il sera tué avec l'application.
pub fn adopt(child: &Child) {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        job::adopt(child.as_raw_handle());
    }
    #[cfg(not(windows))]
    let _ = child;
}

/// Tue les moteurs (whisper-*, llama-server) d'un lancement précédent restés en vie, par exemple après une fermeture
/// forcée pendant le chargement de Whisper : ils occuperaient la mémoire vidéo et bloqueraient les transcriptions.
/// Seuls les exécutables situés dans les dossiers donnés (redist, ia) sont visés, jamais ceux d'Ollama.
pub fn kill_orphans(dirs: &[std::path::PathBuf]) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let roots: Vec<String> = dirs.iter().map(|d| format!("'{}'", d.display().to_string().replace("'", "''"))).collect();
        if roots.is_empty() {
            return;
        }
        let filter = format!(
            "$r=@({}); Get-CimInstance Win32_Process | Where-Object {{ $p=$_.ExecutablePath; $_.Name -match '^(whisper|llama)' -and $p -and ($r | Where-Object {{ $p.StartsWith($_, [StringComparison]::OrdinalIgnoreCase) }}) }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}",
            roots.join(",")
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &filter])
            .creation_flags(0x0800_0000)
            .status();
    }
    #[cfg(not(windows))]
    let _ = dirs;
}

/// Tue tous les moteurs encore en cours (llama.cpp, whisper.cpp, traduction comprise).
pub fn kill_all() {
    crate::live::release();
    #[cfg(windows)]
    job::kill_all();
}
