const fs = require("fs");
const {app, BrowserWindow, ipcMain} = require("electron");
const path = require("path");

function createWindow () {
    const win = new BrowserWindow({
        resizable : false,
        width : 400,
        height : 400,
        webPreferences : {
            preload : path.join(__dirname, "preload.js")
        }
    });
    win.loadFile("display/main.html");
}

function display_data_fetch () {
    let f = {};
    let string = fs.readFileSync(path.join(__dirname, process.argv[2] || "output.test"), "utf-8");
    let items = string.split("\n").join(" ").split(" ");
    const conv = "0123456789abcdef";
    let data_array = [];
    for (let i = 10; i < items.length; i ++) {
        /**@type {String} */
        const item = items[i];
        if (item.length === 0) {
            continue;
        }
        data_array.push(conv.indexOf(item[0]) * 16 + conv.indexOf(item[1]));
    }
    const pf = (s) => {
        let n = 0;
        let p = 1;
        for (let i = 0; i < s.length; i ++) {
            n += (conv.indexOf(s[s.length - i - 1]) * p);
            p *= 16;
        }
        return n;
    };
    f.width = pf(items.slice(0, 4).join(""));
    f.height = pf(items.slice(4, 8).join(""));
    f.data = data_array;
    return f;
}

app.whenReady().then(() => {
    ipcMain.on("debug:log", (_, args) => {console.log(...args)});
    ipcMain.handle("display:data_fetch", display_data_fetch);
    createWindow();
});

app.on("window-all-closed", () => {
    app.quit();
});