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