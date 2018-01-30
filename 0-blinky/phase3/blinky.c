#define GPIO_BASE (0x3F000000 + 0x200000)

volatile unsigned *GPIO_FSEL0 = (volatile unsigned *)(GPIO_BASE);
volatile unsigned *GPIO_SET0  = (volatile unsigned *)(GPIO_BASE + 0x1C);
volatile unsigned *GPIO_CLR0  = (volatile unsigned *)(GPIO_BASE + 0x28);

#define GPIO_INPUT 0
#define GPIO_OUTPUT 1
#define GPIO_FSEL_SIZE 3
#define GPIO_FSEL_MASK 0x7

static void spin_sleep_us(unsigned int us) {
  for (unsigned int i = 0; i < us * 6; i++) {
    asm volatile("nop");
  }
}

static void spin_sleep_ms(unsigned int ms) {
  spin_sleep_us(ms * 1000);
}

static void gpio_fsel(unsigned int gpio_id, unsigned int flags) {
  if (gpio_id > 53) {
    return;
  }

  const unsigned int fsel_register = gpio_id / 10;
  const unsigned int fsel_register_index = gpio_id - fsel_register * 10;
  unsigned reg = GPIO_FSEL0[fsel_register] & ~((GPIO_FSEL_MASK) << (fsel_register_index * GPIO_FSEL_SIZE));
  GPIO_FSEL0[fsel_register] = reg | ((flags & GPIO_FSEL_MASK) << (fsel_register_index * GPIO_FSEL_SIZE));
}

static void gpio_make_output(unsigned int gpio_id) {
  gpio_fsel(gpio_id, GPIO_OUTPUT);
}

static void gpio_set(unsigned int gpio_id) {
  if (gpio_id > 53) {
    return;
  }

  const unsigned int gpio_register = gpio_id / 32;
  const unsigned int gpio_register_index = gpio_id - gpio_register * 32;
  GPIO_SET0[gpio_register] = 1 << gpio_register_index;
}

static void gpio_clear(unsigned int gpio_id) {
  if (gpio_id > 53) {
    return;
  }

  const unsigned int gpio_register = gpio_id / 32;
  const unsigned int gpio_register_index = gpio_id - gpio_register * 32;
  GPIO_CLR0[gpio_register] = 1 << gpio_register_index;
}

int main(void) {
  // Set GPIO Pin 16 as output.
  gpio_make_output(16);

  // Continuously set and clear GPIO 16.
  while (1) {
    gpio_set(16);
    spin_sleep_ms(100);
    gpio_clear(16);
    spin_sleep_ms(100);
  }
}
