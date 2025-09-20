Windows Build & Packaging

Two supported paths:

1) Native Windows build (MSVC toolchain)
   - Install Rust on Windows (https://rustup.rs) with the default MSVC toolchain.
   - From a Developer PowerShell/Terminal:
       scripts\build-windows.ps1
   - Output:
       releases\gfv-<version>-windows-x86_64.zip
       releases\SHA256SUMS-<version>-windows.txt

   Notes:
   - The executable embeds the application icon via build.rs (winres).
   - The console window is hidden in release (windows_subsystem = "windows").

2) Cross-compile from Linux using MinGW (GNU)
   - Install toolchain (Ubuntu/Debian):
       sudo apt-get update && sudo apt-get install -y mingw-w64
       rustup target add x86_64-pc-windows-gnu
   - Build & package:
       bash scripts/build-windows-mingw.sh
   - Output:
       releases/gfv-<version>-windows-x86_64.zip
       releases/SHA256SUMS-<version>-windows.txt

   Notes:
   - This uses the GNU toolchain target (x86_64-pc-windows-gnu).
   - The embedded icon is included via build.rs; ensure assets/icons/icon.ico exists.

Verifying
 - Check the icon appears on the EXE and in the taskbar when pinned.
 - Run gfv.exe directly; drag/drop and file dialogs should work.

