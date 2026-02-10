# NCD (Navigation Change Directory)

**NCD** is a high-speed directory jumper optimized for Windows development environments. It allows you to navigate complex directory structures with minimal keystrokes using a prioritized search pipeline and intelligent pattern matching.

One of the smartest features of the Netware shell was the elipses.It allowed fast navigation to any parent quickly. While hooking the kernel is available, the core routines are deigned to be OS independent, so this version echos the results back. This allows any os to utilize the tool using the command aliasing system inherit to most. 

## 🚀 Key Features

* **Intelligent Ellipsis (`...`):** Jump up multiple levels instantly. `...` goes up 2 levels, `....` goes up 3, and so on.
* **Context-Aware Jumps:** Automatically searches your Current Working Directory (CWD) and your defined `CDPATH` roots.
* **Wildcard & Globbing:** Support for `*` and `?` to find uniquely named project folders without typing the full path.
* **Drive Anchoring:** Resolve paths relative to the drive root or absolute paths seamlessly.
* **Zero-Friction Integration:** Designed to be wrapped in a shell function (like `function ncd { cd $(ncd.exe $@) }`) for instant directory switching.
* **Wildcard Sensing:** In 'fuzzy' mode it can match partial directories without needing the wildcards (default behavior is require wildcards)  

---

## 🛠 Installation & Setup

### 1. Build the Binary

```bash
cargo build --release

```

### 2. Shell Integration (PowerShell/CMD)

NCD outputs the resolved path to `stdout`. To actually change your directory, add a helper to your profile:

**PowerShell (`$PROFILE`):**

```powershell
function j {
    $target = & "ncd.exe" @args
    if ($lastExitCode -eq 0 -and $target) {
        # Set OLDPWD so 'ncd -' works in the next call
        $env:OLDPWD = Get-Location
        Set-Location $target
    }
}
```
```cmd
rem Due to the nuamces of cmd, the macro is best set via a .doskey file
cd=FOR /F "tokens=*" %i IN ('"ncd.exe" $*') DO @(set "OLDPWD=%CD%" & chdir /d "%i")        
```

---

## 📖 Usage Guide

| Command | Action |
| --- | --- |
| `ncd project` | Searches for "project" in CWD or `CDPATH` |
| `ncd pro*` | Wildcard search for directories starting with "pro" |
| `ncd ...` | Go up two levels (Parent of Parent) |
| `ncd ...\build` | Go up two levels, then down into the "build" folder |
| `ncd -` | Toggle back to the previous directory (`OLDPWD`) |
| `ncd ~` | Jump to your Home/UserProfile directory |

### Search Strategies (`--cd`)

* **Origin (Default):** Scans *inside* directories listed in your `CDPATH` (Classic Shell behavior).
* **Target:** Matches the folder name of the `CDPATH` entry itself (Bookmark behavior).
* **Hybrid:** Checks if the entry is the target; if not, scans inside.

---

## ⚙️ Environment Variables

* `CDPATH`: Semicolon-separated list of roots to search (e.g., `V:\Projects;C:\Users\Dev`).
* `NCD_MODE`: Set default strategy (`origin`, `target`, `hybrid`).
* `OLDPWD`: Maintained by your shell to support the `ncd -` toggle.

---

## ⚖️ License

**PolyForm Noncommercial 1.0.0 (Personal & Research Use Only)**

* **Commercial use is strictly prohibited** without a separate agreement.
* Redistribution is permitted provided the license notice remains intact.
* See `src/main.rs` for full license headers.

---

### 🧪 Technical Architecture

NCD is built with a non-blocking search engine that prioritizes results to prevent "FUBAR" jumps:

1. **Literal/Anchored Paths**
2. **Ellipsis Logic**
3. **CWD Context**
4. **CDPATH Search**

---

**TODO:** 
* Improve path separator handling (currently DOS/Win specific for output)
* Create a Win32 hooking binary (AttachProcess/SetWinEventHook/ConPTY)
* Create a Posix hooking binary (forkpty)