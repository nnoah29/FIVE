# FIVE

**F**ile **I**nterface & **V**iew **E**xplorer: a fast, native file manager for Linux, written in Rust with GTK 4.

FIVE shows the folders of the current location as cards, the files as a list or a grid, and keeps
a preview and a disk-usage panel one click away. It is not a web page in a window: the engine is
Rust, the interface is GTK 4 (gtk4-rs), and everything runs locally.

## Features

- Folder cards, list and grid views, sorting, type filters, hidden files, thumbnails.
- Multi-selection, copy / cut / paste with a transfer queue (pause, resume, cancel).
- Rename one file or many at once (replace, regular expression, template).
- Zip and unzip, archive contents browsing.
- Quick look (`Space`), preview panel, disk usage analysis, duplicate finder, Git status marks.
- Command palette (`Ctrl+P` / `Ctrl+K`), search, recent files, favorites, network, trash, mounted disks.
- **Pinned folders** in the sidebar and a **bottom bar** of pinned folders and files, chosen by you
  (right-click any item to pin or unpin it).
- **Automation rules**: when a file is created, modified or stable, on a schedule, at start or on mount,
  run actions (move, rename, notify, pin to the sidebar...). Every new or modified rule runs as a trial first,
  and every run is journaled and can be undone. See [docs/REGLES.md](docs/REGLES.md) (French).
- **System file chooser**: FIVE can replace the "Open" and "Save as" dialogs of your browser and other applications (see below).
- Light and dark themes, accent color following your wallpaper, compact density.

## Install

Pre-built packages are attached to each [release](https://github.com/nnoah29/FIVE/releases).
FIVE needs **GTK 4.16 or newer** and **Pango 1.56 or newer** (and `poppler-utils` for PDF previews, recommended by the packages), which means:

| Distribution | Supported versions | Package |
| --- | --- | --- |
| Arch Linux | current | `five-*-x86_64.pkg.tar.zst` |
| Debian | 13 (trixie) and newer | `five_*_amd64.deb` |
| Ubuntu | 25.04 and newer | `five_*_amd64.deb` |
| Fedora | 42 and newer | `five-*.x86_64.rpm` |
| Any other x86_64 Linux with GTK 4.16 | | `five-*-linux-x86_64.tar.gz` |

```bash
# Arch Linux
sudo pacman -U five-*-x86_64.pkg.tar.zst

# Debian, Ubuntu
sudo apt install ./five_*_amd64.deb

# Fedora
sudo dnf install ./five-*.x86_64.rpm

# Generic tarball (installs in ~/.local, or PREFIX=/usr/local with sudo)
tar xzf five-*-linux-x86_64.tar.gz && ./five-*-linux-x86_64/install.sh
```

Then start **FIVE** from your application menu, or run `five-gtk`.
Ubuntu 24.04 LTS and Debian 12 ship an older GTK 4 and are not supported yet.

## Use FIVE as your file chooser

Applications ask the desktop portal (`xdg-desktop-portal`) for the dialog that picks a file, a folder or a save location.
FIVE ships a small, simple window for that: a list of folders and files, an editable path, your usual and pinned folders,
file type filters, a name field and an overwrite confirmation. It is opt-in. Packages install the portal files; to turn it on:

```bash
mkdir -p ~/.config/xdg-desktop-portal
printf '[preferred]\norg.freedesktop.impl.portal.FileChooser=five;gtk\n' >> ~/.config/xdg-desktop-portal/portals.conf
systemctl --user restart xdg-desktop-portal
```

If the file already has a `[preferred]` section, add the `FileChooser` line to it instead. `gtk` stays as a fallback.
To go back, remove that line. You can also try the window alone: `five-gtk --chooser open` (or `folder`, `save --name report.pdf`);
it prints the chosen paths. Keys: `Enter` or double click to validate, `Ctrl+L` to type a path, `Alt+Up` for the parent folder, `Ctrl+H` for hidden files, `Esc` to cancel.

## Build from source

Requirements: a recent stable Rust toolchain (edition 2024), `pkg-config` and the GTK 4 development files
(`libgtk-4-dev` on Debian and Ubuntu, `gtk4-devel` on Fedora, `gtk4` on Arch).

```bash
git clone https://github.com/nnoah29/FIVE.git
cd FIVE
cargo build --release -p five-gtk
./target/release/five-gtk                 # starts in your home folder
./target/release/five-gtk --root ~/Music  # confines FIVE to a folder
```

Development checks (formatting, clippy, build, tests, code norms): `./ci.sh`.
Packages are built in the same containers as the CI: `packaging/build-in-container.sh deb|rpm|arch`.

## Files and settings

| File | Content |
| --- | --- |
| `~/.config/five/reglages.json` | Settings, sidebar folders, bottom bar |
| `~/.config/five/session.json` | Last location |
| `~/.config/five/regles.json` | Automation rules |
| `~/.config/five/journal-regles.jsonl` | Journal of rule runs (used for undo) |

## Repository layout

- `engine/`: `five-engine`, the Rust engine (listing, search, transfers, zip, trash, rules).
- `native/`: `five-gtk`, the GTK 4 application.
- `packaging/`: deb, rpm, Arch and tarball builds, desktop entry, icon.
- `tools/check-norms`: code norms checker (function length, one doc line per function...).
- `docs/`: design notes and decisions (French).

## License

FIVE is released under the [MIT License](LICENSE).

## Status

FIVE is young (0.x): expect rough edges. Issues and ideas are welcome on GitHub.
