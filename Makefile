lib.name = helloworldpd

cflags = -Itarget/release -Ltarget/release -lopm
ldflags = -Itarget/release -Ltarget/release -lopm
ldlibs = -lopm

class.sources = src/helloworld.c

datafiles =

PDLIBBUILDER_DIR=pd-lib-builder/
include $(PDLIBBUILDER_DIR)/Makefile.pdlibbuilder
