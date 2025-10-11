
MODSRC := $(PWD)/src
KDIR := /lib/modules/$(shell uname -r)/build
OUT := $(PWD)/build

all:
	mkdir -p $(OUT)
	cp $(MODSRC)/* $(OUT)
	$(MAKE) -C $(KDIR) M=$(OUT) modules

clean:
	$(MAKE) -C $(KDIR) M=$(OUT) clean
	rm -r $(OUT)
	
