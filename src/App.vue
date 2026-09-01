<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { DontCrackApi, PLATFORM_COLORS, PLATFORM_LABELS, type Heartbeat, type Instance, type Platform } from './lib/api'

const instances = ref<Instance[]>(JSON.parse(localStorage.getItem('dc-instances') || '[]'))
const selectedId = ref(instances.value[0]?.id || '')
const form = ref<{ name: string; url: string; platform: Platform; password: string }>({
  name: '',
  url: 'http://127.0.0.1:11883',
  platform: 'linux',
  password: '',
})
const heartbeat = ref<Heartbeat | null>(null)
const error = ref('')
const loading = ref(false)
const syncing = ref(false)
const allOnline = ref(false)
const pollInterval = ref(3000)
const pollEnabled = ref(true)
let pollTimer: ReturnType<typeof setInterval> | null = null

const selected = computed(() => instances.value.find(d => d.id === selectedId.value))
const api = computed(() => selected.value ? new DontCrackApi(selected.value) : null)
const platformLabel = computed(() => selected.value ? PLATFORM_LABELS[selected.value.platform] : '')
const platformColor = computed(() => selected.value ? PLATFORM_COLORS[selected.value.platform] : '')
const stateRunning = computed(() => heartbeat.value?.state === 'running')

function save() {
  if (!form.value.name.trim()) return error.value = '请填写实例名称'
  if (!form.value.url.trim()) return error.value = '请填写服务地址'
  const instance: Instance = { ...form.value, url: form.value.url.trim().replace(/\/+$/, ''), id: crypto.randomUUID(), online: false }
  instances.value.push(instance); selectedId.value = instance.id; persist()
  form.value = { name: '', url: 'http://127.0.0.1:11883', platform: instance.platform, password: '' }
  refresh(true)
}
function persist() { localStorage.setItem('dc-instances', JSON.stringify(instances.value)) }
function remove() {
  if (!selected.value) return
  instances.value = instances.value.filter(d => d.id !== selected.value!.id)
  selectedId.value = instances.value[0]?.id || ''; persist(); heartbeat.value = null
}
function select(id: string) { selectedId.value = id; heartbeat.value = null; refresh(true) }

async function refresh(setOnline = false) {
  if (!api.value) { error.value = '请先添加实例'; return }
  loading.value = true; error.value = ''
  try {
    heartbeat.value = await api.value.heartbeat()
    selected.value!.online = true
    selected.value!.status = heartbeat.value.state
    if (setOnline) allOnline.value = true
    persist()
  } catch (e) {
    selected.value!.online = false
    selected.value!.status = 'offline'
    error.value = e instanceof Error ? e.message : '连接失败'
    persist()
  } finally { loading.value = false }
}

async function startup() {
  if (!api.value) return
  loading.value = true; error.value = ''
  try { const r = await api.value.startup(); if (String(r).trim() !== 'ok') error.value = r; await refresh() }
  catch (e) { error.value = e instanceof Error ? e.message : '启动失败' }
  finally { loading.value = false }
}

async function shutdown() {
  if (!api.value) return
  loading.value = true; error.value = ''
  try { const r = await api.value.shutdown(); if (String(r).trim() !== 'ok') error.value = r; await refresh() }
  catch (e) { error.value = e instanceof Error ? e.message : '停止失败' }
  finally { loading.value = false }
}

async function checkAll() {
  syncing.value = true
  let onlineCount = 0
  for (const inst of instances.value) {
    try { await new DontCrackApi(inst).ping(); inst.online = true; onlineCount++ }
    catch { inst.online = false }
  }
  allOnline.value = onlineCount === instances.value.length && instances.value.length > 0
  syncing.value = false; persist()
}

function poll() {
  if (pollTimer) clearInterval(pollTimer)
  if (!pollEnabled.value || pollInterval.value <= 0) return
  pollTimer = setInterval(() => { if (selected.value && !loading.value) refresh() }, pollInterval.value)
}

onMounted(() => { if (selected.value) refresh(true); checkAll(); poll() })
onUnmounted(() => { if (pollTimer) clearInterval(pollTimer) })
</script>

<template>
  <main class="app-shell">
    <header>
      <div>
        <span class="eyebrow">PROCESS GUARDIAN SUITE</span>
        <h1>DontCrack <b>统一管理工具</b></h1>
        <p>统一管理 Windows / Linux / Android / OpenHarmony 各平台 DontCrack 进程管理器</p>
      </div>
      <div class="header-actions">
        <div class="global-status" :class="{ online: allOnline }"><i></i> {{ syncing ? '扫描中…' : (allOnline ? '全部在线' : (instances.length ? '部分离线' : '未添加实例')) }}</div>
        <button class="ghost" @click="checkAll" :disabled="syncing">{{ syncing ? '扫描中…' : '扫描全部' }}</button>
        <button class="ghost" @click="refresh(selected != null)" :disabled="loading || !selected">{{ loading ? '刷新中…' : '刷新状态' }}</button>
      </div>
    </header>

    <section class="layout">
      <aside class="panel sidebar">
        <div class="panel-title"><h2>实例</h2><span>{{ instances.length }}</span></div>
        <div v-if="!instances.length" class="empty">还没有实例配置<br />添加一个 DontCrack 服务开始管理</div>
        <button v-for="inst in instances" :key="inst.id" class="device" :class="{ active: selectedId === inst.id }" @click="select(inst.id)">
          <i :class="{ online: inst.online }" :style="{ background: inst.online ? PLATFORM_COLORS[inst.platform] : undefined }"></i>
          <div>
            <strong>{{ inst.name }}</strong>
            <small>{{ PLATFORM_LABELS[inst.platform] }} · {{ inst.url }}</small>
            <small class="state" :class="{ running: inst.status === 'running' }">{{ inst.status || '—' }}</small>
          </div>
        </button>
        <div class="add-instance">
          <h3>添加实例</h3>
          <input v-model="form.name" placeholder="实例名称（如 边缘网关-01）" />
          <input v-model="form.url" placeholder="http://host:11883" />
          <input v-model="form.password" type="password" placeholder="管理密码（可选）" />
          <select v-model="form.platform">
            <option value="windows">Windows</option>
            <option value="linux">Linux (manylinux)</option>
            <option value="android">Android</option>
            <option value="openharmony">OpenHarmony</option>
          </select>
          <button @click="save">+ 添加实例</button>
        </div>
      </aside>

      <section class="content">
        <div class="panel overview">
          <div>
            <span class="eyebrow">CURRENT INSTANCE</span>
            <h2>{{ selected?.name || '未选择实例' }}</h2>
            <p>{{ selected ? `${platformLabel} · ${selected.url}` : '添加实例后，可查看进程状态、日志并进行启停操作' }}</p>
          </div>
          <div class="status" :class="{ online: selected?.online }"><i :style="selected?.online ? { background: platformColor } : {}"></i>{{ selected?.online ? (stateRunning ? '运行中' : '已停止') : '未连接' }}</div>
          <button v-if="selected" class="danger" @click="remove">删除实例</button>
        </div>

        <p v-if="error" class="error">{{ error }}</p>

        <div class="grid">
          <section class="panel control-panel">
            <div class="panel-title"><h2>进程控制</h2><span>HTTP API</span></div>
            <template v-if="selected && heartbeat">
              <div class="info-grid">
                <div><label>状态</label><b :class="{ running: stateRunning }">{{ heartbeat.state }}</b></div>
                <div><label>版本</label><span>{{ heartbeat.version || '—' }}</span></div>
                <div><label>PID</label><span>{{ heartbeat.process_pid || '—' }}</span></div>
                <div><label>重启次数</label><span>{{ heartbeat.restart_count ?? '—' }}</span></div>
                <div><label>文件类型</label><span>{{ heartbeat.file_type || '—' }}</span></div>
                <div><label>最后退出</label><span>{{ heartbeat.last_exit_time || '—' }}</span></div>
              </div>
              <div class="path">程序路径：{{ heartbeat.process_path || '—' }}</div>
              <div v-if="heartbeat.program_args" class="path">启动参数：{{ heartbeat.program_args }}</div>
              <div v-if="heartbeat.extra_env_raw" class="path">环境变量：{{ heartbeat.extra_env_raw }}</div>
              <p class="info-msg">{{ heartbeat.info || '进程管理器正常运行' }}</p>
            </template>
            <div v-else class="empty">选择实例并点击「刷新状态」获取进程信息</div>
            <div class="btn-row">
              <button class="primary" :disabled="loading || !selected || stateRunning" @click="startup">▶ 启动进程</button>
              <button class="danger-btn" :disabled="loading || !selected || !stateRunning" @click="shutdown">■ 停止进程</button>
            </div>
          </section>

          <section class="panel log-panel">
            <div class="panel-title"><h2>日志</h2><span>{{ heartbeat?.logs?.length ?? 0 }} 条</span></div>
            <div v-if="!heartbeat?.logs?.length" class="empty">暂无日志</div>
            <div v-else class="terminal">
              <div v-for="(line, idx) in heartbeat.logs" :key="idx" :class="{ err: line.includes('[STDERR]') }">{{ line }}</div>
            </div>
          </section>
        </div>

        <section class="panel poll-panel">
          <div class="panel-title"><h2>自动刷新</h2><span>轮询心跳接口</span></div>
          <div class="poll-row">
            <label><input type="checkbox" v-model="pollEnabled" @change="poll" /> 启用轮询</label>
            <label>间隔 <select v-model.number="pollInterval" @change="poll">
              <option :value="1000">1s</option><option :value="3000">3s</option><option :value="5000">5s</option><option :value="10000">10s</option>
            </select></label>
          </div>
        </section>
      </section>
    </section>

    <footer>DontCrack 统一管理工具 · 与原版 DontCrack 各平台共享同一套 HTTP 接口（/startup /heartbeat /shutdown）· 默认端口 11883</footer>
  </main>
</template>