import { client, configs } from "../index.js";

client.on("messageCreate", async (msg) => {
    const config = configs.get(msg.guildID)
    if(!config) return
    const role = config.messageRoleChannels[msg.channel.id]
    if(!role) return
    await msg.member.addRole(role)
})