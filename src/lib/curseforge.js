import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

export async function scrape(url) {
  let unlistenOk;
  let unlistenErr;

  const resultP = new Promise(async (resolve, reject) => {
    unlistenOk = await listen("scraper:result", (evt) => {
      resolve(evt.payload);
    });

    unlistenErr = await listen("scraper:error", (evt) => {
      reject(new Error(evt.payload || "unknown error"));
    });
  });

  // Tell the backend to navigate the hidden window
  await invoke("scrape_in_webview", { url });

  const data = await resultP;

  // Clean up listeners
  if (unlistenOk) unlistenOk();
  if (unlistenErr) unlistenErr();

  return data;
}
