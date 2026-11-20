# Professor Soly × Codex Auto Clicker

A simple Windows desktop tool for repeating a chosen keyboard key or mouse click. Built with Tauri and designed as a collaboration between Professor Soly and Codex.

## Use

- Choose **Keyboard** to capture a key, or **Mouse** to choose left, right, or middle click.
- Select single or double click in Mouse mode.
- Set the interval in seconds. The number box accepts 0.01 seconds through 24 hours; the slider covers 0.05 to 5 seconds.
- Press **Start**, or press **F6** anywhere to start or stop.
- Mouse clicks happen at the pointer's current position. Keyboard presses and clicks go to the currently focused application.

F6 is reserved for the global Start / Stop shortcut and cannot be selected as the repeated key. The chosen mode, key, click type, mouse button, and interval are saved on this PC.

## Portable Windows build

This project is configured to output the app executable without an MSI or NSIS installer. A Windows build is created with:

```powershell
npm install
npm run tauri -- build
```

The executable is written to `src-tauri/target/release/soly-auto-clicker.exe`. The ready-to-run copy is in [`portable/Professor-Soly-Auto-Clicker.exe`](portable/Professor-Soly-Auto-Clicker.exe); launch it directly without installing the app. Tauri uses the Microsoft Edge WebView2 Runtime available on the PC; Windows 10 and 11 usually include it.

## Development

```powershell
npm install
npm run tauri -- dev
```

The UI uses the Hamrah font supplied for this project.
