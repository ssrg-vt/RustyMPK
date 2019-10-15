OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")
ENTRY(_start)

SECTIONS {
	/* Load the kernel reasonably high in memory to avoid special addresses. */
	. = 2M;

	kernel_start = .;

	.boot :
	{
		/* This goes first. */
		KEEP(*(.kheader))
	}

	.text : AT(ADDR(.text))
	{
		*(.text)
		*(.text.*)
	}

	.rodata : AT(ADDR(.rodata))
	{
		*(.rodata)
		*(.rodata.*)
	}

	.data ALIGN(4096) : AT(ADDR(.data))
	{
		*(.data)
		*(.data.*)
	}

	.bss ALIGN(4096) : AT(ADDR(.bss))
	{
		__bss_start = .;
		*(.bss)
		*(.bss.*)
		__bss_end = .;
	}
	.isolated_data 0x400000:
	{
		__isolated_data_start = .;
		*(.isolated_data)
		*(.isolated_data.*)
		. = 0x600000;
		__isolated_data_end = .;
		__isolated_data_size = __isolated_data_end - __isolated_data_start;
	}
	kernel_end = .;
}
