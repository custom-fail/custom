import { readdirSync } from "fs"

const folders = readdirSync("src/modules/");
folders.forEach(async (file) => {
    await import(`../${file}`)
})