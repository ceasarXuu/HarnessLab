import { CheckCircle2, Play, ScrollText } from 'lucide-react'
import { RunBuilder } from '../components/RunBuilder'
import type { RunDraft } from '../data/demo'
import type { Translate } from '../i18n'

interface NewRunPageProps {
  activeStep: string
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
  onStep: (step: string) => void
}

export function NewRunPage({ activeStep, draft, t, onDraft, onLaunch, onStep }: NewRunPageProps) {
  return (
    <main className="workspace two-column-page">
      <div className="content-column">
        <RunBuilder
          activeStep={activeStep}
          draft={draft}
          t={t}
          onDraft={onDraft}
          onLaunch={onLaunch}
          onStep={onStep}
        />
      </div>
      <aside className="detail-rail">
        <section className="surface rail-card">
          <div className="rail-title">
            <ScrollText aria-hidden="true" />
            <h3>{t('createFlow')}</h3>
          </div>
          <p>{t('createFlowDesc')}</p>
          <div className="stage-list">
            <span className={activeStep === 'Source' ? 'active' : undefined}>{t('source')}</span>
            <span className={activeStep === 'Agent' ? 'active' : undefined}>{t('agent')}</span>
            <span className={activeStep === 'Environment' ? 'active' : undefined}>{t('environment')}</span>
            <span className={activeStep === 'Runtime' ? 'active' : undefined}>{t('runtime')}</span>
            <span className={activeStep === 'Review' ? 'active' : undefined}>{t('review')}</span>
          </div>
        </section>
        <section className="surface rail-card">
          <div className="rail-title">
            <Play aria-hidden="true" />
            <h3>{t('runFlow')}</h3>
          </div>
          <p>{t('runFlowDesc')}</p>
          <ul className="doctor-list">
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('jobConfig')}
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              harbor run
            </li>
          </ul>
        </section>
      </aside>
    </main>
  )
}
