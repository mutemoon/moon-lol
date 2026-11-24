export function sendWasmCommand(iframeWindow: Window | null, command: any) {
  if (!iframeWindow) return
  const msg = typeof command === 'string' ? command : JSON.stringify(command)
  iframeWindow.postMessage(msg, '*')
}

export function playVfxRon(iframeWindow: Window | null, ron: string) {
  sendWasmCommand(iframeWindow, {
    type: 'PLAY_RON',
    ron,
  })
}

export function playSingleEmitter(iframeWindow: Window | null, ron: string, index: number) {
  sendWasmCommand(iframeWindow, {
    type: 'PLAY_EMITTER',
    ron,
    index,
  })
}

export function stopVfx(iframeWindow: Window | null) {
  sendWasmCommand(iframeWindow, {
    type: 'STOP',
  })
}

export function resetCamera(iframeWindow: Window | null) {
  sendWasmCommand(iframeWindow, {
    type: 'RESET_CAMERA',
  })
}

/**
 * 辅助读取发射器采样器的数值分量（保证返回的每一项均为纯数字）
 */
export function extractSamplerValue(sampler: any): number[] {
  if (sampler === undefined || sampler === null) return [0, 0, 0]
  if (typeof sampler === 'number' && !isNaN(sampler)) return [sampler]

  let base = sampler.base_sampler !== undefined ? sampler.base_sampler : sampler

  // 处理被包装在 { Constant: ... } 或 { Curve: ... } 的情况
  if (base && typeof base === 'object' && !Array.isArray(base)) {
    if (base.Constant !== undefined) base = base.Constant
    else if (base.Curve !== undefined && base.Curve.samples) base = base.Curve.samples
  }

  // 如果 base 是 Curve 关键帧数组: [[t0, v0], [t1, v1], ...]
  if (
    Array.isArray(base) &&
    base.length > 0 &&
    Array.isArray(base[0]) &&
    base[0].length === 2 &&
    typeof base[0][0] === 'number'
  ) {
    const firstVal = base[0][1]
    if (typeof firstVal === 'number' && !isNaN(firstVal)) return [firstVal]
    if (Array.isArray(firstVal)) {
      return firstVal.map((x) => (typeof x === 'number' && !isNaN(x) ? x : 0))
    }
  }

  // 如果 base 是普通一维数值数组: [x, y, z] 或 [r, g, b, a]
  if (Array.isArray(base)) {
    return base.map((x) => (typeof x === 'number' && !isNaN(x) ? x : 0))
  }

  if (typeof base === 'number' && !isNaN(base)) return [base]

  return [0, 0, 0]
}

/**
 * 格式化数值显示
 */
export function formatNum(val: number | undefined): string {
  if (val === undefined || isNaN(val)) return '0'
  return Number(val.toFixed(3)).toString()
}
