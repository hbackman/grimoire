import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {parse}  from "node-html-parser";

// Request queue to handle concurrent requests
let requestQueue = [];
let isProcessing = false;

export async function scrape(url) {
  return new Promise((resolve, reject) => {
    // Add request to queue
    requestQueue.push({ url, resolve, reject });

    // Process queue if not already processing
    processQueue();
  });
}

async function processQueue() {
  if (isProcessing || requestQueue.length === 0) {
    return;
  }

  isProcessing = true;

  while (requestQueue.length > 0) {
    const { url, resolve, reject } = requestQueue.shift();

    try {
      const data = await scrapeInternal(url);
      resolve(data);
    } catch (error) {
      reject(error);
    }
  }

  isProcessing = false;
}

async function scrapeInternal(url) {
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

/**
 * Extract addons from a given HTML string. This does the heavy lefting
 * of scraping the relevant information from the Curseforge response.
 *
 * @param {string} html
 *
 * @returns {{
 *  image: string,
 *  title: string,
 *  description: string,
 *  author: string,
 * }[]}
 */
async function extractAddonsFromHtml(html) {
  return parse(html).querySelectorAll(".project-card")
    .map(e => {
      let name = e.querySelector(".name")
        .getAttribute("href")
        .split("/");
      name = name[name.length - 1];

      return {
        name,
        image:       e.querySelector(".art img").getAttribute("src"),
        title:       e.querySelector(".name").text.trim(),
        description: e.querySelector(".description").text.trim(),
        author:      e.querySelector(".author").text.trim(),
      };
    });
};

export async function browse(options) {
  const page   = options.page     ?? 1;
  const size   = options.size     ?? 20;
  const search = options.search   ?? null;

  const data = await scrape(
    search
      ? `https://www.curseforge.com/wow/search?page=${page}&pageSize=${size}&sortBy=relevancy&search=${search}`
      : `https://www.curseforge.com/wow/search?page=${page}&pageSize=${size}&sortBy=relevancy&class=addons`
  );
  return extractAddonsFromHtml(data.html);
};

export async function download(addon, version) {
  version = 79434;
  console.log(version);

  const url = `https://www.curseforge.com/wow/addons/${addon}/files/all?page=1&pageSize=20&gameVersionTypeId=${version}`;
  const data = await scrape(url);

  const file = parse(data.html)
    .querySelector(".file-card")
    .getAttribute("href");

  console.log(file);
};

export function versions() {
  return [{
    label: "Retail",
    value: 517,
  }, {
    label: "MoP Classic",
    value: 79434,
  }, {
    label: "Classic",
    value: 67408,
  }];
};
