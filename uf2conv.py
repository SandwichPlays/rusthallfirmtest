#!/usr/bin/env python3
import sys
import struct

# UF2 Conversion Script (Simplified)
# Usage: python3 uf2conv.py firmware.bin firmware.uf2

def convert(base_addr, input_file, output_file):
    with open(input_file, "rb") as f:
        data = f.read()
    
    num_blocks = (len(data) + 255) // 256
    with open(output_file, "wb") as f:
        for i in range(num_blocks):
            block = data[i*256 : (i+1)*256]
            if len(block) < 256:
                block += b'\x00' * (256 - len(block))
            
            header = struct.pack("<IIIIIIII", 
                0x0A324655, # Magic 0
                0x9E5D5157, # Magic 1
                0x00002000, # Flags (Family ID present)
                base_addr + i*256,
                256,
                i,
                num_blocks,
                0x5775207f  # Family ID for STM32F4
            )
            footer = struct.pack("<I", 0x0AB16F30)
            f.write(header + block + b'\x00'*212 + footer)

if __name__ == "__main__":
    convert(0x08010000, sys.argv[1], sys.argv[2])
