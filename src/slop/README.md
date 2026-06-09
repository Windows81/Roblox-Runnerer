# ROBLOX (2016) WindowsClient — Rust port

An idiomatic Rust translation of the `WindowsClient` directory from
[`Artifaqt/ROBLOX2016`](https://github.com/Artifaqt/ROBLOX2016/tree/e0cfac59fea3a5b986843e65b0fda286e439f9fc/WindowsClient)
(commit `e0cfac5`), built on the [`windows`](https://crates.io/crates/windows)
crate, with **all anti-cheat, VMProtect, and Authenticode-verification code
removed** as requested.

> The C++ source lives directly in `WindowsClient/` (there is no `src_cpp`
> subdirectory in that tree). The originals are kept under `_cpp_reference/`
> for side-by-side comparison.

## What this is (and isn't)

This client is a thin Win32/MFC/COM shell over the ROBLOX game engine
(`v8datamodel`, `gfxbase`, `network`, `script`, `reflection`, `util`, …). None
of that engine source is in the `WindowsClient` directory, so a *fully
compiling* binary is impossible from this directory alone. Per the chosen
approach, this port:

- Translates the Win32/MFC/COM logic into **idiomatic Rust** (the `windows`
  crate replaces ATL `CWindowImpl`/`CAxDialogImpl`/`CComPtr`, etc.).
- Funnels every ROBLOX-engine call through a single boundary module,
  `src/rbx/`, expressed as Rust traits + stub functions. Link these against the
  real engine to produce a working binary.
- See **`EXTERNAL_METHODS.md`** for the complete list of symbols called but not
  defined in `WindowsClient` (requirement 3).

Because the engine and several SDKs (DirectInput, VMProtect) are absent,
`cargo build` will **not** link a runnable `.exe` as-is — the `src/rbx/` stubs
type-check and document the surface, and engine bodies are `todo!()`. This is
expected and was agreed up front.

## Module map

| Rust module | From | Notes |
|---|---|---|
| `src/main.rs` | `main.cpp` | `WinMain`, window class, `WndProc` |
| `src/app.rs` | `Application.h/.cpp` | top-level app; **heaviest anti-cheat removal** |
| `src/document.rs` | `Document.h/.cpp` | game state, join-script execution |
| `src/view.rs` | `View.h/.cpp` | viewport, fullscreen/resolution, placement |
| `src/render_job.rs` | `RenderJob.h/.cpp` | render job (speedhack/debugger block removed) |
| `src/user_input.rs` | `UserInput.h/.cpp` | DirectInput mouse/keyboard routing |
| `src/function_marshaller.rs` | `FunctionMarshaller.h/.cpp` | cross-thread closure marshalling |
| `src/game_verbs.rs` | `GameVerbs.h/.cpp` | leave/screenshot/record/fullscreen verbs |
| `src/teleporter.rs` | `Teleporter.h/.cpp` | teleport callback |
| `src/rbx_web_view.rs` | `RbxWebView.h/.cpp` | in-game IE/ActiveX browser dialog |
| `src/web_browser_ax_dialog.rs` | `WebBrowserAxDialog.h/.cpp` | screenshot/video upload dialog |
| `src/resource.rs` | `resource.h` | resource IDs |
| `src/rbx/` | *(engine boundary)* | traits/stubs for everything external |
| `build.rs` | `.vcxproj` RC step | compiles `WindowsClient.rc` if present |

`stdafx.h/.cpp` (precompiled-header includes) and `InitializationError.h`
(folded into `app::InitError`) have no separate module.

## Removed: anti-cheat, VMProtect, and signature verification

### Files deleted wholesale
- **`functionHooks.cpp/.h`** — hotpatch/IAT hooking used to detect injected DLLs.
- **`robloxHooks.cpp` + `RobloxHooks.h`** — `FindWindowA` splice + VEH/
  `KiUserExceptionDispatcher` hooks that flagged debuggers/hooks
  (`HATE_VEH_HOOK`, `HATE_DLL_INJECTION`).
- **`RandomPadding.cpp`** — junk-code padding (`junk<>` / `RBX_BUILDSEED`) that
  shuffled the binary layout to defeat fixed-address exploits.
- **`ReleasePatcher.cpp/.h`** — the "golden hash patcher": NetPmc challenge
  generation, `.text`/`.rdata` code-cave patching (`writecopyTrap`), and the
  self-relaunch (`patchMain`) that rewrote the signed binary.
- **`Crypt.cpp/.h`** — `WinVerifyTrust`/`CryptQueryObject` Authenticode
  verification that the binary was signed by "ROBLOX Corporation" (removed per
  your choice to strip signature checks too).

### Surgical removals inside retained files
- `Application.cpp`: `vmProtectedDetectCheatEngineIcon`, `RBX::isSandboxie`,
  `ProgramMemoryChecker`/`pmcHash` hashing, `protectVmpSections`, `hookApi` +
  VEH wiring + `setupCeLogWatcher`, `setWindowFrame`→`VerifyCryptSignature`,
  all `HATE_*` / `sendStats` token plumbing, `CollectMd5Hash` self-hash, and the
  obfuscated `--waitEvent` keys that invoked `patchMain`. `SetProcessDEPPolicy`
  (DEP) is **kept** as ordinary OS hardening.
- `Document.cpp`: the `VMProtectIsDebuggerPresent` → `HATE_DEBUGGER` hack-flag
  blocks and the VMProtect include.
- `RenderJob.cpp`: the `VMProtectBeginMutation("34")` block running
  `Time::isSpeedCheater()`/`Time::isDebugged()` and `reportHacker(...)`.
- `main.cpp`: the final `VirtualProtect(RBX::Security::rbxVmpBase, …)`.

Every removal is marked with a `// [removed]` comment at the corresponding site
in the Rust source.

## Building (once an engine backend exists)

```sh
cargo build --release   # produces RobloxPlayerBeta(.exe)
```

Provide real implementations for the `src/rbx/` traits/functions (or link the
ROBLOX engine through FFI), drop `WindowsClient.rc` + its assets next to
`build.rs` for icons/dialogs/accelerators, and supply a DirectInput8 backend in
`user_input::dinput`.
