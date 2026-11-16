const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('path')

const devServerUrl = process.env.VITE_DEV_SERVER_URL

function windowFromEvent (event) {
  return BrowserWindow.fromWebContents(event.sender)
}

ipcMain.on('window:minimize', (event) => {
  windowFromEvent(event)?.minimize()
})

ipcMain.on('window:toggle-maximize', (event) => {
  const win = windowFromEvent(event)
  if (!win) return
  if (win.isMaximized()) {
    win.unmaximize()
  } else {
    win.maximize()
  }
})

ipcMain.on('window:close', (event) => {
  windowFromEvent(event)?.close()
})

function createWindow () {
  const win = new BrowserWindow({
    width: 1200,
    height: 760,
    minWidth: 900,
    minHeight: 600,
    title: 'KOR',
    frame: false,
    backgroundColor: '#1e1e1e',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false
    }
  })

  if (devServerUrl) {
    win.loadURL(devServerUrl)
    win.webContents.openDevTools()
  } else {
    win.loadFile(path.join(__dirname, 'dist', 'index.html'))
  }
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
