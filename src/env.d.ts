/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ARDUINO_CLI_PATH?: string
  readonly VITE_MOCK_IPC?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
