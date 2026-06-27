import { Check, Copy, Play, Plus, RotateCcw } from 'lucide-react'
import type { RunDraft } from '../data/demo'

const steps = ['Source', 'Agent', 'Environment', 'Runtime', 'Review']

interface RunBuilderProps {
  draft: RunDraft
  activeStep: string
  onStep: (step: string) => void
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
}

export function RunBuilder({ draft, activeStep, onStep, onDraft, onLaunch }: RunBuilderProps) {
  return (
    <section className="surface run-builder" id="new-run">
      <div className="section-header compact">
        <div>
          <h2>New Run</h2>
          <p>Build a Harbor JobConfig without returning to CLI.</p>
        </div>
        <button className="secondary-button">
          <Copy aria-hidden="true" />
          JobConfig
        </button>
      </div>
      <div className="step-tabs" role="tablist" aria-label="JobConfig sections">
        {steps.map((step) => (
          <button
            key={step}
            className={step === activeStep ? 'active' : undefined}
            onClick={() => onStep(step)}
          >
            {step === activeStep && <Check aria-hidden="true" />}
            {step}
          </button>
        ))}
      </div>
      <div className="run-grid">
        <label>
          Source
          <select value={draft.source} onChange={(event) => onDraft({ ...draft, source: event.target.value })}>
            <option>terminal-bench@2.0</option>
            <option>swe-bench-lite</option>
            <option>harbor/hello-world</option>
          </select>
        </label>
        <label>
          Agent
          <select value={draft.agent} onChange={(event) => onDraft({ ...draft, agent: event.target.value })}>
            <option>claude-code</option>
            <option>codex-cli</option>
            <option>oracle</option>
          </select>
        </label>
        <label>
          Model
          <input value={draft.model} onChange={(event) => onDraft({ ...draft, model: event.target.value })} />
        </label>
        <label>
          Environment
          <select
            value={draft.environment}
            onChange={(event) => onDraft({ ...draft, environment: event.target.value })}
          >
            <option>docker</option>
            <option>local</option>
          </select>
        </label>
        <label>
          Concurrency
          <input
            type="number"
            min="1"
            value={draft.concurrency}
            onChange={(event) => onDraft({ ...draft, concurrency: Number(event.target.value) })}
          />
        </label>
        <label>
          Attempts
          <input
            type="number"
            min="1"
            value={draft.attempts}
            onChange={(event) => onDraft({ ...draft, attempts: Number(event.target.value) })}
          />
        </label>
      </div>
      <div className="config-preview">
        <code>
          harbor run --dataset {draft.source} --agent {draft.agent} --model {draft.model} --env {draft.environment}
        </code>
      </div>
      <div className="button-row">
        <button className="secondary-button">
          <RotateCcw aria-hidden="true" />
          Reset
        </button>
        <button className="secondary-button">
          <Plus aria-hidden="true" />
          Save template
        </button>
        <button className="primary-button" onClick={onLaunch}>
          <Play aria-hidden="true" />
          Run job
        </button>
      </div>
    </section>
  )
}
