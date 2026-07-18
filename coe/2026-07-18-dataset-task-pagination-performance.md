# Problem P-001

- Symptom: Dataset detail takes a long time to show Tasks and renders the returned Task set without pagination.
- Expected: The first page displays quickly, with 20 Tasks per page; search and pagination must not load or parse the full Dataset.
- Actual: `GET /datasets/{ref}/tasks?limit=20` parses every local Task before the API slices the response.
- Environment: local v1.0.5 development services, `swebenchpro@1.0`.
- Fix criteria: filtering and pagination happen before Task summaries are parsed; the UI renders one server page; Task environment details remain lazy-loaded.

# Hypothesis H-001

- Claim: API pagination is applied after `WebUiDatasetService.list_tasks` has parsed the complete local Dataset.
- Prediction: a request for 20 items has latency proportional to the total Task count and returns only 20 items after parsing all Task configs.
- Diagnostic evidence plan: trace the API/service call order and time a real request against a large downloaded Dataset.
- Status: supported.

# Evidence E-001

- Code path: `ornnlab/api/webui.py` calls `list_tasks(dataset_ref, q)` first and only then passes the resulting list to `_page`.
- Service path: `WebUiDatasetService.list_tasks` calls `parse_local_tasks`, which iterates every Task directory and parses each `task.toml`.
- Supports: H-001.

# Evidence E-002

- Probe: `GET /api/webui/v1/datasets/swebenchpro@1.0/tasks?limit=20`.
- Result before repair: HTTP 200, 20 response items, about 12.64 seconds wall time.
- Interpretation: the requested page size does not bound backend parsing work.
- Supports: H-001.

# Hypothesis H-002

- Claim: the frontend compounds the backend cost by requesting 100 Tasks and maintaining a separate search request path, then renders the full returned set.
- Prediction: opening the drawer requests `limit=100`; entering search switches between independent resource states and no Task pagination control is rendered.
- Diagnostic evidence plan: inspect `DatasetsPage` request parameters and `DatasetDetail` rendering.
- Status: supported.

# Evidence E-003

- `DatasetsPage` requests `{ limit: 100 }`, creates a second `useCachedServerSearch` request with the same limit, and passes all visible results to `DatasetDetail`.
- `DatasetDetail` maps the entire `tasks` array and has no Pagination component.
- Supports: H-002.

# Hypothesis H-003

- Claim: after Task pagination is corrected, a cold request can still be delayed because the local Dataset DTO recursively calculates directory size before the Task catalog is read.
- Prediction: an isolated Task request using a lightweight local-path read is fast, while a daemon restart with the Dataset catalog concurrently loading can block on filesystem size traversal.
- Diagnostic evidence plan: inspect `stored_dataset_dto`, compare isolated and daemon-concurrent Task requests, and prohibit `Path.rglob` in a focused runtime-location test.
- Status: supported.

# Evidence E-004

- `stored_dataset_dto` computes `sizeBytes` with `path.rglob("*")` for every local Dataset read.
- The Task endpoint previously called `_local_dataset`, even though it only needs download status and local path.
- Supports: H-003.

# Evidence E-005

- After pagination but before lightweight path reads, warm page/search requests were 0.11-0.30 seconds, while the first daemon request concurrent with Dataset catalog loading remained 13-18 seconds.
- An isolated fresh application request returned 20 of 731 Tasks in about 0.18 seconds.
- Supports: H-003 and distinguishes Task indexing from unrelated Dataset size scanning.

# Evidence E-006

- Fix validation: `stored_dataset_runtime` returns only source, path, and availability; its regression test fails if `Path.rglob` is called.
- Task list items contain only name/identity data. Complete description and environment parsing moved to the single-Task detail request.
- Supports: H-001, H-002, and H-003 repair validation.

# Evidence E-007

- Codex Web Preview validation on `swebenchpro@1.0`: first page rendered 20 rows with `1-20 / 共 731`; next page rendered 20 rows with `21-40 / 共 731`.
- Searching `ansible` from page 2 reset the drawer to page 1 and rendered `1-20 / 共 96`.
- Expanding one Task loaded its environment and `linux/amd64` image platform on demand; browser console errors were empty.
- Supports: P-001 fix criteria.
