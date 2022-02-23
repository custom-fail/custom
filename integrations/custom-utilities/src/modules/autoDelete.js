import { client, configs } from "../index.js";

const wait = (ms) => new Promise((r) => setTimeout(r, ms))

client.on("messageCreate", async (msg) => {
    const config = configs.get(msg.guildID)
    if(!config) return
    const time = config.autoDeleteChannels[msg.channel.id]
    if(!time) return
    await wait(time)
    await msg.delete()
})