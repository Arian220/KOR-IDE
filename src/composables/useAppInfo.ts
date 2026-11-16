import { ref } from 'vue'
import { APP_INFO } from '@/config/appInfo'

export interface AppOsInfo {
  platform: string
  version: string
  architecture: string
  locale: string
  screen: string
}

interface UserAgentData {
  platform?: string
  platformVersion?: string
  architecture?: string
  bitness?: string
}

interface NavigatorWithUserAgentData extends Navigator {
  userAgentData?: {
    getHighEntropyValues(hints: string[]): Promise<UserAgentData>
  }
}

function detectFromUserAgent(): Pick<AppOsInfo, 'platform' | 'version' | 'architecture'> {
  const ua = typeof navigator === 'undefined' ? '' : navigator.userAgent
  const windows = ua.match(/Windows NT ([\d.]+)/)
  if (windows) {
    const versions: Record<string, string> = {
      '10.0': '10/11',
      '6.3': '8.1',
      '6.2': '8',
      '6.1': '7',
    }
    return { platform: 'Windows', version: versions[windows[1]] ?? windows[1], architecture: /Win64|WOW64|x64/i.test(ua) ? 'x86_64' : 'x86' }
  }

  const mac = ua.match(/Mac OS X ([\d_.]+)/)
  if (mac) {
    return { platform: 'macOS', version: mac[1].replace(/_/g, '.'), architecture: /ARM64|aarch64/i.test(ua) ? 'arm64' : 'x86_64' }
  }

  if (/Android/.test(ua)) {
    const version = ua.match(/Android ([\d.]+)/)?.[1] ?? ''
    return { platform: 'Android', version, architecture: /arm64|aarch64/i.test(ua) ? 'arm64' : '' }
  }

  if (/Linux/.test(ua)) {
    return { platform: 'Linux', version: '', architecture: /aarch64|arm64/i.test(ua) ? 'arm64' : /x86_64|amd64|Win64/i.test(ua) ? 'x86_64' : '' }
  }

  return {
    platform: navigator?.platform || 'Desconocido',
    version: '',
    architecture: '',
  }
}

async function detectOsInfo(): Promise<AppOsInfo> {
  const detected = detectFromUserAgent()
  const result: AppOsInfo = {
    ...detected,
    locale: typeof navigator === 'undefined' ? '' : navigator.language,
    screen:
      typeof window === 'undefined'
        ? ''
        : `${window.screen.width} × ${window.screen.height}`,
  }

  const navigatorWithData = typeof navigator === 'undefined'
    ? null
    : (navigator as NavigatorWithUserAgentData)
  if (!navigatorWithData?.userAgentData?.getHighEntropyValues) return result

  try {
    const data = await navigatorWithData.userAgentData.getHighEntropyValues([
      'platform',
      'platformVersion',
      'architecture',
      'bitness',
    ])
    result.platform = data.platform || result.platform
    result.version = data.platformVersion || result.version
    result.architecture = data.architecture
      ? `${data.architecture}${data.bitness ? ` ${data.bitness} bits` : ''}`
      : result.architecture
  } catch {
    // Se conserva la información deducida del agente de usuario.
  }

  return result
}

const osInfo = ref<AppOsInfo>({
  platform: '',
  version: '',
  architecture: '',
  locale: '',
  screen: '',
})

const osLoaded = ref(false)

export function useAppInfo() {
  if (!osLoaded.value) {
    osLoaded.value = true
    void detectOsInfo().then((info) => {
      osInfo.value = info
    })
  }

  return {
    app: APP_INFO,
    os: osInfo,
  }
}
