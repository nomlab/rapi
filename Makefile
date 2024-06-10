SUBDIRS := rapi rapi-daemon
TARGETS := all install install-local clean

SUBDIRS := $(foreach t, $(SUBDIRS), $(addsuffix /.,$t))
SUBDIRS_TARGETS := $(foreach t,$(TARGETS),$(addsuffix $t,$(SUBDIRS)))

.PHONY : $(TARGETS) $(SUBDIRS_TARGETS)

$(TARGETS) : % : $(addsuffix %,$(SUBDIRS))

$(SUBDIRS_TARGETS) :
	$(MAKE) -C $(@D) $(@F:.%=%)

