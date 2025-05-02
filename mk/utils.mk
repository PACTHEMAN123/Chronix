########################################################
# DOCKER (unused)
########################################################

# docker
DOCKER_TAG ?= rcore-tutorial-v3:latest
.PHONY: docker build_docker
	
docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt --name rcore-tutorial-v3 ${DOCKER_TAG} bash

build_docker: 
	docker build -t ${DOCKER_TAG} --target build .

.PHONY: docker build_docker

########################################################
# UTILS (for prettier building process)
########################################################

RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
PURPLE = \033[0;95m
RESET = \033[0m
BOLD = \033[1m

define building
	@echo "${BOLD}${PURPLE} [BUILDING] ${1}${RESET}"
endef

define success
	@echo "${BOLD}${GREEN} [SUCCESS ] ${1}${RESET}"
endef