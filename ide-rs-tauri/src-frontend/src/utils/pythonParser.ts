import type { ScriptLine } from '../types/script'

/**
 * Python version detection result
 */
export type PythonVersion = 'python2' | 'python3' | 'unknown'

/**
 * Sikuli-D Simple mode header marker
 * Simpleモードで作成されたスクリプトを識別するためのヘッダーマーカー
 */
export const SIMPLE_MODE_MARKER = '# Sikuli-D Simple Mode'

/**
 * Parse result including script and detected Python version
 */
export interface ParseResult {
  script: ScriptLine[]
  pythonVersion: PythonVersion
}

/**
 * Detect if script was created in Simple mode
 * Check for Simple mode marker within first 100 lines (like UTF-8 encoding declaration)
 * スクリプトがSimpleモードで作成されたかを検出
 * 最初の100行以内にSimpleモードマーカーがあるかをチェック（UTF-8エンコーディング宣言と同じ考え方）
 */
export function isSimpleModeScript(pythonCode: string): boolean {
  const lines = pythonCode.split('\n')
  const linesToCheck = Math.min(lines.length, 100)

  for (let i = 0; i < linesToCheck; i++) {
    if (lines[i].includes(SIMPLE_MODE_MARKER)) {
      return true
    }
  }
  return false
}

/**
 * Add Simple mode header to source code if not already present
 * Simpleモードヘッダーをソースコードに追加（まだない場合）
 */
export function addSimpleModeHeader(pythonCode: string): string {
  if (isSimpleModeScript(pythonCode)) {
    return pythonCode
  }

  // Add marker after encoding declaration if present, otherwise at the top
  // エンコーディング宣言の後に追加、なければ先頭に追加
  const lines = pythonCode.split('\n')
  let insertIndex = 0

  // Skip shebang and encoding lines
  for (let i = 0; i < Math.min(lines.length, 2); i++) {
    if (lines[i].startsWith('#!') || /^#.*coding[:=]/.test(lines[i])) {
      insertIndex = i + 1
    }
  }

  lines.splice(insertIndex, 0, SIMPLE_MODE_MARKER)
  return lines.join('\n')
}

/**
 * Generate unique ID
 */
const generateId = (): string => {
  return Date.now().toString() + '-' + Math.random().toString(36).substring(2, 11)
}

/**
 * Detect Python version from source code
 * Python 2 indicators: print without parens, xrange, raw_input, except X, e:, unicode/basestring
 * Python 3 indicators: print(), f-strings, async/await, type hints, except X as e:
 */
export function detectPythonVersion(pythonCode: string): PythonVersion {
  let python2Score = 0
  let python3Score = 0

  // Python 2 indicators
  if (/\bprint\s+["'][^)]+$/.test(pythonCode)) python2Score += 3  // print "..." without parens
  if (/\bprint\s+[^("'\s]/.test(pythonCode)) python2Score += 3   // print var (no parens)
  if (/\bxrange\s*\(/.test(pythonCode)) python2Score += 2
  if (/\braw_input\s*\(/.test(pythonCode)) python2Score += 2
  if (/except\s+\w+\s*,\s*\w+\s*:/.test(pythonCode)) python2Score += 3  // except Exception, e:
  if (/\bunicode\s*\(/.test(pythonCode)) python2Score += 2
  if (/\bbasestring\b/.test(pythonCode)) python2Score += 2
  if (/\bexecfile\s*\(/.test(pythonCode)) python2Score += 2
  if (/^#.*coding[:=]\s*(utf-8|ascii)/m.test(pythonCode)) python2Score += 1  // encoding declaration (more common in py2)

  // Python 3 indicators
  // Note: print() is NOT a strong Python 3 indicator because SikuliX/Jython supports print() in Python 2
  // print() だけでは Python 3 とは判断しない（SikuliX/Jython は Python 2 でも print() をサポート）
  if (/f["'].*\{.*\}.*["']/.test(pythonCode)) python3Score += 3  // f-strings
  if (/\basync\s+(def|for|with)\b/.test(pythonCode)) python3Score += 3
  if (/\bawait\s+/.test(pythonCode)) python3Score += 3
  if (/except\s+\w+\s+as\s+\w+\s*:/.test(pythonCode)) python3Score += 2  // except Exception as e:
  if (/def\s+\w+\s*\([^)]*\)\s*->\s*\w+/.test(pythonCode)) python3Score += 3  // type hints in functions
  if (/:\s*(int|str|float|bool|list|dict|None)\s*[=,)]/.test(pythonCode)) python3Score += 2  // variable type hints
  if (/\bnonlocal\s+/.test(pythonCode)) python3Score += 2
  if (/\binput\s*\(/.test(pythonCode)) python3Score += 1  // input() (exists in py3, was raw_input in py2)

  // Return version only when there's a clear difference
  // 明確な差がある場合のみバージョンを返す
  if (python2Score > python3Score + 1) return 'python2'
  if (python3Score > python2Score + 1) return 'python3'
  return 'unknown'  // Ambiguous - return unknown / 不明確な場合は unknown を返す
}

/**
 * Parse SikuliX Python script to ScriptLine array
 */
export function parsePythonScript(pythonCode: string, folderPath: string): ScriptLine[] {
  const lines = pythonCode.split('\n')
  const result: ScriptLine[] = []

  // Add start node
  result.push({
    id: generateId(),
    type: 'start',
  })

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()

    // Skip empty lines and comments
    if (!line || line.startsWith('#') || line.startsWith('from ') || line.startsWith('import ')) {
      continue
    }

    // Parse Pattern with click - click(Pattern("image.png").similar(0.85)) or s.click(...)
    const clickPatternMatch = line.match(/\.?click\s*\(\s*Pattern\s*\(\s*["']([^"']+)["']\s*\)(?:\.similar\s*\(\s*([\d.]+)\s*\))?\s*\)/)
    if (clickPatternMatch) {
      const imagePath = `${folderPath}/${clickPatternMatch[1]}`
      result.push({
        id: generateId(),
        type: 'click',
        target: imagePath,
        similarity: clickPatternMatch[2] ? parseFloat(clickPatternMatch[2]) : 0.7,
      })
      continue
    }

    // Parse direct click with image - click("image.png") or s.click("image.png")
    const clickImageMatch = line.match(/\.?click\s*\(\s*["']([^"']+\.png)["']\s*\)/)
    if (clickImageMatch) {
      const imagePath = `${folderPath}/${clickImageMatch[1]}`
      result.push({
        id: generateId(),
        type: 'click',
        target: imagePath,
        similarity: 0.7,
      })
      continue
    }

    // Parse click with variable - click(target) or s.click(target)
    const clickVarMatch = line.match(/\.?click\s*\(\s*(\w+)\s*\)/)
    if (clickVarMatch && !clickVarMatch[1].includes('"') && !clickVarMatch[1].includes("'")) {
      result.push({
        id: generateId(),
        type: 'click',
        params: clickVarMatch[1],
      })
      continue
    }

    // Parse exists() with Pattern - target = s.exists(Pattern("image.png").similar(0.85), timeout)
    const existsPatternMatch = line.match(/(\w+)\s*=\s*\w*\.?exists\s*\(\s*Pattern\s*\(\s*["']([^"']+)["']\s*\)(?:\.similar\s*\(\s*([\d.]+)\s*\))?\s*(?:,\s*([\d.]+))?\s*\)/)
    if (existsPatternMatch) {
      const imagePath = `${folderPath}/${existsPatternMatch[2]}`
      result.push({
        id: generateId(),
        type: 'find',
        target: imagePath,
        similarity: existsPatternMatch[3] ? parseFloat(existsPatternMatch[3]) : 0.7,
        params: existsPatternMatch[4] || '3',
      })
      continue
    }

    // Parse simple exists - target = s.exists("image.png", timeout)
    const simpleExistsMatch = line.match(/(\w+)\s*=\s*\w*\.?exists\s*\(\s*["']([^"']+\.png)["']\s*(?:,\s*([\d.]+))?\s*\)/)
    if (simpleExistsMatch) {
      const imagePath = `${folderPath}/${simpleExistsMatch[2]}`
      result.push({
        id: generateId(),
        type: 'find',
        target: imagePath,
        similarity: 0.7,
        params: simpleExistsMatch[3] || '3',
      })
      continue
    }

    // Parse sleep() - sleep(seconds)
    const sleepMatch = line.match(/sleep\s*\(\s*([\d.]+)\s*\)/)
    if (sleepMatch) {
      result.push({
        id: generateId(),
        type: 'wait',
        params: sleepMatch[1],
      })
      continue
    }

    // Parse wait() with Pattern - wait(Pattern("image.png").similar(0.85), timeout)
    const waitPatternMatch = line.match(/\.?wait\s*\(\s*Pattern\s*\(\s*["']([^"']+)["']\s*\)(?:\.similar\s*\(\s*([\d.]+)\s*\))?\s*(?:,\s*([\d.]+))?\s*\)/)
    if (waitPatternMatch) {
      const imagePath = `${folderPath}/${waitPatternMatch[1]}`
      result.push({
        id: generateId(),
        type: 'wait',
        target: imagePath,
        similarity: waitPatternMatch[2] ? parseFloat(waitPatternMatch[2]) : 0.7,
        params: waitPatternMatch[3] || '10',
      })
      continue
    }

    // Parse simple wait - wait("image.png", timeout)
    const simpleWaitMatch = line.match(/\.?wait\s*\(\s*["']([^"']+\.png)["']\s*(?:,\s*([\d.]+))?\s*\)/)
    if (simpleWaitMatch) {
      const imagePath = `${folderPath}/${simpleWaitMatch[1]}`
      result.push({
        id: generateId(),
        type: 'wait',
        target: imagePath,
        similarity: 0.7,
        params: simpleWaitMatch[2] || '10',
      })
      continue
    }

    // Parse type() - type(text) or type(target, text)
    const typeMatch = line.match(/\.?type\s*\(\s*(?:(\w+)\s*,\s*)?["']([^"']*)["']\s*\)/)
    if (typeMatch) {
      result.push({
        id: generateId(),
        type: 'type',
        params: typeMatch[2],
        target: typeMatch[1] || undefined,
      })
      continue
    }

    // Parse while loop - while True: or while condition:
    const whileMatch = line.match(/while\s+(.+)\s*:/)
    if (whileMatch) {
      result.push({
        id: generateId(),
        type: 'loop',
        params: whileMatch[1],
        children: [],
      })
      continue
    }

    // Parse if statement - if condition:
    const ifMatch = line.match(/if\s+(.+)\s*:/)
    if (ifMatch) {
      result.push({
        id: generateId(),
        type: 'if',
        params: ifMatch[1],
        children: [],
      })
      continue
    }

    // Parse for loop - for x in ...:
    const forMatch = line.match(/for\s+\w+\s+in\s+(.+)\s*:/)
    if (forMatch) {
      result.push({
        id: generateId(),
        type: 'loop',
        params: forMatch[1],
        children: [],
      })
      continue
    }
  }

  return result
}
