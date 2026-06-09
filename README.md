As of 2026-06-06, I was able to compile an `exe` which loads Studio in its own memory. It basically does the opposite of DLL injection.

I'm basically turning `RobloxStudioBeta.exe` into a DLL and loading the entire file as if it were a DLL.

And with the debug symbols, I will be able to triangulate exactly which functions to call to instantiate servers.

Basically, I only needed to patch like 6 or 7 bytes.

And I wrote a routine which does that for v548. This should work in all VC++-native executables. So essentially, I will be able to execute specific Studio components without needing to fully load Qt.

I'm loading executable memory and would like a way to call Rōblox functions natively; it's a little bit like an injector in reverse.

I'm trying to avoid using `VirtualProtect` and instead opt to re-implement any code that needs to change.

## Why?

Some time prior, I discovered that Studio's code is almost a superset of RCC and Player.

This was determined by comparing the FFlag variables in version 463 of [Studio](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-studio.json) with [Player](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-player.json) and [RCC](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-server.json).

Almost all flags which exist in `RobloxPlayerBeta.exe` also exist in `RobloxStudioBeta.exe` and many flags which exist in `RCCService.exe` also exist in `RobloxStudioBeta.exe`.

Rōblox, versions 463 and previous (up to 2021-01-25), can be hosted on special `RCCService.exe` files, which people can access publicly as of 2026.

However, it's known that revivals _after_ v463 can use Studio.

Studio has been using Qt for a long time; **my goal is to minimise the memory footprint and maximise control**.

**Even better!** [The version-548 debug symbols](https://www.mediafire.com/file/b7b2ybzv9b25yzh/win_studio_x64_0.550.488.5480525.rar/file) tell us a lot of how Rōblox was compiled.

## Hosting?

Rōblox has two modes of hosting:

1. "Unsecured" sessions for Studio
2. "Secure" sessions for Rōblox Player

I think that "unsecured" sessions skip any client-based signature checks and stuff.

In Studio, there is a "Start Server" feature that allows you to test multiple players in a single game.

Look for `RBX::UnsecuredStudioGame` and `RBX::SecurePlayerGame` in [the 2016 source code](https://github.com/Artifaqt/ROBLOX2016/blob/e0cfac59fea3a5b986843e65b0fda286e439f9fc/iOS/SharedCode/PlaceLauncher.mm#L901).

### Why in Rust?

I'm using Rust because it's easier to grab third-party libraries. Yes! Also, you can compile my work more easily!

Why not Python? Speed isn't the issue. I need code that compiles to native x86_64.
