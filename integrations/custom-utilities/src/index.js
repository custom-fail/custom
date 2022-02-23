import { Client } from "eris"
import "dotenv/config"

import "./handlers/modules.js"

export const client = new Client(process.env.DISCORD_TOKEN)

export const configs = new Map()
configs.set("898986393177567242", {
    autoPublisherChannels: ["946159295270096997"]
})

await client.connect()
console.log("Connected to discord api")