<div align="center">
  <img src="./Logo.png" alt="logo" width="100" />
  <h2>DontCrack Desktop</h2>
  <h3>Unified Management Desktop Tool for DontCrack (Tauri 2 + Vue 3)</h3>
</div>

### 1. Introduction

- A cross-platform desktop client built with **Tauri 2 + Vue 3 + TypeScript**, serving as the unified management entry for DontCrack process managers
- Unified management of the DontCrack process management services across four platforms: `DontCrack4Windows`, `DontCrack4ManyLinux`, `DontCrack4AndroidLinuxKernelSide` and `DontCrack4OpenHarmonyLinuxKernelSide`
- Directly reuses the same HTTP API shared by all DontCrack platforms (`/startup` / `/heartbeat` / `/shutdown`, default port 11883); no extra components need to be installed on target devices
- Requests are proxied through `reqwest` on the Rust side, naturally bypassing browser CORS restrictions, so the desktop client can directly reach LAN/local DontCrack services

### 2. Features

- **Multi-instance management**: maintain multiple DontCrack service instances simultaneously, distinguishable by platform (Windows / Linux / Android / OpenHarmony), persisted locally
- **Process monitoring**: poll `/heartbeat` to show process state, PID, restart count, file type, program path, startup arguments, environment variables and last exit time in real time
- **Process control**: one-click `/startup` to start a process (resetting the restart count) and `/shutdown` to terminate it
- **Log viewing**: display process logs returned by the DontCrack heartbeat API in real time (STDERR/STDOUT color-coded)
- **Availability scan**: scan the online status of all instances at once
- **Password authentication**: configure the DontCrack management password; the `password` parameter is attached to requests automatically

### 3. Usage

- After launching the app, fill in the "Add Instance" area on the left:
  - **Instance name**: e.g. `edge-gateway-01`, for identification
  - **Service address**: the HTTP address of DontCrack on the target device, e.g. `http://192.168.1.100:11883` (port 11883 is used by default if omitted)
  - **Management password**: must match the `-password` startup argument of DontCrack (leave empty if password protection is disabled)
  - **Platform type**: Windows / Linux / Android / OpenHarmony, used for grouping
- Click "Add Instance", then click "Refresh Status" to view the running state and logs of the managed process
- Use the "Start Process / Stop Process" buttons to control the target process; "Scan All" probes the online status of every instance at once
- The "Auto Refresh" section at the bottom enables/disables polling with a 1s~10s heartbeat interval

### 4. Development Environment

Requires [Node.js](https://nodejs.org/) (18+) and [Rust](https://www.rust-lang.org/) (1.77+), and the Tauri platform prerequisites (see the [Tauri documentation](https://v2.tauri.app/start/prerequisites/)).

```bash
# Install dependencies
npm install

# Development mode (HMR + Tauri desktop window)
npm run tauri dev

# Type check + frontend build
npm run build

# Bundle desktop installers
npm run tauri build
```

### 5. Architecture

| Layer        | Description                                                                          |
|--------------|--------------------------------------------------------------------------------------|
| Frontend     | Vue 3 (Composition API) + Vite 6 + TypeScript, data persisted in `localStorage`      |
| Desktop shell| Tauri 2 (Rust), exposing the `dc_request` proxy command via `tauri::command`          |
| Call chain   | Frontend `invoke('dc_request')` → Rust `reqwest` → DontCrack HTTP API, with timeout control and password auth |

### 6. API Contract

All DontCrack platform versions (Windows / Linux / Android / OpenHarmony) share the same HTTP API, which this tool reuses directly:

| Endpoint     | Method  | Description                                              |
|--------------|---------|----------------------------------------------------------|
| `/startup`   | GET/POST| Start the process and reset the restart count            |
| `/heartbeat` | GET/POST| Get heartbeat info: startup state and cached logs        |
| `/shutdown`  | GET/POST| Terminate the process                                    |

When password protection is enabled, requests carry `password` in the URL parameters, e.g. `xxx/startup?password=123456`.

### 7. License

This project is licensed under the [Apache License 2.0](./LICENSE), consistent with all other DontCrack platform versions.