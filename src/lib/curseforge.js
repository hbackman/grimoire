import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {parse}  from "node-html-parser";

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

export async function browse(page = 1, size = 20) {
  const url = `https://www.curseforge.com/wow/search?page=${page}&pageSize=${size}&sortBy=relevancy&class=addons`;
  const data = await scrape(url);
  const root = parse(data.html);

  return root.querySelectorAll(".project-card")
    .map(e => {
      return {
        image:       e.querySelector(".art img").getAttribute("src"),
        name:        e.querySelector(".name").text.trim(),
        description: e.querySelector(".description").text.trim(),
        author:      e.querySelector(".author").text.trim(),
      };
    });
}
