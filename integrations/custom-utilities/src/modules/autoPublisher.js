import { client, configs } from "../index.js"

client.on("messageCreate", async (msg) => {
    if(msg.channel.type !== 5) return
    const config = configs.get(msg.guildID)
    if(!config || !config.autoPublisherChannels || !config.autoPublisherChannels.includes(msg.channel.id)) return
    await msg.crosspost()
})