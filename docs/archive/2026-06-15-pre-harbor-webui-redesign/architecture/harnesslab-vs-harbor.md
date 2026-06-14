# HarnessLab vs Harbor 架构对比

## 1. HarnessLab 当前架构

```mermaid
flowchart TB
    subgraph Benchmark["Benchmark Data"]
        BD["terminal-bench-core-0.1.1/hello-world/"]
    end

    subgraph HarnessLab_Core["HarnessLab Framework Core"]
        CLI["CLI: harnesslab run -agent claude-ds -benchmark terminal-bench"]
        AR["AgentRegistry load AgentProfile"]
        AD["TerminalBenchAdapter runtime plugin"]
        AP["AgentProfile: claude-ds.toml<br/>kind=claude-code<br/>command=claude-ds -p -<br/>inherit_env=[...]"]
    end

    subgraph External_Runtime["External Runner Black Box"]
        TB["terminal-bench tb run -agent claude-code"]
        TB_AgentInstall["install agent: standard claude CLI"]
        TB_Container["Docker Compose create container"]
        TB_AgentRun["agent exec: claude -p ..."]
        TB_Verifier["verifier exec: run-tests.sh"]
    end

    BD --> AD
    CLI --> AR
    AR --> AP
    AP -->|map kind to name| AD
    AD -->|invoke| TB
    TB --> TB_Container
    TB_Container --> TB_AgentInstall
    TB_AgentInstall --> TB_AgentRun
    TB_AgentRun --> TB_Verifier
    TB_Verifier -->|return result| TB
    TB -->|return result| AD
    AD -->|wrap result| CLI

    classDef pain fill:#ffcccc,stroke:#cc0000
    class TB,TB_AgentInstall,TB_AgentRun,AP pain
```

**Pain points (red)**:
- AgentProfile `command` and `env` ignored by external runner
- Adapter invokes external runner: runtime black box
- Custom agent (claude-ds) cannot take effect

---

## 2. Harbor Architecture

```mermaid
flowchart TB
    subgraph Benchmark["Benchmark Data"]
        BD["terminal-bench-core-0.1.1/hello-world/"]
    end

    subgraph Adapter_Phase["Adapter Phase (One-time)"]
        TA["TerminalBenchMapper data transformer"]
        TD["Task Directory:<br/>task.toml + instruction.md<br/>environment/Dockerfile<br/>tests/test.sh<br/>solution/solve.sh"]
    end

    subgraph Harbor_Core["Harbor Framework Core (Runtime)"]
        CLI2["CLI: harbor run -agent claude-code -dataset terminal-bench@2.0"]
        Job["Job creation"]
        Trial["Trial orchestration"]

        subgraph EnvLayer["Environment Layer"]
            EF["EnvironmentFactory"]
            BE["BaseEnvironment:<br/>start / stop / exec / upload_file"]
            DC["DockerEnvironment:<br/>docker compose up/exec/cp"]
        end

        subgraph AgentLayer["Agent Layer"]
            AF["AgentFactory"]
            BA["BaseInstalledAgent:<br/>install / setup / run"]
            CC["ClaudeCode agent:<br/>install: curl | bash<br/>run: claude -p ..."]
        end

        subgraph VerifierLayer["Verifier Layer"]
            VF["VerifierFactory"]
            VT["exec tests/test.sh"]
            VR["read /logs/verifier/reward.txt"]
        end
    end

    BD -->|generate_task| TA
    TA --> TD
    TD -->|TaskConfig| Job
    CLI2 --> Job
    Job --> Trial

    Trial -->|1. start| EF
    EF --> BE
    BE --> DC

    Trial -->|2. setup agent| AF
    AF --> BA
    BA --> CC
    CC -->|exec in container| DC

    Trial -->|3. run agent| CC
    CC -->|exec in container| DC

    Trial -->|4. verify| VF
    VF --> VT
    VT -->|exec in container| DC
    VT --> VR

    VR -->|TrialResult| Trial
    Trial -->|JobResult| CLI2

    classDef good fill:#ccffcc,stroke:#009900
    class Adapter_Phase good
```

**Advantages**:
- Adapter only generates static task directory, zero runtime coupling
- Framework core manages container, agent, verifier itself
- Agent config fully effective inside container

---

## 3. Side-by-side Comparison

| Layer | HarnessLab (Current) | Harbor |
|------|---------------------|--------|
| **Adapter** | Runtime plugin, calls external runner | Data transformer, generates static task directory |
| **Task Representation** | No unified directory, data passed in memory | Task-as-Files: task.toml + instruction.md + environment/ + tests/ + solution/ |
| **Container Management** | External runner manages (black box) | BaseEnvironment unified interface, framework manages |
| **Agent Install** | External runner installs (ignores HarnessLab config) | BaseInstalledAgent.install(), framework executes inside container |
| **Agent Execution** | External runner executes | BaseAgent.run(), framework drives inside container |
| **Verifier** | External runner executes | Framework executes tests/test.sh, reads reward.txt |
| **Custom Agent** | Limited by external runner's -agent-import-path | Native subclassing + AgentFactory registration |
| **Environment Backend** | Docker only (via external runner) | Docker / Daytona / Modal / E2B / GKE |
| **Result Collection** | Parse external runner output files | Read from bind-mounted log directory |

---

## 4. Key Migration Points for HarnessLab

```mermaid
flowchart LR
    subgraph Current["HarnessLab Current"]
        C1["Adapter runtime plugin"]
        C2["Invoke tb run"]
        C3["AgentProfile config ineffective"]
    end

    subgraph Target["Target Architecture (Harbor Mode)"]
        T1["Adapter task generator"]
        T2["BaseEnvironment container mgmt"]
        T3["AgentDriver runtime interface"]
        T4["Verifier contract"]
    end

    C1 -->|refactor| T1
    C2 -->|refactor| T2
    C3 -->|refactor| T3
    C2 -->|add| T4

    classDef current fill:#ffcccc,stroke:#cc0000
    classDef target fill:#ccffcc,stroke:#009900
    class Current current
    class Target target
```

### Migration Details

1. **Adapter -> Task Generator**
   - `TerminalBenchAdapter.prepare()` no longer calls `tb run`
   - Generate HarnessLab task directory (like Harbor's Task Directory)
   - Convert terminal-bench's `task.yaml` + `docker-compose.yaml` + `run-tests.sh` to HarnessLab format

2. **Add BaseEnvironment Layer**
   - Define Rust trait: `BaseEnvironment { start(), stop(), exec(), upload_file(), download_file() }`
   - Docker backend based on `docker compose`
   - Support compose override stacking (resource limits, mounts, network policy)

3. **Add AgentDriver Layer**
   - Define Rust trait: `AgentDriver { setup(env), run(instruction, env) }`
   - `ClaudeCodeDriver`: install `claude` in container, exec `claude -p ...`
   - `CodexDriver`: install `codex` in container, exec `codex ...`
   - `CustomDriver`: use AgentProfile.command to exec custom command in container
   - AgentProfile `env` injected via `environment.exec()`

4. **Verifier Contract**
   - Define standard verifier interface: exec verification script in container
   - Script writes result to agreed path (e.g. `/harnesslab/verifier/result.json`)
   - Framework core reads result file

5. **Task Directory Structure**
   ```
   .harnesslab/tasks/<task-id>/
   ├── task.toml          # task config
   ├── instruction.md     # task description
   ├── environment/
   │   ├── Dockerfile
   │   └── docker-compose.yaml
   ├── tests/
   │   └── test.sh        # verification script
   └── solution/
       └── solve.sh       # reference solution
   ```
