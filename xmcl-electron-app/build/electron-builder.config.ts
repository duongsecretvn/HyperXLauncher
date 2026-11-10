/* eslint-disable no-template-curly-in-string */
import { config as dotenv } from 'dotenv'
import type { Configuration } from 'electron-builder'

dotenv()

export const config = {
  productName: 'HyperXLauncher',
  appId: 'hyperxlauncher',
  compression: 'normal',
  directories: {
    output: 'build/output',
    buildResources: 'build',
    app: '.',
  },
  protocols: {
    name: 'HyperXLauncher',
    schemes: ['hyperx', 'xmcl'],
  },
  // assign publish for auto-updater
  // set this to your own repo!
  publish: [{
    provider: 'github',
    owner: 'duongsecretvn',
    repo: 'HyperXLauncher',
  }],
  files: [{
    from: 'dist',
    to: '.',
    filter: ['**/*.js', '**/*.ico', '**/*.png', '**/*.webp', '**/*.svg', '*.node', '*.dll', '*.exe', '**/*.html', '**/*.css', '**/*.woff2', '**/*.woff', '**/*.ttf', '**/*.otf', '**/*.wasm'],
  }, {
    from: '.',
    to: '.',
    filter: 'package.json',
  }],
  artifactName: 'HyperXLauncher-${version}-${platform}-${arch}.${ext}',
  appx: {
    displayName: 'HyperXLauncher',
    applicationId: 'hyperxlauncher',
    identityName: 'hyperxlauncher',
    backgroundColor: 'transparent',
    publisher: process.env.PUBLISHER,
    publisherDisplayName: 'HyperX',
    setBuildNumber: true,
  },
  dmg: {
    artifactName: 'HyperXLauncher-${version}-${arch}.${ext}',
    contents: [
      {
        x: 410,
        y: 150,
        type: 'link',
        path: '/Applications',
      },
      {
        x: 130,
        y: 150,
        type: 'file',
      },
    ],
  },
  mac: {
    icon: 'icons/dark.icns',
    darkModeSupport: true,
    target: [
      {
        target: 'dmg',
        arch: ['arm64', 'x64'],
      },
    ],
    extendInfo: {
      NSLocalNetworkUsageDescription: 'HyperX and Minecraft need access to your local network to discover and connect to LAN games and servers.',
      NSMicrophoneUsageDescription: 'A Minecraft mod wants to access your microphone.',
      NSCameraUsageDescription: 'Please give us access to your camera',
      'com.apple.security.device.audio-input': true,
      'com.apple.security.device.camera': true,
    },
  },
  win: {
    signtoolOptions: {
      certificateFile: undefined as string | undefined,
      publisherName: 'HyperX',
    },
    icon: 'icons/dark.ico',
    electronLanguages: ['en-US'],
    target: [
      {
        target: 'zip',
        arch: [
          'x64',
        ],
      },
      'nsis',
    ],
  },
  nsis: {
    oneClick: false,
    allowToChangeInstallationDirectory: true,
    shortcutName: 'HyperXLauncher',
    artifactName: 'HyperXLauncher-Setup-${version}.${ext}',
    differentialPackage: false,
  },
  linux: {
    maintainer: 'duongsecretvn <duongsecretvn@gmail.com>',
    executableName: 'HyperXLauncher',
    electronLanguages: ['en-US'],
    desktop: {
      entry: {
        MimeType: 'x-scheme-handler/hyperx;x-scheme-handler/xmcl',
        StartupWMClass: 'HyperXLauncher',
      },
    },
    category: 'Game',
    icon: 'icons/dark.icns',
    artifactName: 'HyperXLauncher-${version}-${arch}.${ext}',
    target: [
      { target: 'deb', arch: ['x64', 'arm64'] },
      { target: 'rpm', arch: ['x64', 'arm64'] },
      { target: 'AppImage', arch: ['x64', 'arm64'] },
      { target: 'tar.xz', arch: ['x64', 'arm64'] },
      { target: 'pacman', arch: ['x64', 'arm64'] },
    ],
  },
  snap: {
    publish: [
      'github',
    ],
  },
} satisfies Configuration
