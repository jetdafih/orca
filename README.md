# orca

Two things live here:

- **`orca.html`** — a standalone, single-file click-GUI mockup. Open it directly in a browser; no build step, no server, no dependencies beyond the Google Fonts stylesheet it links. Press **Delete** to show/hide the window.
- **`echo/`** — the editable Echo source candidate (Java + resources, native Rust/JNI projects, patched third-party sources, and the pinned Gradle/native build inputs). This directory is source only: no build outputs, Gradle caches, or generated artifacts are committed.

## Building `echo/`

Tested host: macOS ARM64. Install **JDK 21**, **Python 3**, Rust **1.97.1**, and Xcode command-line tools. Gradle 9.4 and the public Maven/Cargo dependencies are fetched from pinned, checksum-verified sources.

```sh
cd echo
rustup toolchain install 1.97.1
python3 tools/build_candidate.py
```

The builder compiles the application, builds the native libraries from source, remaps with Loom, and writes a hash-bound candidate plus provenance under `echo/build/candidate/`. See `echo/README.md` for the full build contract and the known scope limits.

The four local stages, if you want to run them individually:

```sh
python3 tools/with_heavy_tool.py -- ./gradlew --no-daemon --max-workers=1 prepareCandidate
python3 tools/build/remap_closure.py
python3 tools/with_heavy_tool.py -- ./gradlew --no-daemon --max-workers=1 -p verification-remap candidateJar
python3 tools/build/publish_candidate.py
```

> **Platform note:** the compile and remap stages also run on Linux and produce
> `echo/build/candidate/echo-clean-candidate-intermediary.jar`. The final
> provenance/publish stage is authored for macOS ARM64 native receipts
> (`tools/build/publish_candidate.py` and `tools/build/artifact.py` hardcode
> `macos-aarch64` paths and Mach-O validation), so it will stop there on other
> hosts. The rebuilt artifact is an **unapproved verification candidate**, not a
> release, and has not been runtime-tested unless its own provenance says so.
