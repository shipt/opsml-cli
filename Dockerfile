# Any extra packages to install can go here
ARG EXTRA_SYS_PACKAGES="\
    librdkafka-dev \
"
ARG POETRY_VERSION=1.5.0
ARG PROJECT_NAME=poetry-template
ARG PY_ROOT_MODULE=poetry_template
ARG PY_VER_FULL=3.10.11
ARG PY_VER_MAJOR=3.10
# required on all stages..e.g. CAs and timezones should never be stale
ARG SYS_PACKAGES="ca-certificates tzdata"

FROM harbor.shipttech.com/apps/python-images/shipt-python:${PY_VER_FULL}_poetry_${POETRY_VERSION} as base
ARG EXTRA_SYS_PACKAGES
ARG PROJECT_NAME
ARG PY_VER_MAJOR
ARG SYS_PACKAGES

ENV PROJECT_HOME=/opt/${PROJECT_NAME}
ENV PROJECT_PYTHONPATH=${PROJECT_HOME}/.venv/lib/python${PY_VER_MAJOR}/site-packages
ENV PROJECT_VENV_BIN=${PROJECT_HOME}/.venv/bin
# prefix the venv bin to the path
ENV PATH=${PROJECT_VENV_BIN}${PATH:+":$PATH"}

RUN apt-get update --no-install-recommends \
    && apt-get install --no-install-recommends --yes \
      ${SYS_PACKAGES} \
      ${EXTRA_SYS_PACKAGES} \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get autoremove \
    && apt-get clean \
    && mkdir --parents ${PROJECT_VENV_BIN}  # also creates PROJECT_HOME

WORKDIR ${PROJECT_HOME}


# ================================================================
# Builder image
# adds python project files
# ================================================================
FROM base as builder
ARG PY_ROOT_MODULE
ARG ARTIFACTORY_PYPI_USERNAME
ARG ARTIFACTORY_PYPI_PASSWORD

ENV POETRY_HTTP_BASIC_SHIPT_RESOLVE_USERNAME=$ARTIFACTORY_PYPI_USERNAME
ENV POETRY_HTTP_BASIC_SHIPT_RESOLVE_PASSWORD=$ARTIFACTORY_PYPI_PASSWORD

# separate caching layer for deps
COPY pyproject.toml poetry.lock ./
RUN poetry install --only main --no-root


# caching layer for root
COPY ${PY_ROOT_MODULE} ${PY_ROOT_MODULE}/
RUN poetry install --only main

# # ================================================================
# # Dev Image
# # adds testing and linting related files
# # ================================================================
FROM builder as dev

RUN poetry install

COPY tests tests/
COPY .tool-versions Makefile setup.cfg ./

# # ================================================================
# # Prod Image
# # only module and venv related files
# # ================================================================
FROM python:${PY_VER_FULL}-slim as prod
ARG EXTRA_SYS_PACKAGES
ARG PROJECT_NAME
ARG PY_ROOT_MODULE
ARG PY_VER_MAJOR
ARG SYS_PACKAGES

###### START same as base - TODO way to DRY this out?
ENV PROJECT_HOME=/opt/${PROJECT_NAME}
ENV PROJECT_PYTHONPATH=${PROJECT_HOME}/.venv/lib/python${PY_VER_MAJOR}/site-packages
ENV PROJECT_VENV_BIN=${PROJECT_HOME}/.venv/bin
# prefix the venv bin to the path
ENV PATH=${PROJECT_VENV_BIN}${PATH:+":$PATH"}

RUN apt-get update --no-install-recommends \
    && apt-get install --no-install-recommends --yes \
      ${SYS_PACKAGES} \
      ${EXTRA_SYS_PACKAGES} \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get autoremove \
    && apt-get clean \
    && mkdir --parents ${PROJECT_VENV_BIN}  # also creates PROJECT_HOME

WORKDIR ${PROJECT_HOME}

###### END same as base

COPY --from=builder ${PROJECT_PYTHONPATH} ${PROJECT_PYTHONPATH}
COPY --from=builder ${PROJECT_VENV_BIN} ${PROJECT_VENV_BIN}/
COPY --from=builder "${PROJECT_HOME}/${PY_ROOT_MODULE}" ${PY_ROOT_MODULE}/

# allow invoking without poetry, e.g. can use simply `python -m` instead of `poetry run python -m`
# this saves about a second of start up time in addition to being simpler to inspect later
# e.g. /opt/my-project-name/.venv/lib/python3.10/site-packages
ENV PYTHONPATH=${PROJECT_PYTHONPATH}${PYTHONPATH:+":$PYTHONPATH"}
