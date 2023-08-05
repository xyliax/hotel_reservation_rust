MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR 	:= $(dir $(MKFILE_PATH))

DEBUG_BIN_DIR 	:= ./target/debug
RELEASE_BIN_DIR	:= ./target/release

MICRO_SERVICES	:= 	user-service \
					recommendation-service \
					research-service profile-service rate-service \
					geo-service

MONO_SERVICE	:= monolithic-service

PEEK_INFO_TEST	:= test_peek_info

MICRO_DEBUG_FILENAME	:=	micro-debug.out
MICRO_RELEASE_FILENAME	:=	micro-release.out
MONO_DEBUG_FILENAME		:=	mono-debug.out
MONO_RELEASE_FILENAME	:=	mono-release.out
USER_DEBUG_FILENAME		:=	user-debug.out


micro-debug: $(DEBUG_BIN_DIR) # start all micro-services
	@rm -f $(MICRO_DEBUG_FILENAME)
	@$(foreach service, $(MICRO_SERVICES), \
		$(MKFILE_DIR)/$(DEBUG_BIN_DIR)/$(service)& 2>> $(MICRO_DEBUG_FILENAME))
	@tail -f $(MICRO_DEBUG_FILENAME)

test-peekinfo: $(DEBUG_BIN_DIR)
	@rm -f $(MONO_DEBUG_FILENAME) $(USER_DEBUG_FILENAME)
	@$(MKFILE_DIR)/$(DEBUG_BIN_DIR)/$(PEEK_INFO_TEST) $(item) $(req)& \
		1>> $(USER_DEBUG_FILENAME) 2>> $(MONO_DEBUG_FILENAME)
	@tail -f $(MONO_DEBUG_FILENAME)

stop-all: 
	killall -q $(MICRO_SERVICES) $(MONO_SERVICE) &