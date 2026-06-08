Some time ago, I discovered that Studio's code is almost a superset of RCC and Player.

This was determined by comparing the FFlag variables in version 463 of [Studio](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-studio.json) with [Player](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-player.json) and [RCC](https://github.com/Windows81/Roblox-x64dbg-FFlag-Extractor/blob/main/test/v463-server.json). Almost all flags which exist in `RobloxPlayerBeta.exe` also exist in `RobloxStudioBeta.exe` and many flags which exist in `RCCService.exe` also exist in `RobloxStudioBeta.exe`.

In addition, [the version-548 debug symbols](https://www.mediafire.com/file/b7b2ybzv9b25yzh/win_studio_x64_0.550.488.5480525.rar/file) tell us a lot of how Rōblox was compiled.

As of 2026-06-06, I was able to compile an `exe` which loads Studio in its own memory. It basically does the opposite of DLL injection.

And with the debug symbols, I will be able to triangulate exactly which functions to call to instantiate servers.

Basically, I only needed to patch like 6 or 7 bytes.

And I wrote a routine which does that for v548. This should work in all VC++-native executables. So essentially, I will be able to execute specific Studio components without needing to fully load Qt.
