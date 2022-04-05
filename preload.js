const {contextBridge, ipcRenderer} = require("electron");

contextBridge.exposeInMainWorld("electronAPI", {
    dataFetch : () => ipcRenderer.invoke("display:data_fetch"),
    mlog : (args) => ipcRenderer.send("debug:log", args)
});