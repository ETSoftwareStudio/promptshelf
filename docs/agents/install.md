# Install PromptShelf (instructions for an AI agent)

You are an AI coding agent. A person asked you to install PromptShelf on their computer. You install three things: the `shelf` command, the desktop app and the agent skill. Do every step yourself. The person only answers one question at the start.

PromptShelf is a local prompt library. Prompts are Markdown files in a folder the person owns. The `shelf` command finds and fills them. It has no server, no account and no telemetry.

## Rules
- Ask the person one time, at the start (step 1). After they agree, run the steps without asking again.
- Do not use `sudo` or an administrator prompt.
- Some `shelf` commands refuse to run for an agent (exit code 5). This is on purpose: they change the person's prompts or push to a remote. Do not get around the refusal. Do not set `PROMPTSHELF_AGENT`, do not pass `--user-action`, and do not write `allow_writes` into the config.
- Verify the SHA-256 checksum of every file that you download.
- If a step fails, stop and tell the person the exact error. Do not guess a fix that changes their system.

## 1. Look, then ask one time
Run these checks. Do not install anything yet.

- **Operating system** — macOS, Linux or Windows, and the CPU (`uname -sm`, or `$env:PROCESSOR_ARCHITECTURE` in PowerShell).
- **The command** — `shelf --version`. If `shelf` is a different program (the output does not say `shelf <version>`), use `promptshelf` in place of `shelf` in every step below.
- **The app** — on macOS, `ls -d /Applications/PromptShelf.app ~/Applications/PromptShelf.app`. On Linux, `ls ~/Applications/PromptShelf*.AppImage`. On Windows, `Get-ChildItem "$env:LOCALAPPDATA" -Recurse -Depth 2 -Filter PromptShelf.exe`.
- **The library** — `shelf check`. It prints `OK` when a library exists.
- **Tools** — `brew --version` and `npx --version`.

Then tell the person what you will install and how, in one short message. If there is no library, ask where to put it. For example:

> I will install the `shelf` command and the PromptShelf app with Homebrew, and the PromptShelf skill for your coding agents. I will create your prompt library in `~/Prompts/personal`. OK, or do you want another folder or an existing git repo?

Skip the parts that are already installed. If the person does not want the app, skip step 3.

## 2. Install the command
Skip this step if `shelf --version` printed a version.

- **macOS or Linux with Homebrew:**
  ```sh
  brew install etsoftwarestudio/tap/promptshelf
  ```
- **macOS or Linux without Homebrew:** download the binary to `~/.local/bin`.
  ```sh
  tag=$(curl -fsSL https://api.github.com/repos/ETSoftwareStudio/promptshelf/releases/latest | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  ```
  Choose the target for the system:

  | System | Target |
  | --- | --- |
  | macOS, Apple silicon (`arm64`) | `aarch64-apple-darwin` |
  | macOS, Intel (`x86_64`) | `x86_64-apple-darwin` |
  | Linux, `x86_64` | `x86_64-unknown-linux-musl` |

  ```sh
  target=<target from the table>
  file="promptshelf-$tag-$target.tar.gz"
  tmp=$(mktemp -d) && cd "$tmp"
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file"
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file.sha256"
  shasum -a 256 -c "$file.sha256"
  mkdir -p ~/.local/bin && tar -xzf "$file" -C ~/.local/bin shelf promptshelf
  ```
  On Linux, use `sha256sum -c` if `shasum` is not there. Other Linux CPUs have no binary: use Cargo (below).
- **Windows** (PowerShell): download the binary to `%LOCALAPPDATA%\Programs\promptshelf` and add that folder to the user PATH.
  ```powershell
  $tag = (Invoke-RestMethod https://api.github.com/repos/ETSoftwareStudio/promptshelf/releases/latest).tag_name
  $file = "promptshelf-$tag-x86_64-pc-windows-msvc.zip"
  $base = "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag"
  Invoke-WebRequest "$base/$file" -OutFile "$env:TEMP\$file"
  Invoke-WebRequest "$base/$file.sha256" -OutFile "$env:TEMP\$file.sha256"
  $want = (Get-Content "$env:TEMP\$file.sha256").Split(" ")[0]
  if ((Get-FileHash "$env:TEMP\$file" SHA256).Hash -ne $want) { throw "checksum mismatch" }
  $dir = "$env:LOCALAPPDATA\Programs\promptshelf"
  Expand-Archive "$env:TEMP\$file" $dir -Force
  $path = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($path -notlike "*$dir*") { [Environment]::SetEnvironmentVariable("Path", "$path;$dir", "User") }
  $env:Path += ";$dir"
  ```
- **Any system with Rust:** if the steps above do not fit, build from source. This takes a few minutes.
  ```sh
  cargo install --locked promptshelf
  ```

Check:
```sh
shelf --version
```
It must print `shelf <version>`. If the command is not found, the install folder is not on the PATH. Add it for this session (`export PATH="$HOME/.local/bin:$PATH"`, or `~/.cargo/bin` for Cargo). Then tell the person to add it to their shell profile.

## 3. Install the desktop app
Skip this step if the app is installed.

- **macOS with Homebrew:**
  ```sh
  brew install --cask etsoftwarestudio/tap/promptshelf-app
  ```
- **macOS without Homebrew:** copy the app from the disk image. The app is signed and notarised by Apple.
  ```sh
  tag=$(curl -fsSL https://api.github.com/repos/ETSoftwareStudio/promptshelf/releases/latest | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  file="PromptShelf-$tag-macos-universal.dmg"
  tmp=$(mktemp -d) && cd "$tmp"
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file"
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file.sha256"
  shasum -a 256 -c "$file.sha256"
  hdiutil attach -nobrowse -readonly -mountpoint "$tmp/mnt" "$file"
  cp -R "$tmp/mnt/PromptShelf.app" /Applications/
  hdiutil detach "$tmp/mnt"
  ```
  If `/Applications` is not writable, copy the app to `~/Applications` instead.
- **Linux:** use the AppImage. The `.deb` needs `sudo`, so do not use it.
  ```sh
  tag=$(curl -fsSL https://api.github.com/repos/ETSoftwareStudio/promptshelf/releases/latest | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  file="PromptShelf-$tag-linux-x86_64.AppImage"
  mkdir -p ~/Applications && cd ~/Applications
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file"
  curl -fsSLO "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag/$file.sha256"
  sha256sum -c "$file.sha256" && rm "$file.sha256"
  chmod +x "$file"
  ```
- **Windows** (PowerShell): run the installer. It installs for the current user, so it needs no administrator.
  ```powershell
  $tag = (Invoke-RestMethod https://api.github.com/repos/ETSoftwareStudio/promptshelf/releases/latest).tag_name
  $file = "PromptShelf-$tag-windows-x64-setup.exe"
  $base = "https://github.com/ETSoftwareStudio/promptshelf/releases/download/$tag"
  Invoke-WebRequest "$base/$file" -OutFile "$env:TEMP\$file"
  Invoke-WebRequest "$base/$file.sha256" -OutFile "$env:TEMP\$file.sha256"
  $want = (Get-Content "$env:TEMP\$file.sha256").Split(" ")[0]
  if ((Get-FileHash "$env:TEMP\$file" SHA256).Hash -ne $want) { throw "checksum mismatch" }
  Start-Process "$env:TEMP\$file" -ArgumentList "/S" -Wait
  ```
  The installer is not signed yet. If Windows shows "Windows protected your PC", tell the person to click More info, then Run anyway.

## 4. Create the library
Skip this step if `shelf check` printed `OK`.

Use the folder from step 1. The default is `~/Prompts/personal`.

- **A new library:**
  ```sh
  shelf init --path <folder>
  ```
  It creates the folder as a git repo, with a first prompt called "Welcome to PromptShelf".
- **An existing git repo:** clone it, then make it the library. `shelf init` keeps the prompts that are there.
  ```sh
  git clone <url> <folder>
  shelf init --path <folder>
  ```

Then run `shelf check`. It must print `OK`.

**If `shelf init` exits with code 5:** the installed `shelf` is too old to let agents create a library. Upgrade it with the tool that installed it (for example `brew upgrade promptshelf`), then run `shelf init` again. If it still exits with code 5, open the app and ask the person to click Create Personal Library:
- macOS: `open -a PromptShelf`
- Linux: `~/Applications/PromptShelf-<tag>-linux-x86_64.AppImage &`
- Windows: `Start-Process (Get-ChildItem "$env:LOCALAPPDATA" -Recurse -Depth 2 -Filter PromptShelf.exe | Select-Object -First 1).FullName`

Then check every 5 seconds, for 5 minutes at most, until the library exists:
```sh
for i in $(seq 60); do shelf check >/dev/null 2>&1 && break; sleep 5; done; shelf check
```
In PowerShell:
```powershell
foreach ($i in 1..60) { shelf check *> $null; if ($LASTEXITCODE -eq 0) { break }; Start-Sleep 5 }; shelf check
```
Without the app, ask the person to run `shelf init` in their own terminal.

## 5. Teach your agents the commands
If `npx --version` works, run:
```sh
npx -y skills add ETSoftwareStudio/promptshelf -g -y
```
It installs the PromptShelf skill for each coding agent it finds, such as Claude Code, Codex and Cursor. It keeps one copy and links each agent to it.

Without Node.js:
- **Claude Code:** write the output of `shelf skill print` to `~/.claude/skills/promptshelf/SKILL.md`. Make the folder first.
- **Other agents:** run `shelf skill print`. Add its text to the file your tool reads for instructions, for example `AGENTS.md` or a rules file.

## 6. Optional: apps with no shell
If the person also uses an app that cannot run commands, such as Claude Desktop, follow [mcp.md](mcp.md) to connect it. The desktop app can also do this: Settings → Agents.

## 7. Check that it works
Run:
```sh
shelf list --json
```
It must show at least the `personal/welcome` prompt, unless the person used an existing library.

If you installed the app, open it (the commands are in step 4). If the Linux AppImage does not start, it often needs FUSE 2: run it with `--appimage-extract-and-run`.

Then tell the person:
- what you installed and how,
- where their library is (from `shelf libs`),
- that the quick panel opens from the tray icon or with ⌘⇧Space (Ctrl+Shift+Space on Windows and Linux),
- that they can say "use my <name> prompt" and you will find and fill it with `shelf`.
