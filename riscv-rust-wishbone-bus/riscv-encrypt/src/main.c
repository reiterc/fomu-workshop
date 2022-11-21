#include <generated/csr.h>
#include <irq.h>
#include <rgb.h>
#include <time.h>
#include <usb.h>

void isr(void) {
    unsigned int irqs;
    irqs = irq_pending() & irq_getmask();
    if (irqs & (1 << USB_INTERRUPT))
        usb_isr();
}

// Direct write to a specific raw location in memory
void write_raw_address(unsigned int value, volatile unsigned int address) {
	volatile unsigned int *raw_address = (volatile unsigned int *)address;
	*raw_address = value;
}

// Direct read of a raw value of memory
unsigned int read_raw_address(unsigned int address) {
	unsigned int *raw_address;
	raw_address = (unsigned int *)address;
	return *raw_address;
}

int main(void) {
    irq_setmask(0);
    irq_setie(1);
    usb_init();
    rgb_init();
    usb_connect();
	
    // Hardcode the memory region of the FOMU chip
    const unsigned int SRAM_BASE               = 0x10000000;
    const unsigned int SRAM_ALIGNMENT          = 0x4;
    const unsigned int KEY                     = 0x02082022;
	
    unsigned int current_memory_shared = 0x10000004;
    unsigned int xfer_complete;
    int i = 0;
    while (1) {
	// If the value is 0x99999999, then the HOST->FOMU xfer is complete
	xfer_complete = read_raw_address(SRAM_BASE);
	if(xfer_complete == 0x99999999){	
		// read each chunk of memory until the end of the transfer is reached;
		// after each read, xor with KEY and write to same address;
		// if the end of transfer is reached, write to the status memory location;
		// "working data" memory range is 0x10000008-undetermined;
                unsigned int xfer_word_buffer = read_raw_address(current_memory_shared);
		for (i=0;i<8000;i++) {
			xfer_word_buffer = read_raw_address(current_memory_shared);
			xfer_word_buffer = xfer_word_buffer ^ KEY;
			write_raw_address(xfer_word_buffer, current_memory_shared);
			current_memory_shared += SRAM_ALIGNMENT;
		}
			
		// Tell the HOST the encryption is complete
		write_raw_address(0x88888888, SRAM_BASE);
			
		// Reset the working addresses
		current_memory_shared = SRAM_BASE + SRAM_ALIGNMENT;
			
	}
     msleep(5);
    }
}
