PYTHON ?= python3
ENV = PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src

.PHONY: test check examples api-build api-smoke

test:
	$(ENV) $(PYTHON) -m unittest discover -s tests

check:
	$(ENV) $(PYTHON) -m hayulo check examples/hello.hayulo --json
	$(ENV) $(PYTHON) -m hayulo check examples/todo_api/main.hayulo --json

examples:
	$(ENV) $(PYTHON) -m hayulo run examples/hello.hayulo
	$(ENV) $(PYTHON) -m hayulo test examples/hello.hayulo

api-build:
	$(ENV) $(PYTHON) -m hayulo build examples/todo_api/main.hayulo

api-smoke: api-build
	cd examples/todo_api/generated && npm test
