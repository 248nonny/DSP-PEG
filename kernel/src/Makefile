obj-m += dsp_peg_kdrv.o

dsp_peg_kdrv-objs := main.o comms.o

KDIR := /lib/modules/$(shell uname -r)/build

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules
	
clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
