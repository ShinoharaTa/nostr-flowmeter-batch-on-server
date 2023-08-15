import dotenv from "dotenv";
import cron from "node-cron";
import { format, startOfMinute, subMinutes, getUnixTime } from "date-fns";
import { finishEvent, relayInit } from "nostr-tools";
import "websocket-polyfill";
import { upsertTableOrCreate } from "nostr-key-value";

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
    pub.on("ok", (ev) => {
      console.log(ev);
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
      let count = 0;
      const sub = relay.sub([
        { kinds: [1], since: getUnixTime(from), until: getUnixTime(to), limit: 1000 },
      ]);
      sub.on("event", (ev) => {
        count++;
      });
      return new Promise((resolve) => {
        sub.on("eose", () => {
          resolve(count);
        });
        relay.on("error", () => {
          resolve(NaN);
        });
      });
    } catch (error) {
      console.error(to, error);
      return Promise.resolve(NaN);
    }
  })();
  const formattedNow = format(from, "yyyyMMddHHmm");
  const formattedDate = format(from, "yyyyMMdd");
  const postData = [formattedNow, count.toString()];
  const ev_now = await upsertTableOrCreate(
    [RELAY_URL],
    HEX,
    `frowmeter_${RELAY_NAME}`,
    `frowmeter_${RELAY_NAME}`,
    [],
    [postData]
  );
  const ev_date = await upsertTableOrCreate(
    [RELAY_URL],
    HEX,
    `frowmeter_${RELAY_NAME}_${formattedDate}`,
    `frowmeter_${RELAY_NAME}_${formattedDate}`,
    [],
    [postData]
  );
  console.log(ev_now);
  console.log(ev_date);
  if (!MODE_DEV) {
    await post(ev_now);
    await post(ev_date);
  }
  relay.close();
  return;
};

// テスト処理実行
if (MODE_DEV) {
  submitNostrStorage();
} else {
}

// Schedule Batch
cron.schedule("* * * * *", async () => {
  if (MODE_DEV) return;
  submitNostrStorage();
});

// restart to AM 4:28
cron.schedule("28 4 * * *", async () => {
  if (MODE_DEV) return;
  console.log("restart");
  process.exit();
});
