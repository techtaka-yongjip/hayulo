CARGO ?= cargo
GH ?= gh
HAYULO = $(CARGO) run --quiet --

.PHONY: test check format-check benchmark examples api-build api-smoke verify release-check ci queue-status queue-active

test:
	$(CARGO) test

check:
	$(HAYULO) check --json
	$(HAYULO) check examples/hello.hayulo --json
	$(HAYULO) check examples/data_core.hayulo --json
	$(HAYULO) check examples/todo_api/main.hayulo --json

format-check:
	$(CARGO) fmt --check
	$(HAYULO) format --check .
	$(HAYULO) format --check tests/fixtures/formatted.hayulo

benchmark:
	$(HAYULO) benchmark llm --json
	$(HAYULO) check benchmarks/llm/baselines --json
	$(HAYULO) format --check benchmarks/llm/baselines --json

examples:
	$(HAYULO) run examples/hello.hayulo
	$(HAYULO) test examples/hello.hayulo
	$(HAYULO) run examples/data_core.hayulo
	$(HAYULO) test examples/data_core.hayulo
	$(HAYULO) test --json

api-build:
	$(HAYULO) build examples/todo_api/main.hayulo

api-smoke: api-build
	cd examples/todo_api/generated && npm test

verify: test check format-check benchmark examples api-smoke

release-check: verify
	$(HAYULO) --version
	git diff --check

ci: release-check

queue-status:
	@$(GH) issue list --state open --label queue --limit 50 --json number,title,labels,milestone --jq 'sort_by(([.labels[].name | select(startswith("priority/"))][0]) // "priority/999")[] | "#\(.number) \(.title) | milestone=\(.milestone.title // "-") | labels=\([.labels[].name] | join(","))"'

queue-active:
	@$(GH) issue list --state open --label active --limit 10 --json number,title,labels,milestone --jq '.[] | "#\(.number) \(.title) | milestone=\(.milestone.title // "-") | labels=\([.labels[].name] | join(","))"'
