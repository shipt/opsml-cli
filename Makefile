PROJECT=poetry-template
SOURCE_OBJECTS=poetry_template stubs tests

format.black:
	poetry run black ${SOURCE_OBJECTS}
format.isort:
	poetry run isort --atomic ${SOURCE_OBJECTS}
format: format.isort format.black

lints.format_check:
	poetry run black --check ${SOURCE_OBJECTS}
	poetry run isort --check-only ${SOURCE_OBJECTS}
lints.flake8:
	poetry run flake8 ${SOURCE_OBJECTS}
lints.flake8.ci:
	poetry run flake8 --output-file=flake8-output.txt ${SOURCE_OBJECTS}
lints.mypy:
	poetry run mypy ${SOURCE_OBJECTS}
lints.pylint:
	poetry run pylint --rcfile pyproject.toml  ${SOURCE_OBJECTS}
lints.ruff:
	poetry run ruff check ${SOURCE_OBJECTS}
lints: lints.ruff lints.flake8 lints.pylint lints.mypy
lints.ci: lints lints.format_check

setup: setup.sysdeps setup.python setup.project
# setup.uninstall - handle in and out of project venvs
setup.uninstall:
	@export _venv_path=$$(poetry env info --path); \
    if [ ! -n "$${_venv_path:+1}" ]; then \
      echo "\nsetup.uninstall: didn't find a virtualenv to clean up"; \
      exit 0; \
    fi; \
    echo "\nattempting cleanup of $$_venv_path" \
    && export _venv_name=$$(basename $$_venv_path) \
    && ((poetry env remove $$_venv_name > /dev/null 2>&1 \
         || rm -rf ./.venv) && echo "all cleaned up!") \
    || (echo "\nsetup.uninstall: failed to remove the virtualenv." && exit 1)
setup.project:
	poetry install
setup.python:
	@echo "Active Python version: $$(python --version)"
	@echo "Base Interpreter path: $$(python -c 'import sys; print(sys.executable)')"
	@export _python_version=$$(cat .tool-versions | grep -i python | cut -d' ' -f2) \
      && test "$$(python --version | cut -d' ' -f2)" = "$$_python_version" \
      || (echo "Please activate python version: $$_python_version" && exit 1)
	@poetry env use $$(python -c "import sys; print(sys.executable)")
	@echo "Active interpreter path: $$(poetry env info --path)/bin/python"
setup.sysdeps:
	# bootstrap python first to avoid issues with plugin installs that count on python
	@-asdf plugin-add python; asdf install python
	@asdf plugin update --all \
      && for p in $$(cut -d" " -f1 .tool-versions | sort | tr '\n' ' '); do \
           asdf plugin add $$p || true; \
         done \
      && asdf install \
      || (echo "WARNING: Failed to install sysdeps, hopefully things aligned with the .tool-versions file.." \
         && echo "   feel free to ignore when on drone")

test.clean:
	docker-compose down
	-docker rmi $$(docker images -a | grep ${PROJECT} | tr -s ' ' | cut -d' ' -f3)
	-docker image prune -f

test.unit:
	poetry run coverage run -m pytest \
		--ignore tests/integration \
		--cov=./poetry_template \
		--cov-report=xml:coverage-report-unit-tests.xml \
		--junitxml=coverage-junit-unit-tests.xml \
		--cov-report term
test.integration:
	poetry run pytest tests/integration

util.dev_deps_latest:
	# double check your extras after this command, it's not quite smart enough..
	_plist=$$(dasel select -f pyproject.toml -m 'tool.poetry.dev-dependencies.-' | sort | xargs printf "%s@latest ")\
        && poetry add --dev $$_plist
