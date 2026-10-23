"""Reconstitue un « hiproot » (SDK ROCm de développement utilisable sous Windows) pour compiler whisper.cpp en HIP.

Les wheels `rocm-sdk` répartissent le SDK en paquets frères (_rocm_sdk_core, _rocm_sdk_libraries_*) et un tar
`_devel.tar` dont les liens symboliques ne peuvent pas être créés sous Windows. On fusionne donc les paquets
d'exécution, puis on extrait du tar les en-têtes, bibliothèques d'import (.lib) et fichiers cmake, en remplaçant
chaque lien par une copie de sa cible.

Usage : python scripts/make-hiproot.py <dossier site-packages contenant rocm_sdk_devel> <dossier hiproot à créer>
"""
import os
import shutil
import sys
import tarfile
from pathlib import Path, PurePosixPath

KEEP = ("include/", "lib/", "share/", "libexec/", "cmake/")  # le reste (bin/ de tests, clients/) est inutile pour compiler


def merge(src: Path, dst: Path):
    for root, dirs, files in os.walk(src):
        r = Path(root).relative_to(src)
        (dst / r).mkdir(parents=True, exist_ok=True)
        for f in files:
            d = dst / r / f
            if not d.exists():
                shutil.copy2(Path(root) / f, d)


def main(sp: Path, out: Path):
    out.mkdir(parents=True, exist_ok=True)
    pkgs = [p for p in sp.iterdir() if p.is_dir() and p.name.startswith("_rocm_sdk_") and not p.name.startswith("_rocm_sdk_devel")]
    print("paquets d'exécution :", [p.name for p in pkgs])
    for p in pkgs:
        print("  fusion", p.name)
        merge(p, out)
    tarp = sp / "rocm_sdk_devel" / "_devel.tar"
    pending = []
    n_files = 0
    with tarfile.open(tarp) as t:
        for m in t:
            rel = PurePosixPath(m.name).parts[1:]  # retire « _rocm_sdk_devel/ »
            if not rel:
                continue
            relp = "/".join(rel)
            if not (relp + "/").startswith(KEEP) and not relp.startswith(KEEP):
                continue
            dst = out.joinpath(*rel)
            if m.isdir():
                dst.mkdir(parents=True, exist_ok=True)
            elif m.isfile():
                if dst.exists():
                    continue
                dst.parent.mkdir(parents=True, exist_ok=True)
                with t.extractfile(m) as fi, open(dst, "wb") as fo:
                    shutil.copyfileobj(fi, fo, 1024 * 1024)
                n_files += 1
            elif m.issym() or m.islnk():
                pending.append((rel, m.linkname))
    print("fichiers extraits :", n_files, "| liens à résoudre :", len(pending))
    # résolution des liens (relatifs, parfois chaînés) : plusieurs passes
    missing = []
    for _ in range(4):
        missing = []
        for rel, link in pending:
            dst = out.joinpath(*rel)
            if dst.exists():
                continue
            target = Path(os.path.normpath(dst.parent / link))
            if target.is_file():
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(target, dst)
            elif target.is_dir():
                shutil.copytree(target, dst, dirs_exist_ok=True)
            else:
                missing.append((rel, link))
        pending = missing
        if not missing:
            break
    print("liens non résolus :", len(missing))
    for rel, link in missing[:15]:
        print("  ", "/".join(rel), "->", link)


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
