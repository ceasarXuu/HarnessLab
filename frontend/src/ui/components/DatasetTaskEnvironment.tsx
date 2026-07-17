import type { DatasetTaskEnvironmentDto } from '../../api/contract'
import type { Translate } from '../../i18n'

interface DatasetTaskEnvironmentProps {
  environment: DatasetTaskEnvironmentDto | null
  t: Translate
}

export function DatasetTaskEnvironment({ environment, t }: DatasetTaskEnvironmentProps) {
  if (!environment) return <p className="task-environment-empty">{t('taskEnvironmentUnavailable')}</p>

  const resources = formatResources(environment, t)
  return (
    <div className="task-environment">
      <h4>{t('taskEnvironment')}</h4>
      <dl className="task-environment-grid">
        <EnvironmentValue label={t('operatingSystem')} value={environment.os} />
        <EnvironmentValue label={t('environmentDefinition')} value={environment.definitions.map((item) => t(definitionLabels[item])).join(', ') || '-'} />
        {environment.containerImages.length === 0 && <EnvironmentValue wide label={t('containerImages')} value="-" />}
        {environment.containerImages.map((image) => (
          <ContainerImage key={`${image.source}:${image.reference}`} image={image} t={t} />
        ))}
        <EnvironmentValue label={t('buildTimeout')} value={`${environment.buildTimeoutSeconds}s`} />
        <EnvironmentValue label={t('networkMode')} value={t(networkLabels[environment.networkMode])} />
        {environment.allowedHosts.length > 0 && <EnvironmentValue wide label={t('environmentAllowedHosts')} value={environment.allowedHosts.join(', ')} />}
        <EnvironmentValue label={t('resourceRequirements')} value={resources} />
        <EnvironmentValue label={t('workingDirectory')} value={environment.workdir ?? '-'} />
      </dl>
    </div>
  )
}

function ContainerImage({ image, t }: {
  image: DatasetTaskEnvironmentDto['containerImages'][number]
  t: Translate
}) {
  const label = image.source === 'dockerfile-base' ? t('dockerfileBaseImage') : t('prebuiltEnvironmentImage')
  return (
    <>
      <EnvironmentValue wide label={label} value={image.reference} />
      {image.platforms !== null && (
        <EnvironmentValue wide label={t('imagePlatforms')} value={image.platforms.join(', ') || t('unknown')} />
      )}
    </>
  )
}

function EnvironmentValue({ label, value, wide = false }: { label: string; value: string; wide?: boolean }) {
  return (
    <div className={wide ? 'task-environment-value wide' : 'task-environment-value'}>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  )
}

const definitionLabels = {
  'docker-compose': 'environmentDefinitionCompose',
  'docker-image': 'environmentDefinitionImage',
  dockerfile: 'environmentDefinitionDockerfile',
} as const

const networkLabels = {
  allowlist: 'networkAllowlist',
  'no-network': 'networkDisabled',
  public: 'networkPublic',
} as const

function formatResources(environment: DatasetTaskEnvironmentDto, t: Translate): string {
  const { resources } = environment
  const values = [
    resources.cpus === null ? null : `CPU ${resources.cpus}`,
    resources.memoryMb === null ? null : `${t('memory')} ${resources.memoryMb} MB`,
    resources.storageMb === null ? null : `${t('storage')} ${resources.storageMb} MB`,
    resources.gpus === null ? null : `GPU ${resources.gpus}${resources.gpuTypes.length ? ` (${resources.gpuTypes.join(', ')})` : ''}`,
    resources.tpu === null ? null : `TPU ${resources.tpu.type} ${resources.tpu.topology}`,
  ].filter(Boolean)
  return values.join(', ') || t('notSpecified')
}
