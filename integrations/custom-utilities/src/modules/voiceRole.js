import { client, configs } from "../index.js";

client.on("voiceChannelJoin", async (member) => {
    const config = configs.get(member.guild?.id)
    if(!config || !config.voiceRole) return
    await member.addRole(config.voiceRole)
})

client.on("voiceChannelLeave", async (member) => {
    const config = configs.get(member.guild?.id)
    if(!config || !config.voiceRole) return
    await member.removeRole(config.voiceRole)
})