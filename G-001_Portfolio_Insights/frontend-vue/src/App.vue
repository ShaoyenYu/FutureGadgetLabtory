<script setup>
import { onBeforeUnmount, onMounted, ref, nextTick, watch, computed } from 'vue'
import axios from 'axios'
import * as echarts from 'echarts'

const STORAGE_KEY = 'portfolio:last'
const THEME_KEY = 'ui:theme'
const FUND_NAME_CACHE_KEY = 'fund-name-cache'
const FUND_NAME_URL_CACHE_KEY = 'fund-name-url-cache'
const COLUMN_WIDTH_KEY = 'ui:column-widths'
const portfolioItems = ref([{ code: '015202', shares: 100 }])
const startDate = ref('')
const endDate = ref('')
const loading = ref(false)
const error = ref('')
const chartEl = ref(null)
const debugInfo = ref(null)
const portfolioData = ref(null)
const warnings = ref([])
const chartMode = ref('LJJZ')
const theme = ref('light')
const showTotalAmount = ref(true)
const fundNameMap = ref({})
const fundNameUrlMap = ref({})
const fundUnitValueMap = ref({})
const fundNameErrors = ref({})
const sortKey = ref('')
const sortOrder = ref('')
const suppressFundNameWatch = ref(false)
const lastCodeSnapshot = ref([])
let chartInstance = null
lastCodeSnapshot.value = portfolioItems.value.map((item) => String(item.code || '').trim())
const DAY_MS = 24 * 60 * 60 * 1000
const columnWidths = ref({
  code: 96,
  shares: 104,
  name: 220,
  nav: 110,
  position: 140,
  return: 110,
  sharpe: 110,
  mdd: 120,
  mddDuration: 110,
  action: 80
})
const minColumnWidths = {
  code: 70,
  shares: 80,
  name: 160,
  nav: 90,
  position: 110,
  return: 90,
  sharpe: 90,
  mdd: 90,
  mddDuration: 90,
  action: 60
}

const columnStyle = computed(() => ({
  '--col-code': `${columnWidths.value.code}px`,
  '--col-shares': `${columnWidths.value.shares}px`,
  '--col-name': `${columnWidths.value.name}px`,
  '--col-nav': `${columnWidths.value.nav}px`,
  '--col-position': `${columnWidths.value.position}px`,
  '--col-return': `${columnWidths.value.return}px`,
  '--col-sharpe': `${columnWidths.value.sharpe}px`,
  '--col-mdd': `${columnWidths.value.mdd}px`,
  '--col-mdd-duration': `${columnWidths.value.mddDuration}px`,
  '--col-action': `${columnWidths.value.action}px`
}))

const loadColumnWidths = () => {
  const raw = localStorage.getItem(COLUMN_WIDTH_KEY)
  if (!raw) return
  try {
    const data = JSON.parse(raw)
    if (!data || typeof data !== 'object') return
    const next = { ...columnWidths.value }
    Object.keys(next).forEach((key) => {
      const value = Number(data[key])
      if (Number.isFinite(value) && value > 0) {
        next[key] = value
      }
    })
    columnWidths.value = next
  } catch {
    return
  }
}

const saveColumnWidths = () => {
  localStorage.setItem(COLUMN_WIDTH_KEY, JSON.stringify(columnWidths.value))
}

const startResize = (key, event) => {
  event.preventDefault()
  const startX = event.clientX
  const startWidth = columnWidths.value[key]
  const minWidth = minColumnWidths[key] || 60
  const onMouseMove = (moveEvent) => {
    const delta = moveEvent.clientX - startX
    const nextWidth = Math.max(minWidth, startWidth + delta)
    columnWidths.value = { ...columnWidths.value, [key]: nextWidth }
  }
  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    saveColumnWidths()
  }
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

const formatDate = (date) => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

const toDateMs = (value) => {
  if (!value) return null
  const ms = Date.parse(`${value}T00:00:00`)
  return Number.isNaN(ms) ? null : ms
}

const initDates = () => {
  const today = new Date()
  const lastYear = new Date()
  lastYear.setFullYear(today.getFullYear() - 1)
  startDate.value = formatDate(lastYear)
  endDate.value = formatDate(today)
}

const applyTheme = (value) => {
  document.documentElement.setAttribute('data-theme', value)
}

const toggleTheme = () => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
}

const loadSavedPortfolio = () => {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (!raw) return false
  try {
    const data = JSON.parse(raw)
    if (Array.isArray(data.items) && data.items.length) {
      portfolioItems.value = data.items.map((item) => ({
        code: String(item.code || '').trim(),
        shares: Number(item.shares) || 0
      }))
    }
    if (data.start_date) startDate.value = data.start_date
    if (data.end_date) endDate.value = data.end_date
    if (data.sort_key) sortKey.value = data.sort_key
    if (data.sort_order) sortOrder.value = data.sort_order
    if (data.fund_name_cache && typeof data.fund_name_cache === 'object') {
      fundNameMap.value = data.fund_name_cache
    } else {
      const cachedNames = localStorage.getItem(FUND_NAME_CACHE_KEY)
      if (cachedNames) {
        fundNameMap.value = JSON.parse(cachedNames)
      }
    }
    if (data.fund_name_url_cache && typeof data.fund_name_url_cache === 'object') {
      fundNameUrlMap.value = data.fund_name_url_cache
    } else {
      const cachedUrls = localStorage.getItem(FUND_NAME_URL_CACHE_KEY)
      if (cachedUrls) {
        fundNameUrlMap.value = JSON.parse(cachedUrls)
      }
    }
    return true
  } catch {
    return false
  }
}

const savePortfolio = () => {
  const payload = {
    items: portfolioItems.value.map((item) => ({
      code: String(item.code || '').trim(),
      shares: Number(item.shares) || 0
    })),
    start_date: startDate.value,
    end_date: endDate.value,
    sort_key: sortKey.value,
    sort_order: sortOrder.value,
    fund_name_cache: fundNameMap.value,
    fund_name_url_cache: fundNameUrlMap.value
  }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(payload))
  localStorage.setItem(FUND_NAME_CACHE_KEY, JSON.stringify(fundNameMap.value))
  localStorage.setItem(FUND_NAME_URL_CACHE_KEY, JSON.stringify(fundNameUrlMap.value))
}

const exportCsv = () => {
  const header = ['code', 'shares']
  const rows = portfolioItems.value.map((item) => [
    String(item.code || '').trim(),
    Number(item.shares) || 0
  ])
  const csvLines = [header, ...rows].map((row) =>
    row.map((value) => `"${String(value).replace(/"/g, '""')}"`).join(',')
  )
  const csvContent = csvLines.join('\n')
  const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  const date = new Date()
  const datePart = `${date.getFullYear()}${String(date.getMonth() + 1).padStart(2, '0')}${String(date.getDate()).padStart(2, '0')}`
  link.download = `portfolio_${datePart}.csv`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

const parseCsvLine = (line) => {
  const result = []
  let current = ''
  let inQuotes = false
  for (let i = 0; i < line.length; i += 1) {
    const char = line[i]
    if (char === '"') {
      if (inQuotes && line[i + 1] === '"') {
        current += '"'
        i += 1
      } else {
        inQuotes = !inQuotes
      }
    } else if (char === ',' && !inQuotes) {
      result.push(current)
      current = ''
    } else {
      current += char
    }
  }
  result.push(current)
  return result
}

const parseNumber = (value) => {
  if (value === null || value === undefined) return null
  const cleaned = String(value).replace(/,/g, '').trim()
  if (!cleaned) return null
  const num = Number(cleaned)
  return Number.isFinite(num) ? num : null
}

const importCsv = (eventOrFile) => {
  const file = eventOrFile?.target?.files?.[0] || eventOrFile
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    const content = String(reader.result || '')
      .replace(/\r\n/g, '\n')
      .replace(/\r/g, '\n')
    const lines = content.split('\n').filter((line) => line.trim() !== '')
    if (!lines.length) return
    const first = parseCsvLine(lines[0])
    const firstHeader = String(first[0] || '').trim().toLowerCase()
    const secondHeader = String(first[1] || '').trim().toLowerCase()
    const isHeader = (firstHeader === 'code' || firstHeader === '基金代码')
      && (secondHeader === 'shares' || secondHeader === 'position' || secondHeader === '仓位' || secondHeader === '份额')
    const startIndex = first.length >= 2 && isHeader ? 1 : 0
    const items = lines.slice(startIndex).map((line) => {
      const [code, shares] = parseCsvLine(line)
      const parsedShares = parseNumber(shares)
      return {
        code: String(code || '').trim(),
        shares: parsedShares ?? 0
      }
    }).filter((item) => item.code || item.shares)
    if (items.length) {
      suppressFundNameWatch.value = true
      portfolioItems.value = items
      fetchFundNames()
      updateFundUnitValues()
      savePortfolio()
      setTimeout(() => {
        suppressFundNameWatch.value = false
      }, 0)
    }
    if (eventOrFile?.target) {
      eventOrFile.target.value = ''
    }
  }
  reader.readAsText(file)
}

const getPortfolioDrawdownInfo = (data) => {
  if (!data.length) return null
  let peak = null
  let peakIndex = 0
  let maxDrawdown = 0
  let startIndex = 0
  let endIndex = 0
  const dateMsMap = data.map((item) => toDateMs(item.date))
  data.forEach((item, index) => {
    const rawValue = chartMode.value === 'DWJZ' ? item.normalized_total_value : item.normalized_value
    const value = rawValue === null || rawValue === undefined || Number.isNaN(Number(rawValue)) ? null : Number(rawValue)
    const ms = dateMsMap[index]
    if (value === null || ms === null) return
    if (peak === null) {
      peak = value
      peakIndex = index
      return
    }
    if (value > peak) {
      peak = value
      peakIndex = index
      return
    }
    const drawdown = peak ? (value - peak) / peak : 0
    if (drawdown < maxDrawdown) {
      maxDrawdown = drawdown
      startIndex = peakIndex
      endIndex = index
    }
  })
  if (peak === null || startIndex === endIndex) return null
  const startMs = dateMsMap[startIndex]
  const endMs = dateMsMap[endIndex]
  if (startMs === null || endMs === null) return null
  const days = Math.max(0, Math.round((endMs - startMs) / DAY_MS))
  return {
    startIndex,
    endIndex,
    days
  }
}

const renderChart = (data) => {
  if (!chartEl.value) return
  if (chartInstance) {
    chartInstance.dispose()
    chartInstance = null
  }
  chartInstance = echarts.init(chartEl.value)
  const dates = data.map((item) => item.date)
  const values = data.map((item) =>
    chartMode.value === 'DWJZ' ? item.normalized_total_value : item.normalized_value
  )
  const lastIndex = values.length - 1
  const lastValue = lastIndex >= 0 ? Number(values[lastIndex]) : null
  const lastDate = lastIndex >= 0 ? dates[lastIndex] : null
  const drawdownInfo = getPortfolioDrawdownInfo(data)
  const title = 'Portfolio Asset Value (Normalized)'
  const isDark = theme.value === 'dark'
  const axisColor = isDark ? '#cfd7e6' : '#1f2d3d'
  const splitColor = isDark ? '#2f3744' : '#ebeef5'
  const drawdownShade = isDark ? 'rgba(255, 125, 125, 0.16)' : 'rgba(245, 108, 108, 0.18)'
  const option = {
    title: { text: title, left: 'center', textStyle: { color: axisColor } },
    tooltip: { trigger: 'axis' },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: { color: axisColor },
      axisLine: { lineStyle: { color: axisColor } },
      axisTick: { lineStyle: { color: axisColor } }
    },
    yAxis: {
      type: 'value',
      scale: true,
      axisLabel: { color: axisColor },
      axisLine: { lineStyle: { color: axisColor } },
      splitLine: { lineStyle: { color: splitColor } }
    },
    series: [
      {
        data: values,
        type: 'line',
        smooth: true,
        areaStyle: {},
        markArea: drawdownInfo ? {
          itemStyle: { color: drawdownShade },
          label: { show: true, color: axisColor },
          data: [[
            { xAxis: dates[drawdownInfo.startIndex] },
            { xAxis: dates[drawdownInfo.endIndex], label: { formatter: `回撤 ${drawdownInfo.days}天` } }
          ]]
        } : undefined,
        markPoint: lastDate !== null && Number.isFinite(lastValue) ? {
          symbol: 'circle',
          symbolSize: 8,
          label: { show: true, color: axisColor, formatter: (params) => formatMetric(params.value) },
          data: [{ coord: [lastDate, lastValue], value: lastValue }]
        } : undefined
      }
    ],
    dataZoom: [{ type: 'inside', start: 0, end: 100 }, { start: 0, end: 100 }]
  }
  chartInstance.setOption(option, true)
  chartInstance.resize()
}

const getPortfolioSeries = () => {
  const data = portfolioData.value?.portfolio?.data || []
  if (!data.length) return []
  return data.map((item) =>
    chartMode.value === 'DWJZ' ? item.normalized_total_value : item.normalized_value
  ).filter((value) => value !== null && value !== undefined && !Number.isNaN(Number(value)))
    .map((value) => Number(value))
}

const getValidCodes = (items) => items
  .map((item) => String(item.code || '').trim())
  .filter((code) => /^\d{6}$/.test(code))

const fetchFundNames = async (codes = null) => {
  const baseCodes = codes ? codes : getValidCodes(portfolioItems.value)
  const uniqueCodes = Array.from(new Set(baseCodes))
  if (!uniqueCodes.length) {
    if (!codes) {
      fundNameMap.value = {}
      fundNameUrlMap.value = {}
      fundNameErrors.value = {}
    }
    return
  }
  const missingCodes = uniqueCodes.filter((code) => !fundNameMap.value[code])
  if (!missingCodes.length) {
    return
  }
  try {
    const response = await axios.post('/api/fund-name/batch', { codes: missingCodes })
    fundNameMap.value = { ...fundNameMap.value, ...(response.data?.names || {}) }
    const debugInfo = response.data?.debug_info || {}
    const urlMap = { ...fundNameUrlMap.value }
    Object.keys(debugInfo).forEach((code) => {
      const url = debugInfo[code]?.url
      if (url) {
        urlMap[code] = url
      }
    })
    fundNameUrlMap.value = urlMap
    const errors = response.data?.errors || []
    fundNameErrors.value = errors.reduce((acc, item) => {
      acc[item.code] = item.error
      return acc
    }, {})
    savePortfolio()
  } catch {
    fundNameErrors.value = {}
  }
}

const getLatestEntry = (entries, targetDate) => {
  if (!entries || !entries.length) return null
  const sorted = entries.slice().sort((a, b) => a.date.localeCompare(b.date))
  if (!targetDate) return sorted[sorted.length - 1]
  const filtered = sorted.filter((entry) => entry.date <= targetDate)
  return filtered.length ? filtered[filtered.length - 1] : sorted[sorted.length - 1]
}

const updateFundUnitValues = () => {
  const funds = portfolioData.value?.funds || []
  const map = {}
  funds.forEach((fund) => {
    const entry = getLatestEntry(fund.data || [], endDate.value)
    if (!entry || !entry.value) {
      map[fund.code] = null
      return
    }
    map[fund.code] = entry.value
  })
  fundUnitValueMap.value = map
}

const getCodeKey = (item) => String(item.code || '').trim()

const normalizeDateRange = () => {
  if (!startDate.value || !endDate.value) return
  const startMs = toDateMs(startDate.value)
  const endMs = toDateMs(endDate.value)
  if (startMs === null || endMs === null) return
  if (startMs > endMs) {
    const temp = startDate.value
    startDate.value = endDate.value
    endDate.value = temp
  }
}

const formatAmount = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '-'
  return Number(value).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

const getEntriesInRange = (entries) => {
  const sorted = (entries || []).slice().sort((a, b) => a.date.localeCompare(b.date))
  if (!sorted.length) return []
  const start = startDate.value
  const end = endDate.value
  const filtered = sorted.filter((entry) => (!start || entry.date >= start) && (!end || entry.date <= end))
  return filtered.length ? filtered : sorted
}

const fundReturnMap = computed(() => {
  const map = {}
  const funds = portfolioData.value?.funds || []
  funds.forEach((fund) => {
    const entries = getEntriesInRange(fund.data || [])
    const startEntry = entries[0]
    const endEntry = entries[entries.length - 1]
    if (!startEntry || !endEntry) {
      map[fund.code] = null
      return
    }
    if (startEntry.cumulative_value === null || endEntry.cumulative_value === null) {
      map[fund.code] = null
      return
    }
    const base = Number(startEntry.cumulative_value)
    const tail = Number(endEntry.cumulative_value)
    if (!Number.isFinite(base) || !Number.isFinite(tail) || base === 0) {
      map[fund.code] = null
      return
    }
    map[fund.code] = tail / base - 1
  })
  return map
})

const fundSharpeMap = computed(() => {
  const map = {}
  const funds = portfolioData.value?.funds || []
  funds.forEach((fund) => {
    const entries = getEntriesInRange(fund.data || [])
    const values = entries
      .map((entry) => entry.cumulative_value)
      .filter((value) => value !== null && value !== undefined && !Number.isNaN(Number(value)))
      .map((value) => Number(value))
    if (values.length < 2) {
      map[fund.code] = null
      return
    }
    const returns = []
    for (let i = 1; i < values.length; i += 1) {
      const prev = values[i - 1]
      const curr = values[i]
      if (!prev) continue
      returns.push(curr / prev - 1)
    }
    if (!returns.length) {
      map[fund.code] = null
      return
    }
    const mean = returns.reduce((sum, val) => sum + val, 0) / returns.length
    const variance = returns.reduce((sum, val) => sum + (val - mean) ** 2, 0) / returns.length
    const std = Math.sqrt(variance)
    map[fund.code] = std ? (mean / std) * Math.sqrt(252) : 0
  })
  return map
})

const fundDrawdownDetailMap = computed(() => {
  const map = {}
  const funds = portfolioData.value?.funds || []
  funds.forEach((fund) => {
    const entries = getEntriesInRange(fund.data || [])
      .map((entry) => ({
        value: entry.cumulative_value,
        dateMs: toDateMs(entry.date)
      }))
      .filter((entry) => entry.dateMs !== null && entry.value !== null && entry.value !== undefined && !Number.isNaN(Number(entry.value)))
      .map((entry) => ({
        value: Number(entry.value),
        dateMs: entry.dateMs
      }))
    if (!entries.length) {
      map[fund.code] = null
      return
    }
    let peak = entries[0].value
    let peakDateMs = entries[0].dateMs
    let maxDrawdown = 0
    let maxDrawdownDays = 0
    entries.forEach((entry) => {
      if (entry.value > peak) {
        peak = entry.value
        peakDateMs = entry.dateMs
        return
      }
      const drawdown = peak ? (entry.value - peak) / peak : 0
      if (drawdown < maxDrawdown) {
        maxDrawdown = drawdown
        maxDrawdownDays = Math.max(0, Math.round((entry.dateMs - peakDateMs) / DAY_MS))
      }
    })
    map[fund.code] = { value: maxDrawdown, days: maxDrawdownDays }
  })
  return map
})

const getFundReturnRate = (item) => {
  const codeKey = getCodeKey(item)
  return fundReturnMap.value[codeKey] ?? null
}

const getFundSharpe = (item) => {
  const codeKey = getCodeKey(item)
  return fundSharpeMap.value[codeKey] ?? null
}

const getFundDrawdown = (item) => {
  const codeKey = getCodeKey(item)
  const detail = fundDrawdownDetailMap.value[codeKey]
  return detail ? detail.value : null
}

const getFundDrawdownDays = (item) => {
  const codeKey = getCodeKey(item)
  const detail = fundDrawdownDetailMap.value[codeKey]
  return detail ? detail.days : null
}

const formatReturnPercent = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '-'
  return `${(Number(value) * 100).toFixed(3)}%`
}

const formatReturnPercentAbs = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '-'
  return `${(Math.abs(Number(value)) * 100).toFixed(3)}%`
}

const formatDrawdownDays = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '-'
  return String(Math.round(Number(value)))
}

const formatMetric = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return '-'
  return Number(value).toFixed(3)
}

const getReturnClass = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return ''
  if (value > 0) return 'return-positive'
  if (value < 0) return 'return-negative'
  return ''
}

const getMetricClass = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return ''
  if (value > 0) return 'metric-positive'
  if (value < 0) return 'metric-negative'
  return ''
}

const getSharpeClass = (value) => {
  if (value === null || value === undefined || Number.isNaN(value)) return ''
  if (value > 0) return 'metric-negative'
  if (value < 0) return 'metric-positive'
  return ''
}

const portfolioMetrics = computed(() => {
  const values = getPortfolioSeries()
  if (values.length < 2) {
    return {
      returnRatio: null,
      sharpe: null,
      drawdown: null
    }
  }
  const base = values[0]
  const tail = values[values.length - 1]
  const returnRatio = base ? tail / base - 1 : null
  const returns = []
  for (let i = 1; i < values.length; i += 1) {
    const prev = values[i - 1]
    const curr = values[i]
    if (!prev) continue
    returns.push(curr / prev - 1)
  }
  const mean = returns.length ? returns.reduce((sum, val) => sum + val, 0) / returns.length : null
  const variance = returns.length
    ? returns.reduce((sum, val) => sum + (val - mean) ** 2, 0) / returns.length
    : null
  const std = variance !== null ? Math.sqrt(variance) : null
  const sharpe = std ? (mean / std) * Math.sqrt(252) : null
  let peak = values[0]
  let maxDrawdown = 0
  values.forEach((value) => {
    if (value > peak) peak = value
    const drawdown = peak ? (value - peak) / peak : 0
    if (drawdown < maxDrawdown) maxDrawdown = drawdown
  })
  return {
    returnRatio,
    sharpe,
    drawdown: maxDrawdown
  }
})

const getSortValue = (item) => {
  const codeKey = getCodeKey(item)
  if (sortKey.value === 'code') return codeKey
  if (sortKey.value === 'name') return fundNameMap.value[codeKey] || ''
  if (sortKey.value === 'shares') return Number(item.shares) || 0
  if (sortKey.value === 'nav') return fundUnitValueMap.value[codeKey] ?? null
  if (sortKey.value === 'return') return getFundReturnRate(item)
  if (sortKey.value === 'sharpe') return getFundSharpe(item)
  if (sortKey.value === 'drawdown') return getFundDrawdown(item)
  if (sortKey.value === 'mddDuration') return getFundDrawdownDays(item)
  const unitValue = fundUnitValueMap.value[codeKey]
  if (!unitValue) return null
  return unitValue * (Number(item.shares) || 0)
}

const getFundAmount = (item) => {
  const codeKey = getCodeKey(item)
  const unitValue = fundUnitValueMap.value[codeKey]
  const shares = Number(item.shares)
  if (!unitValue || !Number.isFinite(shares) || shares <= 0) return null
  return unitValue * shares
}

const portfolioSummary = computed(() => {
  const items = portfolioItems.value
    .map((item) => ({
      code: String(item.code || '').trim(),
      shares: Number(item.shares)
    }))
    .filter((item) => item.code && Number.isFinite(item.shares) && item.shares > 0)
  const fundCount = items.length
  const totalAmount = items.reduce((sum, item) => {
    const unitValue = fundUnitValueMap.value[item.code]
    if (!unitValue) return sum
    return sum + unitValue * item.shares
  }, 0)
  return {
    fundCount,
    totalAmount: fundCount ? totalAmount : null
  }
})

const sortedPortfolioItems = computed(() => {
  const items = portfolioItems.value.map((item, index) => ({
    item,
    index
  }))
  if (!sortKey.value || !sortOrder.value) {
    return items
  }
  return items.slice().sort((a, b) => {
    const aVal = getSortValue(a.item)
    const bVal = getSortValue(b.item)
    const isString = sortKey.value === 'code' || sortKey.value === 'name'
    if (aVal === undefined || aVal === null || aVal === '') return 1
    if (bVal === undefined || bVal === null || bVal === '') return -1
    let result = 0
    if (isString) {
      result = String(aVal).localeCompare(String(bVal), 'zh-Hans-CN', { numeric: true })
    } else {
      result = Number(aVal) - Number(bVal)
    }
    return sortOrder.value === 'asc' ? result : -result
  })
})

const toggleSort = (key) => {
  if (sortKey.value !== key) {
    sortKey.value = key
    sortOrder.value = 'asc'
    return
  }
  if (sortOrder.value === 'asc') {
    sortOrder.value = 'desc'
    return
  }
  if (sortOrder.value === 'desc') {
    sortKey.value = ''
    sortOrder.value = ''
    return
  }
  sortOrder.value = 'asc'
}

const fetchData = async () => {
  const items = portfolioItems.value
    .map((item) => ({
      code: String(item.code || '').trim(),
      shares: Number(item.shares)
    }))
    .filter((item) => item.code && Number.isFinite(item.shares) && item.shares > 0)
  if (!items.length) {
    error.value = 'Enter at least one fund and shares.'
    return
  }
  savePortfolio()
  loading.value = true
  error.value = ''
  debugInfo.value = null
  portfolioData.value = null
  warnings.value = []
  try {
    const response = await axios.post('/api/portfolio', {
      items,
      start_date: startDate.value,
      end_date: endDate.value
    })
    const data = response.data?.portfolio?.data || []
    debugInfo.value = response.data?.debug_info || null
    warnings.value = response.data?.warnings || []
    if (!data.length) {
      error.value = 'No data found.'
      return
    }
    portfolioData.value = response.data
    updateFundUnitValues()
    await nextTick()
    renderChart(data)
  } catch (err) {
    const details = err?.response?.data?.details || []
    if (details.length) {
      error.value = `${err?.response?.data?.error || 'Request failed'}: ${details.map((item) => `${item.code}: ${item.error}`).join(', ')}`
    } else {
      error.value = err?.response?.data?.error || err?.message || 'Request failed'
    }
  } finally {
    loading.value = false
  }
}

const handleResize = () => {
  if (chartInstance) {
    chartInstance.resize()
  }
}

const addItem = () => {
  portfolioItems.value.push({ code: '', shares: 0 })
}

const removeItem = (index) => {
  if (portfolioItems.value.length === 1) return
  portfolioItems.value.splice(index, 1)
}

const clearItems = () => {
  portfolioItems.value = [{ code: '', shares: 0 }]
  portfolioData.value = null
  warnings.value = []
  error.value = ''
  fetchFundNames()
  updateFundUnitValues()
}

onMounted(() => {
  initDates()
  const hasSaved = loadSavedPortfolio()
  loadColumnWidths()
  const savedTheme = localStorage.getItem(THEME_KEY)
  if (savedTheme === 'dark' || savedTheme === 'light') {
    theme.value = savedTheme
  } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    theme.value = 'dark'
  }
  applyTheme(theme.value)
  fetchData()
  fetchFundNames()
  window.addEventListener('resize', handleResize)
})

watch([portfolioItems, startDate, endDate, sortKey, sortOrder], () => {
  savePortfolio()
}, { deep: true })

watch([startDate, endDate], () => {
  normalizeDateRange()
})

watch(portfolioItems, (nextItems, prevItems) => {
  const nextCodeList = nextItems.map((item) => String(item.code || '').trim())
  if (suppressFundNameWatch.value) {
    lastCodeSnapshot.value = nextCodeList
    return
  }
  const prevCodeList = lastCodeSnapshot.value || []
  const nextCodes = Array.from(new Set(getValidCodes(nextItems)))
  const prevCodes = Array.from(new Set(prevCodeList.filter((code) => /^\d{6}$/.test(code))))
  const nextSet = new Set(nextCodes)
  const prevSet = new Set(prevCodes)
  const added = nextCodes.filter((code) => !prevSet.has(code))
  const removed = prevCodes.filter((code) => !nextSet.has(code))
  if (removed.length) {
    const nextNameMap = {}
    Object.keys(fundNameMap.value).forEach((code) => {
      if (nextSet.has(code)) {
        nextNameMap[code] = fundNameMap.value[code]
      }
    })
    const nextUrlMap = {}
    Object.keys(fundNameUrlMap.value).forEach((code) => {
      if (nextSet.has(code)) {
        nextUrlMap[code] = fundNameUrlMap.value[code]
      }
    })
    const nextErrors = {}
    Object.keys(fundNameErrors.value).forEach((code) => {
      if (nextSet.has(code)) {
        nextErrors[code] = fundNameErrors.value[code]
      }
    })
    fundNameMap.value = nextNameMap
    fundNameUrlMap.value = nextUrlMap
    fundNameErrors.value = nextErrors
    savePortfolio()
  }
  const newlyCompleted = nextCodeList
    .map((code, index) => {
      const prevCode = prevCodeList[index] || ''
      if (!/^\d{6}$/.test(code)) return null
      if (code === prevCode) return null
      return code
    })
    .filter(Boolean)
  if (newlyCompleted.length) {
    const missing = Array.from(new Set(newlyCompleted))
      .filter((code) => !fundNameMap.value[code])
    if (missing.length) {
      fetchFundNames(missing)
    }
  }
  updateFundUnitValues()
  lastCodeSnapshot.value = nextCodeList
}, { deep: true })

watch(endDate, () => {
  updateFundUnitValues()
})

watch(chartMode, () => {
  const data = portfolioData.value?.portfolio?.data || []
  if (data.length) {
    renderChart(data)
  }
})

watch(theme, (value) => {
  applyTheme(value)
  localStorage.setItem(THEME_KEY, value)
  const data = portfolioData.value?.portfolio?.data || []
  if (data.length) {
    renderChart(data)
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize)
  if (chartInstance) {
    chartInstance.dispose()
    chartInstance = null
  }
})
</script>

<template>
  <div class="page">
    <div class="header-bar">
      <div class="header-block">
        <div class="header">G-001 Portfolio Insiights</div>
        <div class="header-subtitle">未来ガジェット研究所</div>
      </div>
      <button class="theme-toggle" @click="toggleTheme">
        {{ theme === 'dark' ? 'Light Mode' : 'Dark Mode' }}
      </button>
    </div>
    <div class="chart-card">
      <div class="chart-toolbar">
        <button
          class="chip"
          :class="{ active: chartMode === 'LJJZ' }"
          @click="chartMode = 'LJJZ'"
        >
          Cumulate
        </button>
        <button
          class="chip"
          :class="{ active: chartMode === 'DWJZ' }"
          @click="chartMode = 'DWJZ'"
        >
          Net
        </button>
        <div class="chart-metrics">
          <div>
            Return Ratio:
            <span :class="getReturnClass(portfolioMetrics.returnRatio)">
              {{ formatReturnPercent(portfolioMetrics.returnRatio) }}
            </span>
          </div>
          <div>
            Sharpe Ratio:
            <span :class="getReturnClass(portfolioMetrics.sharpe)">
              {{ formatMetric(portfolioMetrics.sharpe) }}
            </span>
          </div>
          <div>Max Drawdown: {{ formatReturnPercentAbs(portfolioMetrics.drawdown) }}</div>
        </div>
      </div>
      <div ref="chartEl" class="chart"></div>
    </div>
    <div class="panel">
      <div class="form">
        <label class="field date-field">
          <span>Start Date</span>
          <div class="date-control">
            <span class="date-icon">📅</span>
            <input v-model="startDate" type="date" class="date-input" />
          </div>
        </label>
        <label class="field date-field">
          <span>End Date</span>
          <div class="date-control">
            <span class="date-icon">📅</span>
            <input v-model="endDate" type="date" class="date-input" />
          </div>
        </label>
        <button class="primary" :disabled="loading" @click="fetchData">
          {{ loading ? 'Loading...' : 'Calculate Portfolio' }}
        </button>
      </div>
      <div class="portfolio-summary">
        <div>Funds: {{ portfolioSummary.fundCount }}</div>
        <div class="total-amount">
          <span>Total Amount: {{ showTotalAmount ? formatAmount(portfolioSummary.totalAmount) : '***' }}</span>
          <button class="amount-toggle" type="button" @click="showTotalAmount = !showTotalAmount">👀</button>
        </div>
      </div>
      <div class="portfolio" :style="columnStyle">
        <div class="portfolio-toolbar">
          <button class="sort-order" @click="clearItems">
            Clear All
          </button>
          <button class="sort-order" @click="exportCsv">
            Export CSV
          </button>
          <label class="sort-order import-label">
            Import CSV
            <input type="file" accept=".csv" @change="importCsv($event)" />
          </label>
        </div>
        <div class="portfolio-head">
          <div class="sort-header header-cell" @click="toggleSort('code')">
            Code
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'code' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'code' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('code', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('shares')">
            Shares
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'shares' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'shares' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('shares', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('name')">
            Fund Name
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'name' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'name' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('name', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('nav')">
            NAV
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'nav' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'nav' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('nav', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('amount')">
            Position
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'amount' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'amount' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('position', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('return')">
            Return
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'return' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'return' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('return', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('sharpe')">
            Sharpe
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'sharpe' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'sharpe' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('sharpe', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('drawdown')">
            MDD
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'drawdown' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'drawdown' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('mdd', $event)"></span>
          </div>
          <div class="sort-header header-cell" @click="toggleSort('mddDuration')">
            MDD Duration
            <span class="sort-icons">
              <span :class="{ active: sortKey === 'mddDuration' && sortOrder === 'asc' }">▲</span>
              <span :class="{ active: sortKey === 'mddDuration' && sortOrder === 'desc' }">▼</span>
            </span>
            <span class="col-resizer" @mousedown="startResize('mddDuration', $event)"></span>
          </div>
          <div class="header-cell action-header">
            <span class="col-resizer" @mousedown="startResize('action', $event)"></span>
          </div>
        </div>
        <div v-for="entry in sortedPortfolioItems" :key="entry.index" class="portfolio-row">
          <input v-model="entry.item.code" placeholder="e.g. 015202" />
          <input
            v-if="showTotalAmount"
            v-model.number="entry.item.shares"
            type="number"
            min="0"
            step="1"
          />
          <div v-else class="portfolio-meta">
            ***
          </div>
          <div class="portfolio-meta">
            <a
              v-if="getCodeKey(entry.item)"
              class="fund-link"
              :href="`https://fundf10.eastmoney.com/jjjz_${getCodeKey(entry.item)}.html`"
              target="_blank"
              rel="noreferrer"
            >
              {{ fundNameMap[getCodeKey(entry.item)] || '-' }}
            </a>
            <span v-else>{{ fundNameMap[getCodeKey(entry.item)] || '-' }}</span>
          </div>
          <div class="portfolio-meta">
            {{ formatMetric(fundUnitValueMap[getCodeKey(entry.item)]) }}
          </div>
          <div class="portfolio-meta">
            {{ showTotalAmount ? formatAmount(getFundAmount(entry.item)) : '***' }}
          </div>
          <div class="portfolio-meta" :class="getReturnClass(getFundReturnRate(entry.item))">
            {{ formatReturnPercent(getFundReturnRate(entry.item)) }}
          </div>
          <div class="portfolio-meta" :class="getSharpeClass(getFundSharpe(entry.item))">
            {{ formatMetric(getFundSharpe(entry.item)) }}
          </div>
          <div class="portfolio-meta mdd-value">
            {{ formatReturnPercentAbs(getFundDrawdown(entry.item)) }}
          </div>
          <div class="portfolio-meta mdd-days">
            {{ formatDrawdownDays(getFundDrawdownDays(entry.item)) }}
          </div>
          <button class="ghost delete-button" :disabled="portfolioItems.length === 1" @click="removeItem(entry.index)">
            ✕
          </button>
        </div>
        <button class="add" @click="addItem">Add Fund</button>
      </div>
      <div v-if="error" class="error">{{ error }}</div>
    </div>
    <div v-if="warnings.length" class="panel warning">
      <div class="summary-title">Some Funds Failed</div>
      <div v-for="item in warnings" :key="item.code" class="warning-item">
        {{ item.code }}: {{ item.error }}
      </div>
    </div>
    <details v-if="debugInfo" class="debug">
      <summary>Debug Info</summary>
      <pre>{{ debugInfo }}</pre>
    </details>
  </div>
</template>
