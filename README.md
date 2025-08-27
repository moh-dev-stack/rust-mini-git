# mini-git

A tiny, learning-focused version control tool written in Rust.
It demonstrates **content-addressed storage** (SHA-1 blobs) and a simple **staging index** — great for showing employers you understand filesystems, hashing, and CLI ergonomics.

> **Current commands:** `init`, `add`
> *(Commit/log/status are suggested next steps below.)*

---

## Table of contents

* [Requirements](#requirements)
* [Install & Build](#install--build)
* [How It Works (Concepts)](#how-it-works-concepts)
* [Quick Start (Copy/Paste)](#quick-start-copypaste)
* [CLI Usage](#cli-usage)
* [Project Layout on Disk](#project-layout-on-disk)
* [Verify It Works (Sanity Checks)](#verify-it-works-sanity-checks)
* [Learning Notes (Interview Talking Points)](#learning-notes-interview-talking-points)
* [Run Tests](#run-tests)
* [Next Steps / Roadmap](#next-steps--roadmap)
* [Troubleshooting](#troubleshooting)

---

## Requirements

* Rust **stable** (1.70+ is safe). Install via [rustup](https://rustup.rs/).
* macOS / Linux / WSL. (Windows paths should also work.)

---

## Install & Build

```bash
git clone <your-repo-url> mini-git
cd mini-git
cargo build --release
```

The binary will be at:

```
./target/release/mini-git
```

(You can also run via `cargo run -- <args>` while developing.)

---

## How It Works (Concepts)

**Content-addressed storage**

* File bytes are hashed with **SHA-1** → a 40-char hex string (the “blob id”).
* Blobs are stored at `.minigit/objects/<blob-id>`.
* Identical content ⇒ identical blob id ⇒ automatic deduplication.

**Staging index**

* `.minigit/index.json` is a JSON map:

  ```json
  { "path/relative/to/repo": "<blob-id>", ... }
  ```
* Running `mini-git add <files-or-dirs>`:

  1. Reads file bytes
  2. Computes blob id (SHA-1)
  3. Writes the blob under `.minigit/objects/` (if not already present)
  4. Records `path → blob_id` in `index.json`

**Repo-relative paths**

* Paths are stored **relative** to the repo’s root (portable across machines).

---

## Quick Start (Copy/Paste)

```bash
# 1) Initialize a repo
./target/release/mini-git init
# -> creates .minigit/, .minigit/objects/, and .minigit/index.json

# 2) Create a couple of files
echo "alpha" > a.txt
mkdir -p notes && echo "beta" > notes/b.txt

# 3) Stage both a single file and a directory (recursive)
./target/release/mini-git add a.txt notes

# 4) Inspect results
cat .minigit/index.json            # path -> blob_id (40 hex chars)
ls -1 .minigit/objects            # files named by SHA-1 of file bytes
```

You should see `index.json` mapping each path to a 40-hex blob id, and `.minigit/objects/` containing a file for each unique blob id.

---

## CLI Usage

```
mini-git init
    Initialize a mini-git repo in the current directory (.minigit/).

mini-git add <files-or-dirs>
    Stage files or directories (recursively).
    Updates .minigit/index.json and writes blobs to .minigit/objects/.
```

**Examples**

```bash
# stage a single file
mini-git add README.md

# stage a directory (recursively)
mini-git add src
```

---

## Project Layout on Disk

```
.
├─ a.txt
├─ notes/
│  └─ b.txt
└─ .minigit/
   ├─ objects/          # content-addressed blobs (filename is the SHA-1)
   └─ index.json        # staging area: { "path": "<blob-id>", ... }
```

---

## Verify It Works (Sanity Checks)

**Check that the blob filename equals the file’s hash:**

```bash
shasum a.txt                          # macOS/Linux prints "<hash>  a.txt"
ls .minigit/objects                   # should list a file named <hash>
```

**Edit and stage again → a new blob appears, index updates:**

```bash
echo "more" >> a.txt
mini-git add a.txt
ls -1 .minigit/objects | wc -l        # count should increase by 1
cat .minigit/index.json               # a.txt now maps to the NEW hash
```

---

## Next Steps / Roadmap

* **`commit -m "<msg>"`**
  Save a snapshot of the index plus metadata (parent, timestamp).
* **`log`**
  Print commits from HEAD backward (follow parent links).
* **`status`**
  Compare working directory vs index vs last commit.
* **`checkout <commit>`**
  Restore files from a snapshot.
* **Compression**
  Store blobs compressed (e.g., zstd) to save space.
* **Refactor storage**
  Replace `index.json` with structured refs, per-object files, `HEAD` ref, etc.

*(These are perfect stretch goals to showcase deeper systems understanding.)*

---

## Troubleshooting

* **`Not a mini-git repo (missing .minigit)...`**
  Run `mini-git init` first in the directory.

* **Hash doesn’t change after edit**
  Make sure you **re-run `mini-git add <file>`** after editing. The index only updates when you add again.

* **Where is the hash coming from?**
  It’s the **SHA-1 of the file’s bytes**. You can verify with `shasum file`.

* **Index looks wrong**
  Open `.minigit/index.json` and confirm it maps the path to a 40-char lowercase hex string.

---


---

