
#include <asm/barrier.h>
#include <asm/cacheflush.h> // __flush_dcache_area
#include <linux/delay.h>
#include <linux/firmware.h> /* for firmware struct to load dsp binary. */
#include <linux/init.h>     /* Needed for the macros */
#include <linux/io.h>
#include <linux/io.h>         /* Needed for __iomem, ioremap, etc. */
#include <linux/ioport.h>     /* struct resource itself */
#include <linux/kernel.h>     /* Needed for KERN_INFO */
#include <linux/module.h>     /* Needed by all modules */
#include <linux/of.h>         /* Needed for device_node */
#include <linux/of_address.h> /* for of_address_to_resource() */
#include <linux/printk.h>
#include <linux/slab.h>

#include "comms.h"

#define DRIVER_AUTHOR "Ronny Cravioto-Ross <ronny.cravioto@gmail.com>"
#define DRIVER_DESC "The DSP PEG Guitar Pedal Kernel Driver."

// This function looks for a node names "name" in the device tree, claims
// its resource, and attempts to ioremap() it. It is intended to be used to
// claim the shared and reserved memory regions which will be used by the
// bare-metal program.
static void __iomem *get_dsp_node_memory_from_name(const char *name,
                                                   resource_size_t *size) {
  struct device_node *np;
  struct resource res;

  np = of_find_node_by_name(NULL, name);

  if (!np) {
    printk(KERN_ERR "Could not locate device tree node %s.\n", name);
    return NULL;
  }

  if (of_address_to_resource(np, 0, &res)) {
    printk(KERN_ERR "Could not translate device node '%s' to resource...\n",
           name);
    of_node_put(np);
    return NULL;
  }
  of_node_put(np);

  printk(KERN_INFO "Acquired resource %pR from name '%s'.\n", &res, name);

  // Map memory! :D
  void __iomem *out = ioremap(res.start, resource_size(&res));

  printk(KERN_INFO "Finished executing ioremap for '%s'.\n", name);

  if (!out) {
    printk(KERN_ERR "ioremap for '%s' failed...\n", name);
    return NULL;
  }

  // Pass along the size of the allocated region.
  if (size)
    *size = resource_size(&res);

  // Return successfully mapped pointer.
  return out;
}

// This function is for validating the bare metal binary payload; it
// was introduced as a sanity check when the bare metal was not running to
// confirm that the problem was not in loading the binary to memory.
//
// The function takes in an __iomem pointer and a length, and dumps a hex
// of what is there.
static void hexdump_iomem(void __iomem *base, size_t len) {
  size_t to_dump = min(len, 1024);
  void *tmp = kmalloc(to_dump, GFP_KERNEL);
  if (!tmp) {
    pr_err("hexdump_iomem: OOM\n");
    return;
  }
  memcpy_fromio(tmp, base, to_dump);
  print_hex_dump(KERN_INFO, "dsp_reserved: ", DUMP_PREFIX_OFFSET, 16, 1, tmp,
                 to_dump, true);
  kfree(tmp);
}

// This function wakes up an additional core and gets it to attempt to
// execute bare metal code at "entry_pa."
static void kick_core_spintable(phys_addr_t release_pa, phys_addr_t entry_pa) {

  // Get the virtual address corresponding to the physical spin table address.
  volatile u32 *slot = (volatile u32 *)phys_to_virt(release_pa);

  // write the entry address to the spin table address.
  WRITE_ONCE(*slot, entry_pa);

  // Clean cache lines to make sure everything is written properly.
  flush_dcache_page(pfn_to_page(release_pa >> PAGE_SHIFT));

  // Full system barrier before the wakeup instruction
  dsb(sy);
  isb();

  // Read back the value to confirm it is written (this wouldn't necessarily
  // catch cache issues, but I'm doing it anyways as an extra sanity check)
  pr_info("spin-table[%pa] = 0x%016lx\n", &release_pa,
          (unsigned long)READ_ONCE(*slot));

  // Wake up the secondary cpu with the 'sev' instruction! (this is how you wake
  // the extra cores on RPi)
  asm volatile("sev" ::: "memory");
}

// Kernel module entry point
static int __init init_dsp_peg(void) {

  // Make it clear in dmesg when the module is loaded lmao (print a bunch of
  // empty lines) (should be removed later).
  for (int i = 0; i < 10; i++) {
    printk(KERN_INFO "\n");
  }

  // In general verbose logging is good here since it will be easier to
  // troubleshoot.
  printk(KERN_INFO "Loading DSP PEG driver... pi ≈ %i\n", pi());
  printk(KERN_INFO "Reserving DSP Comms memory...\n");

  // Initialize shared and reserved memory
  void __iomem *base_shared = NULL;
  void __iomem *base_bare_metal = NULL;
  resource_size_t size_shared = 0;
  resource_size_t size_bare_metal = 0;

  base_shared = get_dsp_node_memory_from_name("dsp_shared", &size_shared);
  base_bare_metal =
      get_dsp_node_memory_from_name("dsp_reserved", &size_bare_metal);

  // Make sure we actually got pointers from the ioremap functions.
  if (base_shared == NULL || base_bare_metal == NULL)
    return -EINVAL;

  printk(KERN_INFO "Successfully mapped memory!\n");

  // Write a canary value to the base shared address so we can tell
  // if the bare metal is executing and mutating this value.
  printk(KERN_INFO "Writing and reading to shared memory as test...\n");
  writeq(0xDEADBEEFC0DEC0DEULL, base_shared + 0x0);
  // Also read the value back as a sanity check.
  u64 v = readq(base_shared + 0x0);
  pr_info("shared roundtrip: %016llx\n", (unsigned long long)v);

  printk(KERN_INFO "Loading in bare-metal DSP firmware...\n");

  // Request the bare-metal binary payload as a 'firmware' struct,
  // and write its contents to the beginning of the dsp_shared memory chunk.
  const struct firmware *fw;
  const char *fw_name = "dsp_peg_fw.bin";

  int ret = request_firmware(&fw, fw_name, NULL);

  if (ret) {
    printk(KERN_ERR "Unsucessful firmware request for '%s'", fw_name);
    return ret;
  }

  if (fw->size > size_bare_metal) {
    printk(KERN_ERR "Firmware does not fit in the reserved memory chunk!\n");
    return -EFBIG;
  }

  // Write bare metal binary payload to memory for execution.
  memcpy_toio(base_bare_metal, fw->data, fw->size);

  // Memory barriers to make sure the payload is properly loaded.
  dsb(sy);
  isb();

  wmb();

  // Don't need firmware anymore, we can release it.
  release_firmware(fw);

  // Print a hex dump of the binary payload. This can be compared to hex of the
  // binary file as a sanity check.
  hexdump_iomem(base_bare_metal, 256);

  printk(KERN_INFO "Starting bare metal execution!\n");

  // Tell the core to start executing the bare metal payload!
  kick_core_spintable(0xE0, 0x10100000);

  // Check the base address of the shared region for changes;
  // the bare metal program is currently set up to set this to 0 and then
  // increment it every 200 ms.
  for (int i = 0; i < 20; i++) {
    u64 a = readq(base_shared + 0x0);
    msleep(200);
    u64 b = readq(base_shared + 0x0);
    // Print out "hearbeat", which should be incrementing
    // by 1 every time.
    pr_info("BM heartbeat64: %016llx -> %016llx\n", (unsigned long long)a,
            (unsigned long long)b);
  }

  printk(KERN_INFO "Done loading DSP PEG kernel driver.\n");
  return 0;
}

// Kernel module exit point.
static void __exit cleanup_dsp_peg(void) {
  printk(KERN_INFO "Unloaded DSP PEG driver.\n");
}

module_init(init_dsp_peg);
module_exit(cleanup_dsp_peg);

/*
 * Get rid of taint message by declaring code as GPL.
 */
MODULE_LICENSE("GPL");

MODULE_AUTHOR(DRIVER_AUTHOR);    /* Who wrote this module? */
MODULE_DESCRIPTION(DRIVER_DESC); /* What does this module do */
