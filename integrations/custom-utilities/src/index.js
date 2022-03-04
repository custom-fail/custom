import { Client } from "eris"
import "dotenv/config"

import "./handlers/modules.js"

export const client = new Client(process.env.DISCORD_TOKEN)

export const configs = new Map()
configs.set("898986393177567242", {
    autoPublisherChannels: ["946159295270096997"],
    autoDeleteChannels: {
        "946179628207308810": 1 * 60 * 1000
    },
    messageRoleChannels: {
        "947886352203149462": "947886434323415051"
    },
    voiceRole: "946173995768774687"
})

await client.connect()
console.log("Connected to discord api")