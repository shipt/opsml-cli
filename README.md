This template has 3 make helpers: make format, make lints, make tests.
**When creating a new service, please read the [How to Create a New Service](https://techhub.shipt.com/products-and-services/how-to-create-a-new-service/) Documentation first.**


This document is a model for how to order the sections of your ReadMe.  This format is _highly_ encouraged for structuring your README to keep documentation as consistent as possible. A good living example of this document is: https://github.com/shipt/shipt-search. 

# Name of Your Product
Describe your product in 1-2 sentences (there is room for more details later).

## Table of Contents

* [Maintainers](#Maintainers)
* [Slack Channel](#slack-channel)
* [Overview](#Overview)
  * [Architecture](#architecture)
  * [Dependencies](#dependencies)
  * [Terminology](#terminology)
* [Deployment and Configuration](#deployment-and-configuration)
  * [Environmental Variables](#environmental-variables)
  * [Metrics and Telemetry](#metrics-and-telemetry)
  * [Logging](#logging)
  * [Analytics](#analytics)
  * [Versioning](#versioning)
  * [CI/CD](#cicd)
* [Developer Details](#developer-details)
  * [Installation](#installation)
  * [File Linting](#file-linting)
  * [Git, Pull Requests, and Reviews Process](#git-pull-requests-and-reviews-process)
  * [Testing](#testing)
  * [Resources](#resources)
  * [Environments](#environments)
  * [Unit Testing](#unit-testing)

## Maintainers
A list of names of people who actively maintain, review PRs, and can support this code.

## Slack Channel

The channel you'd prefer people contact your team at in case of issues. 

## Overview
A paragraph or two describing your thing in appropriate detail for a fairly wide (non-engineering) audience. Include 
code names here.  

## PyShipt
There are still some [some kinks](https://github.com/shipt/pyshipt#install-and-use-pyshipt-in-another-project) to work 
out with adding pyshipt as a dependency via Poetry, so this template comes with the dependency commented out in `pyproject.toml`.

### Architecture
As this applies, include information about how the system is structured. Include a diagram or a link to one if at all 
possible. A picture is worth a thousand words. I encourage you to use [Lucidchart](https://www.lucidchart.com) as we 
have an enterprise account for this. 

Ex:

![Microservice Architecture October 2019 (2)](https://user-images.githubusercontent.com/42652171/66686504-4bcaef00-ec45-11e9-9292-8427a3896a31.jpeg)

### Dependencies
List any service dependencies that this thing has - Shipt or non-Shipt. Optimally these are also covered in the diagram above. They should also be listed [here](https://github.com/shipt/TechHub/blob/master/content/services/a-guide-to-service-repos.md) under your service.

### Terminology
Ideally you have links throughout this document to terminology already described in the [Shipt Dictionary](https://github.com/shipt/TechHub/blob/master/content/culture/tech-culture/shipt-dictionary.md). If an explanation of terms is required beyond that, do so here.

## Deployment and Configuration

### Environmental Variables
As applies

### Metrics and Telemetry
As applies

### Logging
As applies

### Analytics
As applies
Ex: [Segway Analytics](https://github.com/shipt/segway##analytics)

### Versioning
Describe the versioning strategy, how to create a version or a release, and how different versions are maintained (if supporting multiple versions).

### CI/CD
- Where are the tests executed (drone, jenkins, codeship, circle ci, other hosted solution, other in house solution)? 
- When are they executed? 

## Developer Details

### Installation 
Give directions to install necessary applications and tools.

#### Basic assumptions about your shell
Capture all the steps for setup of your shell from scratch

TL;DR curl install homebrew, `brew install zsh asdf direnv git openssl readline sqlite3 xz zlib`,
curl install oh-my-zsh, use the .profile file below

Broken out by step
1. Homebrew is installed, I curl installed it.
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

1. ZSH is up-to-date: `brew install zsh`, brew will receive bug fixes for zsh faster than macOS does

1. asdf is up-to-date: `brew install asdf`, this tool is for managing tool versions e.g. python, poetry, and many others

1. direnv is installed: `brew install direnv`, this tool manages env vars and scripts that activate on directory switch

1. Git is installed: `brew install git`

1. Python C libs: `brew install openssl readline sqlite3 xz zlib`, this list is based on pyenv recommendations

1. Using oh-my-zsh (.zshrc), I curl installed it.
   ```bash
   sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
   ```

1. Grab the [example `.profile`](#example-profile) below and save it to your `$HOME` directory. Go over it and make sure things agree with any customizations you might have on your system.

1. Where you git clone things to is set with `REPOS_ROOT` env var. I use $HOME/source, use whatever makes sense for you.
   Having this variable set makes it easy to share scripts.

1. ZSH
   1. When you update zsh there is the possibility of ~/.zshrc being overwritten and your shell config no longer having effect. Completing these steps to reinstall will solve it.
   1. Make sure to add `source ~/.profile` to your `~/.zshrc` file so new shells start with it loaded.

1. Restart your terminal or source your customized profile file for your current shell: `source ~/.profile`

#### Example .profile:
```bash
#
#   Basic recipe when starting fresh
#       curl install homebrew
#       `brew install zsh asdf direnv git`
#       curl install oh-my-zsh
#       grab this example file and save to $HOME/.profile
#       echo "source ~/.profile" >> ~/.zshrc
#
## where I git clone my things
export REPOS_ROOT=$HOME/source

#
#   asdf
#    `brew install asdf`
#
source "$(brew --prefix asdf)/asdf.sh"
## if you run into issues with asdf 0.9.0+ you may need to use an alternate config
## to do so, uncomment the 2 lines below and comment out the brew --prefix asdf line above
## then resource this profile file
# export ASDF_DIR=/usr/local/opt/asdf/libexec
# source "${ASDF_DIR}/asdf.sh"

#
#   direnv
#     `brew install direnv`
#
eval "$(direnv hook zsh)"

#
#   postgres
#    `brew install libpq`
# 
## add psql to path
export PATH="$(brew --prefix libpq)/bin:$PATH"

#
#   python
#
export ARTIFACTORY_PYPI_USERNAME=jklehm
export ARTIFACTORY_PYPI_PASSWORD=REDACTED
export POETRY_HTTP_BASIC_SHIPT_RESOLVE_USERNAME=${ARTIFACTORY_PYPI_USERNAME}
export POETRY_HTTP_BASIC_SHIPT_RESOLVE_PASSWORD=${ARTIFACTORY_PYPI_PASSWORD}
## python c toolchain gunk
##   see pyenv https://github.com/pyenv/pyenv/wiki#suggested-build-environment
##    `brew install openssl readline sqlite3 xz zlib`
##   for psycopg2 psql
##    `brew install libpq`
##
### supplement ldflags and cppflags with brew libs
###   add other C libraries that python packages need here in the future
for l in libpq openssl readline sqlite3 xz zlib; do
   _lib_prefix=$(brew --prefix $l); \
   export LDFLAGS="${LDFLAGS} -L${_lib_prefix}/lib"; \
   export CPPFLAGS="${CPPFLAGS} -I${_lib_prefix}/include"; \
   unset _lib_prefix; \
done;
```

### File Linting
List what tools you're using (insert another section for Style Guide if needed), we aim to provide two verbs that are
easy to use `make format` and `make lints`

This template provides a way to use isort, black, flake8, pylint, and mypy in conjunction. After running `make setup` 
once to create your venv you can then use `make lints` to run these 3 tools against your code.  To run each individually
you can use:
* `make lints.ruff` - configuration is the default for now
* `make lints.flake8` - configuration is pulled from setup.cfg (flake8/darglint don't support pyproject.toml)
* `make lints.pylint` - configuration is pulled from pyproject.toml
* `make lints.mypy` - configuration is pulled from pyproject.toml
* `make lints` - runs all 4 of the above
* `make format` - uses isort and black to format code - config pulled from pyproject.toml
* `make format.check` - returns success if your code has no formatting changes needed


### Git, Pull Requests and Reviews Process
Document the process that the development team should follow for this repo.  

- You're welcome to link to [this doc](https://github.com/shipt/TechHub/blob/main/content/engineering/shopper/processes/pull-requests.md)

### Testing
List what is being used for testing and any additional instructions on how to test for this application or service. 

### Resources
This is the place to include any links that are used anywhere else in this document to other sources or documentation so that it's clear and easy to find them. 

### Environments
List those here. 

### Unit Testing
Code coverage should never decrease.  The metrics for this are [here](https://www.hostedgraphite.com/9c9857b9/grafana/dashboard/db/test-coverage).

