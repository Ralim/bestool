Baud: 921600; no flow control 8N1

# UART Commands Tx/Rx

~> Tx/Rx from point of view of pc

Suspected format:

- 0xBE
- CMD
- ...
- Checksum Total sum of all bytes included == 0xFF; so 0xFF-total with wrap

## Startup

### SYNC {BE,50,00,03,00,00,01,ED} [Rx]

Version: 0x0100

### SYNC Resp {BE,50,00,01,01,EF} [Tx]

### SYNC Confirm {BE,50,00,03,02,00,01,EB} [Rx]

Version 0x0100
No special data operation

## Loading programmer blob

### CODE? Start Programmer {BE,53,00,0C,DC,05,01,20,DC,32,01,00,C0,A7,E8,0C,76} [Tx]

Address 0x200105dc
Length 78556 (0x0132DC)

### CODE? Start Programmer reply {BE,53,00,01,00,ED} [Rx]

### CODE Load? message {....} [Tx]

Very long message, suspect its programmer payload

### CODE Load? Confirmation {BE,54,A2,01,20,2A} [Rx]

### CODE Run command? {BE,55,01,00,EB} [Tx]

### Programmer running / Programmer version {BE,60,00,06,03,01,00,90,00,00,47} [Rx]

Version 0x0103
Sector size 0x00009000

## Starting burning image to flash

### FLASH_CMD_GET_ID {BE,65,02,01,11,C8} [Tx]

### FLASH_CMD_GET_ID Response{BE,65,02,04,00,C8,60,16,98} [Rx]

ID: C8-60-16

### FLASH_CMD_GET_UNIQUE_ID {BE,65,03,01,12,C6} [Tx]

### FLASH_CMD_GET_UNIQUE_ID Resp {BE,65,03,11,00,30,31,32,33,49,0A,88,4A,41,53,FF,FF,FF,FF,FF,FF,5F} [Rx]

00000000: 30 31 32 33 39 0a 88 4a 41 53 ff ff ff ff ff ff 01239..JAS......

## Getting factory information

I have no idea what is going on here, documenting for the sake of it

### SYS_GET_CFGDATA {BE,03,04,08,00,E0,0F,3C,20,0,0,0,E7} [Tx]

## Actually programming

Start address appears to be 0x3C000000
Length of data was 0xcf90C but padded out to 0xd0000
Force to fill one sector 0x790C??

### ERASE BURN START {BE,61,07,0c,00,00,00,3C,00,00,0D,00,00,80,00,00,04} [Tx]

-> 0x3C000000 start address
-> 0xd0000 length

### ERASE BURN START Resp {BE,61,07,01,00,D8} [Rx]

### ERASE BURN DATA {BE,62,C1,0B,00,80,00,00,AB,77,7F,F4,00,00,00,FE,...DATA...} [Tx]

-> Sequence of 01 (0x0000)
-> Length 0x8000
-> CRC32 of data is F47F77AB
Thus
-> BE (header)
-> 62 (command)
-> C1 (sequence?)
-> 0b ???
-> 0800 (length)
-> 0000 ???
-> F47F77AB (CRC32)
-> 0000 Sequence
-> FE (checksum)

### ERASE BURN DATA {BE,62,C2,0B,00,80,00,00,34,90,61,F9,01,00,00,73,...DATA...} [Tx]

-> Sequence of 02 (0x8000)
-> Length 0x8000

### ERASE BURN DATA Resp {BE,62,C1,03,60,00,00,BB} [Rx]

-> Total blocks 26
-> Current burned block 1
-> Prog 7.407%

### ERASE BURN DATA {BE,62,C3,0B,00,80,00,00,67,80,10,8C,02,00,00,0C,...DATA} [Tx]

-> Sequence of 03 (0x10000)
-> Length 0x8000

### ERASE BURN DATA Resp {BE,62,C2,03,60,01,00,B9} [Rx]

-> Total blocks 26
-> Current burned block 2
-> Prog 11.111%

### ERASE BURN DATA {BE,62,C4,0B,00,80,00,00,FC,28,0D,3A,03,00,00,22,...DATA} [Tx]

-> Sequence of 04 (0x18000)
-> Length 0x8000

### ERASE BURN DATA Resp {BE,62,C3,03,60,02,00,B7} [Rx]

-> Total blocks 26
-> Current burned block 2
-> Prog 14.814

### BURN CMD BURN DATA {BE,65,08,09,22,00,00,00,3C,1C,EC,57,BE,50} [Tx]

-> 0x3C000000 start address

### BURN CMD BURN DATA Resp {BE,65,08,01,00,D3} [Rx]

Fin...
