import { useEffect, useState } from 'react'
import type { ApiResponse, ModelPricingPreviewDto } from '../../api/contract'
import type { AgentRow, ModelPricing, ModelPricingSource } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { EditableStringList } from './EditableStringList'
import { Metric } from './Metric'

interface AgentModelSettingsProps {
  loadPricing?: (modelName: string) => Promise<ApiResponse<ModelPricingPreviewDto | null>>
  readOnly?: boolean
  t: Translate
  value: AgentRow
  onChange: (value: AgentRow) => void
}

export function AgentModelSettings({ loadPricing, readOnly = false, t, value, onChange }: AgentModelSettingsProps) {
  const [modelRows, setModelRows] = useState(() => parseModelNames(value.models))
  const [previews, setPreviews] = useState<Record<string, ModelPricingPreviewDto | null>>({})
  const models = readOnly
    ? parseModelNames(value.models)
    : modelRows.map((model) => model.trim()).filter(Boolean)
  const modelKey = models.join('\n')
  useEffect(() => {
    if (!loadPricing || !modelKey) return
    let active = true
    void Promise.all(models.map(async (model) => {
      const response = await loadPricing(model)
      return [model, response.data] as const
    })).then((entries) => {
      if (active) setPreviews(Object.fromEntries(entries))
    })
    return () => { active = false }
  }, [loadPricing, modelKey])
  const updateModels = (nextModels: string[]) => {
    setModelRows(nextModels)
    const cleanModels = nextModels.map((model) => model.trim()).filter(Boolean)
    onChange({
      ...value,
      models: cleanModels.join(', '),
      modelPricing: synchronizePricing(parseModelNames(value.models), cleanModels, value.modelPricing),
    })
  }
  const updatePricing = (modelName: string, next: ModelPricing) => {
    onChange({
      ...value,
      modelPricing: models.map((model) => model === modelName
        ? next
        : pricingForModel(model, value.modelPricing)),
    })
  }

  if (readOnly) {
    return (
      <div className="model-settings field-wide">
        <Metric label={t('supportedModels')} value={models.length ? models.join(', ') : t('configuredAtJobRun')} />
        {models.map((model) => {
          const pricing = pricingForModel(model, value.modelPricing)
          return (
            <div className="model-pricing-card" key={model}>
              <PricingSummary model={model} pricing={pricing} t={t} />
              {pricing.source !== 'custom' && (
                <ResolvedPricing pricing={pricing} preview={loadPricing ? previews[model] : null} t={t} />
              )}
            </div>
          )
        })}
      </div>
    )
  }

  return (
    <div className="model-settings field-wide">
      <div id="agent-models" tabIndex={-1}>
        <EditableStringList
          addLabel={t('add')}
          className="model-list-control"
          deleteLabel={t('delete')}
          itemAriaLabel={() => t('modelName')}
          label={t('supportedModels')}
          placeholder={t('modelNamePlaceholder')}
          values={modelRows}
          onChange={updateModels}
        />
      </div>
      {models.map((model) => (
        <ModelPricingEditor
          key={model}
          model={model}
          pricing={pricingForModel(model, value.modelPricing)}
          preview={loadPricing ? previews[model] : null}
          t={t}
          onChange={(next) => updatePricing(model, next)}
        />
      ))}
    </div>
  )
}

function ModelPricingEditor({ model, pricing, preview, t, onChange }: {
  model: string
  pricing: ModelPricing
  preview?: ModelPricingPreviewDto | null
  t: Translate
  onChange: (value: ModelPricing) => void
}) {
  const setSource = (source: string) => onChange({
    modelName: model,
    source: source as ModelPricingSource,
    ...(source === 'custom' ? customRates(pricing) : {}),
  })
  const setRate = (field: keyof Pick<ModelPricing,
    'inputCacheMissUsdPerMillion' | 'inputCacheHitUsdPerMillion' | 'outputUsdPerMillion'>, raw: string) => {
    onChange({ ...pricing, [field]: raw === '' ? undefined : Number(raw) })
  }
  return (
    <div className="model-pricing-card">
      <strong>{model}</strong>
      <label>
        {t('pricingSource')}
        <CustomSelect
          ariaLabel={`${t('pricingSource')}: ${model}`}
          options={pricingSourceOptions(t)}
          value={pricing.source}
          onChange={setSource}
        />
      </label>
      {pricing.source === 'custom' && (
        <div className="model-pricing-rates">
          <PriceInput label={t('inputCacheMissPrice')} value={pricing.inputCacheMissUsdPerMillion} onChange={(raw) => setRate('inputCacheMissUsdPerMillion', raw)} />
          <PriceInput label={t('inputCacheHitPrice')} value={pricing.inputCacheHitUsdPerMillion} onChange={(raw) => setRate('inputCacheHitUsdPerMillion', raw)} />
          <PriceInput label={t('outputPrice')} value={pricing.outputUsdPerMillion} onChange={(raw) => setRate('outputUsdPerMillion', raw)} />
        </div>
      )}
      {pricing.source !== 'custom' && (
        <ResolvedPricing pricing={pricing} preview={preview} t={t} />
      )}
    </div>
  )
}

function ResolvedPricing({ pricing, preview, t }: {
  pricing: ModelPricing
  preview?: ModelPricingPreviewDto | null
  t: Translate
}) {
  if (preview === undefined) return <p className="model-pricing-note">{t('pricingLoading')}</p>
  if (preview === null) return <p className="model-pricing-note warning-copy">{t('pricingUnavailable')}</p>
  return (
    <>
      <div className="model-pricing-rates readonly">
        <PriceValue label={t('inputCacheMissPrice')} value={preview.inputCacheMissUsdPerMillion} />
        <PriceValue label={t('inputCacheHitPrice')} value={preview.inputCacheHitUsdPerMillion} />
        <PriceValue label={t('outputPrice')} value={preview.outputUsdPerMillion} />
      </div>
      <p className="model-pricing-note">
        {pricing.source === 'reported' ? t('pricingHarnessReferenceNote') : t('pricingLiteLlmNote')}
      </p>
    </>
  )
}

function PriceValue({ label, value }: { label: string; value: number }) {
  return <div><span>{label}</span><strong>{formatPrice(value)}</strong></div>
}

function PriceInput({ label, value, onChange }: { label: string; value?: number; onChange: (value: string) => void }) {
  return (
    <label>
      {label}
      <input min="0" step="any" type="number" value={value ?? ''} onChange={(event) => onChange(event.target.value)} />
    </label>
  )
}

function formatPrice(value: number) {
  return `$${value.toLocaleString(undefined, { maximumFractionDigits: 12 })}`
}

function PricingSummary({ model, pricing, t }: { model: string; pricing: ModelPricing; t: Translate }) {
  const source = pricingSourceOptions(t).find((option) => option.value === pricing.source)?.label ?? pricing.source
  const rates = pricing.source === 'custom'
    ? `${pricing.inputCacheMissUsdPerMillion} / ${pricing.inputCacheHitUsdPerMillion} / ${pricing.outputUsdPerMillion}`
    : source
  return <Metric label={`${model} · ${t('pricingSource')}`} value={rates} />
}

function pricingSourceOptions(t: Translate) {
  return [
    { label: t('pricingSourceReported'), value: 'reported' },
    { label: t('pricingSourceLiteLlm'), value: 'litellm' },
    { label: t('pricingSourceCustom'), value: 'custom' },
  ]
}

function pricingForModel(modelName: string, pricing: ModelPricing[]) {
  return pricing.find((item) => item.modelName === modelName)
    ?? { modelName, source: 'reported' as const }
}

function synchronizePricing(previousModels: string[], nextModels: string[], pricing: ModelPricing[]) {
  return nextModels.map((model, index) => {
    const exact = pricing.find((item) => item.modelName === model)
    if (exact) return exact
    const previous = pricingForModel(previousModels[index] ?? '', pricing)
    return { ...previous, modelName: model }
  })
}

function customRates(pricing: ModelPricing) {
  return {
    inputCacheMissUsdPerMillion: pricing.inputCacheMissUsdPerMillion ?? 0,
    inputCacheHitUsdPerMillion: pricing.inputCacheHitUsdPerMillion ?? 0,
    outputUsdPerMillion: pricing.outputUsdPerMillion ?? 0,
  }
}

function parseModelNames(value: string) {
  if (!value || value === '-') return []
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}
