import { CustomSelect } from './CustomSelect'

const tpuTypes = ['none', 'v6e', 'trillium', 'v4', 'tpu-v6e-slice', 'tpu7x']

interface TpuSpecControlProps {
  label: string
  value: string
  onChange: (value: string) => void
  labels?: {
    notConfigured: string
    topologyX: string
    topologyY: string
    topologyZ: string
    type: string
  }
}

interface ParsedTpuSpec {
  type: string
  x: string
  y: string
  z: string
}

export function TpuSpecControl({ label, labels, value, onChange }: TpuSpecControlProps) {
  const parsed = parseTpuSpec(value)
  const controlLabels = labels ?? {
    notConfigured: 'Not configured',
    topologyX: 'Topology X',
    topologyY: 'Topology Y',
    topologyZ: 'Topology Z',
    type: 'TPU type',
  }
  const update = (next: Partial<ParsedTpuSpec>) => {
    onChange(formatTpuSpec({ ...parsed, ...next }))
  }

  return (
    <div className="tpu-spec-control field-wide">
      <span className="tpu-spec-label">{label}</span>
      <label>
        {controlLabels.type}
        <CustomSelect
          ariaLabel={controlLabels.type}
          value={parsed.type}
          options={tpuTypes.map((type) => ({ label: type === 'none' ? controlLabels.notConfigured : type, value: type }))}
          onChange={(type) => update({ type })}
        />
      </label>
      <label>
        {controlLabels.topologyX}
        <input
          disabled={parsed.type === 'none'}
          min="1"
          type="number"
          value={parsed.x}
          onChange={(event) => update({ x: event.target.value })}
        />
      </label>
      <label>
        {controlLabels.topologyY}
        <input
          disabled={parsed.type === 'none'}
          min="1"
          type="number"
          value={parsed.y}
          onChange={(event) => update({ y: event.target.value })}
        />
      </label>
      <label>
        {controlLabels.topologyZ}
        <input
          disabled={parsed.type === 'none'}
          min="1"
          placeholder="optional"
          type="number"
          value={parsed.z}
          onChange={(event) => update({ z: event.target.value })}
        />
      </label>
    </div>
  )
}

function parseTpuSpec(value: string): ParsedTpuSpec {
  if (!value || value === 'none') return { type: 'none', x: '', y: '', z: '' }
  const [type, topology = ''] = value.split('=')
  const [x = '', y = '', z = ''] = topology.split('x')
  return { type: tpuTypes.includes(type) ? type : 'v6e', x, y, z }
}

function formatTpuSpec(spec: ParsedTpuSpec) {
  if (spec.type === 'none') return 'none'
  if (!spec.x || !spec.y) return 'none'
  const topology = [spec.x, spec.y, spec.z].filter(Boolean).join('x')
  return `${spec.type}=${topology}`
}
