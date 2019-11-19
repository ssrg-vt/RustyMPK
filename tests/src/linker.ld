OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")
ENTRY(_start)

SECTIONS {
	. = 2M;

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

	.safe_data 0x400000:
	{
		__safe_data_start = .;
		*(.safe_data)
		*(.safe_data.*)
		. = 0x600000;
	}

	.unsafe_data 0x600000:
	{
		__unsafe_data_start = .;
		*(.unsafe_data)
		*(.unsafe_data.*)
		. = 0x800000;
	}
}
