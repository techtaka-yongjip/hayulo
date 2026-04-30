PYTHON ?= python3
GH ?= gh
ENV = PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src

.PHONY: test check format-check benchmark examples api-build api-smoke verify release-check ci queue-status queue-active

test:
	$(ENV) $(PYTHON) -m unittest discover -s tests

check:
	$(ENV) $(PYTHON) -m hayulo check --json
	$(ENV) $(PYTHON) -m hayulo check examples/hello.hayulo --json
	$(ENV) $(PYTHON) -m hayulo check examples/data_core.hayulo --json
	$(ENV) $(PYTHON) -m hayulo check examples/todo_api/main.hayulo --json

format-check:
	$(ENV) $(PYTHON) -m hayulo format --check .
	$(ENV) $(PYTHON) -m hayulo format --check tests/fixtures/formatted.hayulo

benchmark:
	$(ENV) $(PYTHON) -m hayulo benchmark llm --json
	$(ENV) $(PYTHON) -m hayulo check benchmarks/llm/baselines --json
	$(ENV) $(PYTHON) -m hayulo format --check benchmarks/llm/baselines --json

examples:
	$(ENV) $(PYTHON) -m hayulo run examples/hello.hayulo
	$(ENV) $(PYTHON) -m hayulo test examples/hello.hayulo
	$(ENV) $(PYTHON) -m hayulo run examples/data_core.hayulo
	$(ENV) $(PYTHON) -m hayulo test examples/data_core.hayulo
	$(ENV) $(PYTHON) -m hayulo test --json

api-build:
	$(ENV) $(PYTHON) -m hayulo build examples/todo_api/main.hayulo

api-smoke: api-build
	cd examples/todo_api/generated && npm test

verify: test check format-check benchmark examples api-smoke

release-check: verify
	$(ENV) $(PYTHON) -m hayulo --version
	git diff --check

ci: release-check

queue-status:
	@$(GH) issue list --state open --label queue --limit 50 --json number,title,labels,milestone --jq 'sort_by(([.labels[].name | select(startswith("priority/"))][0]) // "priority/999")[] | "#\(.number) \(.title) | milestone=\(.milestone.title // "-") | labels=\([.labels[].name] | join(","))"'

queue-active:
	@$(GH) issue list --state open --label active --limit 10 --json number,title,labels,milestone --jq '.[] | "#\(.number) \(.title) | milestone=\(.milestone.title // "-") | labels=\([.labels[].name] | join(","))"'
