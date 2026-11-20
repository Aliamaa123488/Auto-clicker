import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

const $ = (selector) => document.querySelector(selector);

const controls = $("#config-controls");
const keyboardPanel = $("#keyboard-panel");
const mousePanel = $("#mouse-panel");
const keyChoice = $("#key-choice");
const keyValue = $("#key-value");
const intervalNumber = $("#interval-number");
const intervalSlider = $("#interval-slider");
const startButton = $("#start-button");
const stopButton = $("#stop-button");
const statusIndicator = $("#status-indicator");
const statusTitle = $("#status-title");
const statusDetail = $("#status-detail");
const notice = $("#notice");

let mode = localStorage.getItem("mode") || "keyboard";
let selectedKey = localStorage.getItem("key") || "";
let clickType = localStorage.getItem("clickType") || "single";
let running = false;
let capturingKey = false;

function displayKey(key) {
  const pretty = {
    " ": "Space",
    ArrowUp: "↑ Up",
    ArrowDown: "↓ Down",
    ArrowLeft: "← Left",
    ArrowRight: "→ Right",
    Escape: "Esc",
    Enter: "Enter",
    Backspace: "Backspace",
    Delete: "Delete",
    PageUp: "Page Up",
    PageDown: "Page Down",
    CapsLock: "Caps Lock",
    NumLock: "Num Lock",
    ScrollLock: "Scroll Lock",
    PrintScreen: "Print Screen",
    Control: "Ctrl",
    Meta: "Windows",
    ShiftLeft: "Left Shift",
    ShiftRight: "Right Shift",
    ControlLeft: "Left Ctrl",
    ControlRight: "Right Ctrl",
    AltLeft: "Left Alt",
    AltRight: "Right Alt",
    MetaLeft: "Left Windows",
    MetaRight: "Right Windows",
  };
  return pretty[key] || key;
}

function setNotice(message = "", kind = "") {
  notice.textContent = message;
  notice.className = `notice ${kind}`.trim();
}

function saveSettings() {
  localStorage.setItem("mode", mode);
  localStorage.setItem("key", selectedKey);
  localStorage.setItem("clickType", clickType);
  localStorage.setItem("interval", intervalNumber.value);
  localStorage.setItem("mouseButton", $("#mouse-button").value);
}

function setMode(nextMode) {
  if (running) return;
  mode = nextMode;
  const keyboardActive = mode === "keyboard";
  $("#keyboard-tab").classList.toggle("active", keyboardActive);
  $("#keyboard-tab").setAttribute("aria-selected", String(keyboardActive));
  $("#mouse-tab").classList.toggle("active", !keyboardActive);
  $("#mouse-tab").setAttribute("aria-selected", String(!keyboardActive));
  keyboardPanel.hidden = !keyboardActive;
  mousePanel.hidden = keyboardActive;
  saveSettings();
}

function setRunning(isRunning) {
  running = isRunning;
  controls.disabled = isRunning;
  startButton.disabled = isRunning;
  stopButton.disabled = !isRunning;
  statusIndicator.classList.toggle("running", isRunning);
  statusTitle.textContent = isRunning ? "Running" : "Ready when you are";
  statusDetail.textContent = isRunning
    ? "Press F6 or Stop to end the repeated actions."
    : "Focus the window you want to control, then press F6.";
}

function currentConfig() {
  const intervalSecs = Number(intervalNumber.value);
  if (!Number.isFinite(intervalSecs) || intervalSecs < 0.01 || intervalSecs > 86400) {
    throw new Error("Enter an interval between 0.01 and 86,400 seconds.");
  }
  if (mode === "keyboard" && !selectedKey) {
    throw new Error("Choose a keyboard key first.");
  }
  return {
    mode,
    key: mode === "keyboard" ? selectedKey : null,
    button: $("#mouse-button").value,
    clickType,
    intervalSecs,
  };
}

async function startAutomation() {
  if (running || capturingKey) return;
  try {
    setNotice("");
    const config = currentConfig();
    const started = await invoke("start_clicker", { config });
    if (started) setRunning(true);
  } catch (error) {
    setNotice(String(error), "error");
  }
}

async function stopAutomation() {
  try {
    await invoke("stop_clicker");
    setRunning(false);
    setNotice("");
  } catch (error) {
    setNotice(String(error), "error");
  }
}

async function toggleAutomation() {
  if (capturingKey) {
    setNotice("F6 is reserved for Start / Stop.", "hint");
    return;
  }
  try {
    setNotice("");
    const config = currentConfig();
    const nowRunning = await invoke("toggle_clicker", { config });
    setRunning(nowRunning);
  } catch (error) {
    setNotice(String(error), "error");
  }
}

document.querySelectorAll(".mode-tab").forEach((button) => {
  button.addEventListener("click", () => setMode(button.dataset.mode));
});

document.querySelectorAll(".segment").forEach((button) => {
  button.addEventListener("click", () => {
    if (running) return;
    clickType = button.dataset.clickType;
    document.querySelectorAll(".segment").forEach((segment) => {
      const active = segment === button;
      segment.classList.toggle("active", active);
      segment.setAttribute("aria-pressed", String(active));
    });
    saveSettings();
  });
});

keyChoice.addEventListener("click", () => {
  if (running) return;
  capturingKey = true;
  keyChoice.classList.add("capturing");
  keyValue.textContent = "Press any key…";
  setNotice("Press the key you want to repeat. F6 is reserved.", "hint");
  keyChoice.focus();
});

document.addEventListener("keydown", (event) => {
  if (!capturingKey) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "F6") {
    setNotice("F6 is reserved for Start / Stop. Choose a different key.", "hint");
    return;
  }
  if (event.key === "Dead" || event.key === "Unidentified") {
    setNotice("That key could not be identified. Try another key.", "error");
    return;
  }
  selectedKey = event.code.startsWith("Numpad") || /^(Shift|Control|Alt|Meta)(Left|Right)$/.test(event.code)
    ? event.code
    : event.key;
  keyValue.textContent = displayKey(selectedKey);
  capturingKey = false;
  keyChoice.classList.remove("capturing");
  setNotice("");
  saveSettings();
}, true);

document.addEventListener("pointerdown", (event) => {
  if (capturingKey && !keyChoice.contains(event.target)) {
    capturingKey = false;
    keyChoice.classList.remove("capturing");
    keyValue.textContent = selectedKey ? displayKey(selectedKey) : "Choose a key";
    setNotice("");
  }
});

intervalSlider.addEventListener("input", () => {
  intervalNumber.value = Number(intervalSlider.value).toFixed(2);
  saveSettings();
});

intervalNumber.addEventListener("input", () => {
  const value = Number(intervalNumber.value);
  if (Number.isFinite(value)) {
    intervalSlider.value = String(Math.min(5, Math.max(0.05, value)));
  }
  saveSettings();
});

$("#mouse-button").addEventListener("change", saveSettings);
startButton.addEventListener("click", startAutomation);
stopButton.addEventListener("click", stopAutomation);

const savedInterval = localStorage.getItem("interval");
if (savedInterval && Number.isFinite(Number(savedInterval))) {
  intervalNumber.value = savedInterval;
  intervalSlider.value = String(Math.min(5, Math.max(0.05, Number(savedInterval))));
}
const savedMouseButton = localStorage.getItem("mouseButton");
if (["left", "right", "middle"].includes(savedMouseButton)) {
  $("#mouse-button").value = savedMouseButton;
}
if (selectedKey) keyValue.textContent = displayKey(selectedKey);
document.querySelectorAll(".segment").forEach((button) => {
  const active = button.dataset.clickType === clickType;
  button.classList.toggle("active", active);
  button.setAttribute("aria-pressed", String(active));
});
setMode(mode === "mouse" ? "mouse" : "keyboard");
setRunning(false);

listen("clicker-state", ({ payload }) => setRunning(Boolean(payload)));
listen("automation-error", ({ payload }) => {
  setRunning(false);
  setNotice(String(payload), "error");
});
listen("hotkey-toggle", () => toggleAutomation());
