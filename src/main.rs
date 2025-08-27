use anyhow::{bail, Context, Result};
use std::{fs, path::{Path, PathBuf}};
use std::collections::HashMap;   // in-memory key/value store
use sha1::{Digest, Sha1};

/* -------- repo paths -------- */

fn repo_dir() -> PathBuf {
    PathBuf::from(".minigit")
}

fn objects_dir() -> PathBuf {
    repo_dir().join("objects")
}

fn index_path() -> PathBuf {
    repo_dir().join("index.json")
}

fn commits_path() -> PathBuf {
    repo_dir().join("commits.jsonl")
}

/* -------- guard -------- */

/// Ensure we're inside a mini-git repo (i.e., `.minigit/` exists).
fn ensure_repo() -> Result<()> {
    if !repo_dir().exists() {
        bail!("Not a mini-git repo (missing .minigit). Run `mini-git init` first.");
    }
    Ok(())
}

/// path -> blob_id (e.g., "src/main.rs" -> "a94a8fe5...").
type Index = HashMap<String, String>;

/// Load `.minigit/index.json`. If it doesn't exist, return an empty map.
/// Returns Result<Index> so errors bubble up cleanly.
fn load_index() -> Result<Index> {
    if !index_path().exists() {
        return Ok(Index::new());
    }
    // Read the whole file into bytes
    let bytes = fs::read(index_path())
        .with_context(|| format!("reading {}", index_path().display()))?;

    // Turn JSON bytes into an Index (HashMap<String, String>)
    let idx: Index = serde_json::from_slice(&bytes)
        .with_context(|| "parsing index.json as JSON")?;

    Ok(idx)
}

/// Save the Index as human-readable JSON (pretty) to `.minigit/index.json`.
fn save_index(index: &Index) -> Result<()> {
    // Serialize the HashMap as JSON bytes
    let data = serde_json::to_vec_pretty(index)
        .with_context(|| "serializing index to JSON")?;

    // Write to disk
    fs::write(index_path(), data)
        .with_context(|| format!("writing {}", index_path().display()))?;

    Ok(())
}



// Turn any input bytes into a lowercase SHA-1 hex string (40 chars).
fn sha1_hex(bytes: impl AsRef<[u8]>) -> String {
    // 1) Make a new SHA-1 hasher.
    let mut h = Sha1::new();

    // 2) Feed the input (as bytes) into the hasher. (No copies; as_ref() borrows.)
    h.update(bytes.as_ref());

    // 3) Finish the hash: get 20 raw bytes (not text!).
    let out = h.finalize(); // e.g. [0xaa, 0xf4, 0xc6, …] for "hello"

    // 4) A tiny lookup table: 0..15 → '0'..'f' (hex digits).
    const T: &[u8; 16] = b"0123456789abcdef";

    // 5) Pre-allocate space for 40 characters (2 hex chars per byte).
    let mut s = String::with_capacity(out.len() * 2);

    // 6) For each byte, split into two 4-bit numbers (nibbles) and map to hex.
    for b in out {
        // high nibble: top 4 bits → index 0..15 → hex char
        s.push(T[(b >> 4) as usize] as char);
        // low nibble: bottom 4 bits → index 0..15 → hex char
        s.push(T[(b & 0x0f) as usize] as char);
    }

    // 7) Return the 40-char hex string.
    s
}

fn to_repo_relative(path: &Path) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let abs = if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) };
    let rel = abs.strip_prefix(&cwd).unwrap_or(&abs);
    Ok(rel.to_string_lossy().into_owned())
}

fn stage_file(path: &Path, index: &mut Index) -> Result<()> {
    // 1) read bytes of the file
    let data = fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    // 2) hash bytes → blob id (hex)
    let blob_id = sha1_hex(&data);

    // 3) write blob once under .minigit/objects/<hash>
    let obj = objects_dir().join(&blob_id);
    if !obj.exists() {
        fs::write(&obj, &data).with_context(|| format!("writing blob {}", blob_id))?;
    }

    // 4) record staging: repo-relative path → blob id
    let rel = to_repo_relative(path)?;
    index.insert(rel, blob_id);

    Ok(())
}

fn walkdir(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            out.extend(walkdir(&p)?);
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(out)
}

fn cmd_init() -> Result<()> {
    if repo_dir().exists() {
        println!(".minigit already exists");
        return Ok(());
    }
    fs::create_dir_all(objects_dir())?;
    save_index(&Index::new())?;
    println!("Initialized empty mini-git repo in .minigit/");
    Ok(())
}

fn cmd_add(paths: Vec<PathBuf>) -> Result<()> {
    ensure_repo()?; // guard: must run `init` first
    fs::create_dir_all(objects_dir())?; // safe if already exists

    let mut index = load_index()?;

    for p in paths {
        if p.is_dir() {
            for f in walkdir(&p)? {
                stage_file(&f, &mut index)?;
            }
        } else if p.is_file() {
            stage_file(&p, &mut index)?;
        } else {
            eprintln!("Skipping (not found): {}", p.display());
        }
    }

    save_index(&index)?;
    println!("Staged {} path(s).", index.len());
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("init") => cmd_init(),
        Some("add") => {
            let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
            if paths.is_empty() {
                bail!("Usage: mini-git add <files-or-dirs>");
            }
            cmd_add(paths)
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  mini-git init");
            eprintln!("  mini-git add <files-or-dirs>");
            Ok(())
        }
    }
}
