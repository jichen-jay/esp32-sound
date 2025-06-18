/* ESP32-H2 Memory Layout */
MEMORY
{
  /* SRAM */
  RAM : ORIGIN = 0x40800000, LENGTH = 320K
  
  /* External Flash */
  FLASH : ORIGIN = 0x42000000, LENGTH = 4M
}

/* Stack size */
_stack_size = 8K;

/* Heap size */
_heap_size = 0;

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

/* RTC memory sections */
_rtc_fast_bss_start = 0;
_rtc_fast_bss_end = 0;
_rtc_fast_persistent_start = 0;
_rtc_fast_persistent_end = 0;

/* Required symbols */
_bss_start = ORIGIN(RAM);
_bss_end = ORIGIN(RAM) + LENGTH(RAM);
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_max_hart_id = 0;