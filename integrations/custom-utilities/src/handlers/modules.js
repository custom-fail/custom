import { readdirSync } from "fs"

const folders = readdirSync("src/modules/")
folders.forEach(async (file) => {
    await import(`../modules/${file}`)
    console.log(`Loaded ${file} as module`)
})