"""Install the USB prototype alongside the factory app on the bench unit."""

import argparse
import hashlib
from pathlib import Path
import struct
import sys
import zlib

import esptool
from esptool.cmds import detect_flash_size
from esptool.util import FatalError
from serial import SerialException
from serial.tools import list_ports


SECTOR = 0x1000
FLASH_SIZE = 0x1000000
TABLE = 0x8000
OTA_DATA = 0xE000
FACTORY_APP = 0x10000
PROTOTYPE_APP = 0x210000
APP_SIZE = 0x200000
ERASED_SECTOR = b"\xff" * SECTOR


def ota_record(sequence):
    sequence_bytes = struct.pack("<I", sequence)
    checksum = zlib.crc32(sequence_bytes, 0xFFFFFFFF)
    return struct.pack("<I20sII", sequence, b"\xff" * 20, 2, checksum).ljust(
        SECTOR, b"\xff"
    )


def validate_backup(backup):
    if len(backup) != FLASH_SIZE or backup[0] != 0xE9 or backup[FACTORY_APP] != 0xE9:
        raise ValueError("Expected a complete 16 MiB factory flash image.")

    required = {
        (1, 0): (OTA_DATA, 2 * SECTOR),
        (0, 0x10): (FACTORY_APP, APP_SIZE),
        (0, 0x11): (PROTOTYPE_APP, APP_SIZE),
    }
    found = {}
    for offset in range(TABLE, TABLE + SECTOR, 32):
        magic, kind, subtype, address, size = struct.unpack_from("<HBBII", backup, offset)
        if magic != 0x50AA:
            break
        if (kind, subtype) in required:
            if (kind, subtype) in found:
                raise ValueError("Duplicate OTA partition in the factory backup.")
            found[kind, subtype] = (address, size)
    if found != required or backup[PROTOTYPE_APP:PROTOTYPE_APP + APP_SIZE] != b"\xff" * APP_SIZE:
        raise ValueError("The factory backup does not have the expected empty app1 slot.")

    sequence, _, state, crc = struct.unpack_from("<I20sII", backup, OTA_DATA)
    if (sequence, state, crc) != (1, 2, zlib.crc32(struct.pack("<I", 1), 0xFFFFFFFF)):
        raise ValueError("The factory app is not selected as expected in the backup.")
    if backup[OTA_DATA + SECTOR:OTA_DATA + 2 * SECTOR] != ERASED_SECTOR:
        raise ValueError("The second boot-selection sector was already used in the backup.")


def matches(esp, address, data):
    return esp.flash_md5sum(address, len(data)) == hashlib.md5(data).hexdigest()


def restart(esp):
    try:
        esptool.reset_chip(esp, "watchdog-reset")
    except SerialException as error:
        # Windows invalidates the open handle when the native USB device resets.
        reason = next((arg for arg in error.args if isinstance(arg, OSError)), None)
        if getattr(reason, "winerror", None) != 433 and "None, 433)" not in str(error):
            raise
        print("USB disconnected during reset. Confirm that the selected app starts.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", required=True)
    parser.add_argument("--backup", required=True, type=Path)
    parser.add_argument("--before", choices=("default-reset", "no-reset"), default="no-reset")
    operation = parser.add_mutually_exclusive_group(required=True)
    operation.add_argument("--image", type=Path, help="Application .bin from build.ps1")
    operation.add_argument("--check", action="store_true", help="Check the board and backup")
    operation.add_argument("--factory", action="store_true", help="Boot the factory app again")
    args = parser.parse_args()

    usb = next((port for port in list_ports.comports() if port.device == args.port), None)
    if usb is None or (usb.vid, usb.pid) != (0x303A, 0x1001):
        raise ValueError("Select the ESP32-S3 native USB Serial/JTAG port.")

    backup = args.backup.read_bytes()
    validate_backup(backup)
    image = None
    if args.image is not None:
        if not args.image.name.endswith(".ino.bin") or not 24 <= args.image.stat().st_size <= APP_SIZE:
            raise ValueError("Use the built application .ino.bin (maximum 2 MiB).")
        image = args.image.read_bytes()
        if image[0] != 0xE9 or struct.unpack_from("<H", image, 12)[0] != 9:
            raise ValueError("Expected an ESP32-S3 app image.")

    with esptool.detect_chip(port=args.port, connect_mode=args.before) as rom:
        if rom.CHIP_NAME != "ESP32-S3" or rom.secure_download_mode:
            raise ValueError("Expected ESP32-S3 in standard serial download mode.")
        if rom.get_secure_boot_enabled() or rom.get_flash_encryption_enabled():
            raise ValueError("This prototype expects secure boot and flash encryption disabled.")

        esp = esptool.run_stub(rom)
        esptool.attach_flash(esp)
        if detect_flash_size(esp) != "16MB":
            raise ValueError("Expected a 16 MiB flash chip.")

        for address, size in ((0, TABLE + SECTOR), (FACTORY_APP, APP_SIZE)):
            if not matches(esp, address, backup[address:address + size]):
                raise ValueError("Board bootloader, partition table or factory app differs from backup.")
        print("Factory bootloader, partition table and app0 match the backup.", flush=True)

        factory_selection = backup[OTA_DATA:OTA_DATA + SECTOR]
        if not matches(esp, OTA_DATA, factory_selection):
            raise ValueError("Factory boot selection differs from backup.")
        selector_address = OTA_DATA + SECTOR
        prototype_selected = matches(esp, selector_address, ota_record(2))
        if not prototype_selected and not matches(esp, selector_address, ERASED_SECTOR):
            raise ValueError("Unexpected data in the second boot-selection sector.")

        if args.check:
            esptool.reset_chip(esp, "no-reset-stub")
            return

        if args.factory:
            if prototype_selected:
                esptool.write_flash(esp, [(selector_address, ERASED_SECTOR)], no_progress=True)
            print("Factory app selected.", flush=True)
        else:
            # Leave a bootable app selected if a replacement write is interrupted.
            if prototype_selected:
                esptool.write_flash(esp, [(selector_address, ERASED_SECTOR)], no_progress=True)
            esptool.write_flash(esp, [(PROTOTYPE_APP, image)], no_progress=True)
            esptool.verify_flash(esp, [(PROTOTYPE_APP, image)])
            esptool.write_flash(esp, [(selector_address, ota_record(2))], no_progress=True)
            print("Prototype selected. Factory app0 remains available.", flush=True)

        restart(esp)


if __name__ == "__main__":
    try:
        main()
    except (FatalError, OSError, ValueError) as error:
        sys.exit(str(error))
