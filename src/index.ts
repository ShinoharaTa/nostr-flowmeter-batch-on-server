import dotenv from "dotenv";
import cron from "node-cron";
import { format, startOfMinute, subMinutes, getUnixTime } from "date-fns";
import { finishEvent, relayInit } from "nostr-tools";
import "websocket-polyfill";
import { upsertTableOrCreate } from "nostr-key-value";
import { eventKind, NostrFetcher } from "nostr-fetch";

dotenv.config();

const MODE_DEV = process.argv.includes("--dev");
const RELAY_URL = process.env.RELAY_URL ?? "";
const RELAY_NAME = process.env.RELAY_NAME ?? "";
const HEX = process.env.HEX ?? "";

const relay = relayInit(RELAY_URL);

const post = async (ev: any) => {
  const post = finishEvent(ev, HEX);
  return new Promise((resolve) => {
    const pub = relay.publish(post);
    pub.on("failed", (ev) => {
      console.error("failed to send event", ev);
      resolve("ok");
    });
    pub.on("ok", () => {
      resolve("ok");
    });
  });
};

const submitNostrStorage = async (): Promise<null> => {
  const to = startOfMinute(new Date());
  const from = subMinutes(startOfMinute(new Date()), 1);
  try {
    await relay.connect();
  } catch (error) {
    console.error(to, error);
    return;
  }
  const count = await (async () => {
    try {
      const fetcher = NostrFetcher.init();
      const allPosts = await fetcher.fetchAllEvents(
        [RELAY_URL],
        { kinds: [eventKind.text] },
        { since: getUnixTime(from), until: getUnixTime(to) },
        { sort: true }
      );
      return allPosts ? allPosts.length : NaN;
    } catch (error) {
      return NaN;
    }
  })();
  const formattedNow = format(from, "yyyyMMddHHmm");
  const formattedDate = format(from, "yyyyMMdd");
  const postData = [formattedNow, count.toString()];
  const ev_now = await upsertTableOrCreate(
    [RELAY_URL],
    HEX,
    `flowmeter_${RELAY_NAME}`,
    `flowmeter_${RELAY_NAME}`,
    [],
    [postData]
  );
  const table_info = ev_now.tags.slice(0, 3);
  const table_data = ev_now.tags.slice(3).slice(-1440);
  ev_now.tags = [...table_info, ...table_data];
  const ev_date = await upsertTableOrCreate(
    [RELAY_URL],
    HEX,
    `flowmeter_${RELAY_NAME}_${formattedDate}`,
    `flowmeter_${RELAY_NAME}_${formattedDate}`,
    [],
    [postData]
  );
  console.log(ev_now);
  console.log(ev_date);
  await post(ev_now);
  await post(ev_date);
  if (!MODE_DEV) {
  }
  relay.close();
  return;
};

// テスト処理実行
if (MODE_DEV) {
  // submitNostrStorage();
} else {
}

// Schedule Batch
cron.schedule("* * * * *", async () => {
  // if (MODE_DEV) return;
  submitNostrStorage();
});

// restart to AM 4:28
cron.schedule("28 4 * * *", async () => {
  if (MODE_DEV) return;
  console.log("restart");
  process.exit();
});
