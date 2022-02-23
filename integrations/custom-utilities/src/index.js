import { Client } from "eris"
import "dotenv/config"

import "./handlers/modules.js"

export const client = new Client(process.env.DISCORD_TOKEN)

export const configs = new Map()
configs.set("898986393177567242", {
    
})

await client.connect()
console.log("Connected to discord api")