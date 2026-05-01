import { listen } from "@tauri-apps/api/event";

const mic = document.getElementById("mic");

listen<string>("language-mode", (event) => {
  if (!mic) return;
  mic.classList.toggle("urdu", event.payload === "ur");
});
