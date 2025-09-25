<p align="center" style="color:grey">

 <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./app/public/logo_dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="./app/public/logo.svg">
    <img alt="Logo" src="./app/public/logo.svg" style="transform: rotate(-90deg);">
  </picture>

<div align="center">
<table>
<tbody>
<td align="center">
<img width="2000" height="0"><br>

##### `"This app is still in early development but aims to help you organize your files, your emails, and your tasks as efficient as possible."`

![GitHub Stars](https://img.shields.io/github/stars/emilyantosch/hestia_tauri?style=social) ![GitHub Forks](https://img.shields.io/github/forks/emilyantosch/hestia_tauri?style=social) ![GitHub Issues](https://img.shields.io/github/issues/emilyantosch/hestia_tauri) ![License: MIT](https://img.shields.io/badge/license-MIT-green)

<img width="2000" height="0">
</td>
</tbody>
</table>
</div>
</p>

# Hestia

Hestia is a **cross-platform file management system** that uses intelligent tagging to organize your files without forcing you to reorganize your existing folder structure. **Built with Rust** for speed and security, it features visual thumbnails for images, videos, PDFs, and documents, plus plans for email, password, and task management integration. Designed **offline-first** with cross-device sync capabilities, Hestia keeps your data secure and accessible across Windows, macOS, Linux, and mobile platforms.

## Features

- **Modular by Design:** Each feature and plugin is its own module. Mix, match, or extend as you need.
- **Multiple Configurations:**
  - **Bare:** Minimal setup for fast, distraction-free editing and server use.
  - **Core:** Daily-driver configuration with LSP and language support.
  - **Full:** Comprehensive environment including advanced plugins (LaTeX, etc.).
- **Easy Customization:** Import only what you want, or override settings in `config.nvix`.
- **Flake Native:** Built for the modern Nix ecosystem.

## Quickstart

Clone the repo, `cd` into the directory, install dependencies via:

```sh
deno install
```

and run:

```sh
deno task tauri dev
```

This should compile the rust code base and start the development server as well as the app itself.

> [!NOTE]
> Now no need for ~--accept-flake-config~ flag, as Nvix's all package are already cached in the nixos cache

## Installation

### Prerequisites

- [Nix Package Manager](https://nixos.org/download.html) (multi-user recommended)
- Flakes enabled (`experimental-features = nix-command flakes`)
- Familiarity with [NixOS](https://nixos.org/) or [Home Manager](https://nix-community.github.io/home-manager/) is helpful

**Install Nix (multi-user):**

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
    sh -s -- install --no-confirm --extra-conf "trusted-users = $(whoami)"
```

### Add Nvix as a flake input

## FAQs

**What is Nvix for?**
Nvix started as a Nix learning experiment and evolved into a robust, composable Neovim configuration framework.

**Why three configurations?**

- `bare`: Minimal, distraction-free
- `core`: Daily usage with LSP and language support
- `full`: Advanced workflows such as LaTeX

**Can I use just one module?**
Absolutely. Nvix is designed so you can import only what you need.

**Are contributions welcome?**
Yes! Docs, configs, plugins, suggestions—all are valued. Open a PR or issue.

**Why only some options in `config.nvix`?**
[Nixvim](https://github.com/nix-community/nixvim) already exposes most customization options. Nvix adds curated, ready-to-use modules.

### Contributing

> Just keep it simple, stupid! -> `kiss` design principle

You’re welcome to contribute in any way:

- Improve docs or fix typos
- Suggest features or plugin support
- Enhance language/LSP integration

> [!IMPORTANT]

```sh
git clone --single-branch --branch master https://github.com/niksingh710/nvix.git

# ssh
git clone --single-branch --branch master git@github.com:niksingh710/nvix.git
```

Please open an issue or PR with your ideas or improvements!

### License

This project is licensed under the [MIT License](./LICENSE).

### Acknowledgments

- Thanks to all the authors of the projects, libraries and crates that I have had the honour to use in the development of this app.

> [!NOTE]
> I started working Hestia as a personal project, because I didn't find what I wanted with other similar tools.
> If you also happen to find it useful, please consider starring the repository or open a pull request as per the Contributing section!
> Thank you so much for your time!

Have questions or suggestions? [Open an issue](https://github.com/emilyantosch/hestia_tauri/issues)—I’m always happy to help.
