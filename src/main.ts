import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./styles.css";

type TabId = "portal" | "panel";

const TABS: TabId[] = ["portal", "panel"];

const appWindow = getCurrentWindow();
const tabButtons = Array.from(document.querySelectorAll<HTMLButtonElement>(".tab"));

let activeTab: TabId = "portal";

function selectTab(tab: TabId) {
  activeTab = tab;
  for (const button of tabButtons) {
    const selected = button.dataset.tab === tab;
    button.classList.toggle("is-active", selected);
    button.setAttribute("aria-selected", String(selected));
  }
}

function runCommand(command: string, args?: Record<string, unknown>) {
  void invoke(command, args).catch((error) => {
    console.error(`${command} failed`, error);
  });
}

async function switchTab(tab: TabId) {
  if (tab === activeTab) {
    return;
  }
  selectTab(tab);
  try {
    await invoke("activate_tab", { tab });
  } catch (error) {
    console.error("activate_tab failed", error);
  }
}

for (const button of tabButtons) {
  button.addEventListener("click", () => {
    void switchTab(button.dataset.tab as TabId);
  });
}

document.getElementById("reload")!.addEventListener("click", () => {
  runCommand("reload_active_site", { tab: activeTab });
});

document.getElementById("external")!.addEventListener("click", () => {
  runCommand("open_active_site_in_browser", { tab: activeTab });
});

document.getElementById("minimize")!.addEventListener("click", () => {
  void appWindow.minimize();
});

document.getElementById("maximize")!.addEventListener("click", () => {
  void appWindow.toggleMaximize();
});

document.getElementById("close")!.addEventListener("click", () => {
  void appWindow.close();
});

void invoke<string>("initial_tab")
  .then((tab) => {
    if ((TABS as string[]).includes(tab)) {
      selectTab(tab as TabId);
    }
  })
  .catch((error) => {
    console.error("initial_tab failed", error);
  });
