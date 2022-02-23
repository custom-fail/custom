import { client } from "../index.js";

client.on("error", (err) => console.error(err))
client.on("warn", (warn) => console.warn(warn))