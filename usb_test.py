#!/usr/bin/env python3

import usb1
import random
import sys

from time import sleep
# import struct


class USB():

    # VID = 0x16c0
    # PID = 0x05dc
    VID = 0x2e8a
    PID = 0xbeef

    def __init__(self):
        self.context = usb1.USBContext()
        self.handle = self.context.openByVendorIDAndProductID(
            self.VID,
            self.PID,
        )
        if self.handle is None:
            print("handle is None")
            sys.exit()
        self.request_type = usb1.REQUEST_TYPE_VENDOR | usb1.RECIPIENT_DEVICE

    def led(self):
        print("toggling led")
        request = 0x3
        value = 0
        index = 0
        data = b'\xaa'
        timeout = 5000
        self.handle.controlWrite(
            self.request_type,
            request,
            value,
            index,
            data,
            timeout
        )

    def test(self, data):
        with self.handle.claimInterface(2):
            self.handle.bulkWrite(0x02, b'\x01', 5000)



def main(argv):

    if len(argv) == 0:
        print(__doc__, file=sys.stderr)
        return -1

    usb = USB()

    if argv[0] == 'led':
        usb.led()
        return 0
    elif argv[0] == 'test':
        print("argv[0] ::", argv[0])
        print("argv[1] ::", argv[1])
        arg = int(argv[1])
        usb.test(bytes((arg,)))
        return 0
    elif argv[0] == 'get':
        print("getting")
        try:
            with usb.handle.claimInterface(2):
                data = usb.handle.bulkRead(0x83, 64, 0)
            print("data ::", data)
            return 0
        except usb1.USBErrorTimeout:
            pass

    elif argv[0] == 'set':
        print("setting")
        with usb.handle.claimInterface(2):
            a = bytearray(random.getrandbits(8) for _ in range(17))
            usb.handle.bulkWrite(0x02, a, 5000)
            data = usb.handle.bulkRead(0x83, 64, 0)
            print("data ::", data)
        return 0


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1:]))
    except Exception as e:
        print("Fatal error:", repr(e), file=sys.stderr)
        sys.exit(1)
