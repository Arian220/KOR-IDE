const { contextBridge } = require('electron')
const { execFile } = require('child_process')
const fs = require('fs')
const path = require('path')

const execOptions = { windowsHide: true }
const defaultCli = process.platform === 'win32' ? 'C:\\ArduinoCLI\\arduino-cli.exe' : 'arduino-cli'

function resolveCli(rutaCli) {
  return rutaCli || defaultCli
}

function writeSketch(codigo) {
  const dir = path.join(__dirname, 'temp_sketch')
  fs.mkdirSync(dir, { recursive: true })
  const sketchPath = path.join(dir, 'temp_sketch.ino')
  fs.writeFileSync(sketchPath, codigo)
  return dir
}

contextBridge.exposeInMainWorld('api', {
  detectarPlacas: (rutaCli) => {
    return new Promise((resolve, reject) => {
      execFile(
        resolveCli(rutaCli),
        ['board', 'list', '--format', 'json'],
        execOptions,
        (error, stdout) => {
          if (error) {
            reject(error)
            return
          }
          resolve(stdout)
        },
      )
    })
  },

  compilar: (codigo, rutaCli) => {
    return new Promise((resolve) => {
      const dir = writeSketch(codigo)

      execFile(
        resolveCli(rutaCli),
        ['compile', '-b', 'arduino:avr:uno', dir],
        execOptions,
        (error, stdout, stderr) => {
          if (error) {
            resolve({ success: false, error: error.message, stdout, stderr })
            return
          }
          resolve({ success: true, stdout, stderr })
        },
      )
    })
  },

  subirAPlaca: (codigo, rutaCli, puerto) => {
    return new Promise((resolve) => {
      const dir = writeSketch(codigo)

      execFile(
        resolveCli(rutaCli),
        ['compile', '-b', 'arduino:avr:uno', '-p', puerto, '--upload', dir],
        execOptions,
        (error, stdout, stderr) => {
          if (error) {
            resolve({ success: false, error: error.message, stdout, stderr })
            return
          }
          resolve({ success: true, stdout, stderr })
        },
      )
    })
  },
})
