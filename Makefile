MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR 	:= $(dir $(MKFILE_PATH))

DEBUG_BIN_DIR 	:= ./target/debug

MICRO_SERVICES	:= 	user-service \
					recommendation-service \
					research-service profile-service rate-service \
					geo-service

MONO_SERVICE	:= monolithic-service

TRACE_FILENAME	:=	dbg.out

debug: $(DEBUG_BIN_DIR) # start all micro-services
	@$(foreach service, $(MICRO_SERVICES), \
		$(MKFILE_DIR)/$(DEBUG_BIN_DIR)/$(service)& 2>> $(TRACE_FILENAME))

stop: 
	killall $(MICRO_SERVICES) &