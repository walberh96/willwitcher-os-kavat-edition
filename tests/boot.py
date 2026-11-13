#!/usr/bin/env python3
"""Boot the ISO in a virtual machine and check that it reaches the desktop.

usage: tests/boot.py [iso]
  iso   the image to boot; the newest one in out/ when not given

This is the gate of phase 1. The machine boots from the ISO with UEFI and no network. The
checks run inside it, through QEMU's guest agent, and a picture of the screen is left in
out/test/desktop.png. The exit status is 0 only when every check passes.

Needs qemu-desktop and edk2-ovmf. It runs as a normal user.
"""
import base64
import glob
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OVMF = '/usr/share/edk2/x64'
BOOT_TIMEOUT = 240     # seconds until the system answers
DESKTOP_TIMEOUT = 120  # seconds, after that, until the desktop is up
# hyprctl as the live user; the machine runs a single Hyprland
HYPRCTL = 'runuser -u live -- env XDG_RUNTIME_DIR=/run/user/1000 hyprctl --instance 0'


class Channel:
    """A JSON conversation with the guest agent, over its socket: one request, one answer."""

    def __init__(self, path, timeout):
        deadline = time.time() + timeout
        while True:
            try:
                self.sock = socket.socket(socket.AF_UNIX)
                self.sock.connect(path)
                break
            except OSError:
                if time.time() > deadline:
                    raise
                time.sleep(0.2)
        self.buf = b''

    def send(self, name, **arguments):
        message = {'execute': name}
        if arguments:
            message['arguments'] = arguments
        self.sock.sendall(json.dumps(message).encode() + b'\n')

    def read(self, timeout):
        """The next answer, or None when none comes in time. Events and noise are skipped."""
        deadline = time.time() + timeout
        while True:
            while b'\n' in self.buf:
                line, self.buf = self.buf.split(b'\n', 1)
                try:
                    message = json.loads(line)
                except ValueError:
                    continue
                if 'return' in message or 'error' in message:
                    return message
            left = deadline - time.time()
            if left <= 0:
                return None
            self.sock.settimeout(left)
            try:
                data = self.sock.recv(65536)
            except socket.timeout:
                return None
            if not data:
                return None
            self.buf += data

    def ask(self, name, timeout=30, **arguments):
        self.send(name, **arguments)
        answer = self.read(timeout)
        if answer is None:
            raise TimeoutError(f'{name}: no answer')
        if 'error' in answer:
            raise RuntimeError(f"{name}: {answer['error'].get('desc', answer['error'])}")
        return answer['return']


def grab(vnc, path):
    """Save what the machine's screen shows as a PNG, asking its VNC server for one frame.

    QEMU's own screendump cannot read a screen that the GPU draws, so this speaks just
    enough of the VNC protocol: no password, one full frame, uncompressed.
    """
    sock = socket.socket(socket.AF_UNIX)
    sock.settimeout(20)
    sock.connect(vnc)

    def read(n):
        data = bytearray()
        while len(data) < n:
            chunk = sock.recv(n - len(data))
            if not chunk:
                raise ConnectionError('the VNC server closed the connection')
            data += chunk
        return bytes(data)

    read(12)                                    # the server's protocol version
    sock.sendall(b'RFB 003.008\n')
    if 1 not in read(read(1)[0]):               # 1 is "no password"
        raise RuntimeError('the VNC server asks for a password')
    sock.sendall(b'\x01')
    if struct.unpack('>I', read(4))[0] != 0:
        raise RuntimeError('the VNC server refused the connection')
    sock.sendall(b'\x01')                       # share the screen with other viewers
    width, height = struct.unpack('>HH', read(4))
    read(16)                                    # the server's pixel format; ours is set below
    read(struct.unpack('>I', read(4))[0])       # the machine's name
    # pixels as red, green, blue and a spare byte
    sock.sendall(struct.pack('>B3xBBBBHHHBBB3x', 0, 32, 24, 0, 1, 255, 255, 255, 0, 8, 16))
    sock.sendall(struct.pack('>BxHi', 2, 1, 0))                    # uncompressed only
    sock.sendall(struct.pack('>BBHHHH', 3, 0, 0, 0, width, height))  # one full frame

    frame = bytearray(width * height * 4)
    while True:
        kind = read(1)[0]
        if kind == 0:
            break
        if kind == 2:                           # a bell
            continue
        if kind == 3:                           # clipboard text
            read(struct.unpack('>3xI', read(7))[0])
            continue
        raise RuntimeError(f'unexpected VNC message {kind}')
    for _ in range(struct.unpack('>xH', read(3))[0]):
        x, y, w, h, encoding = struct.unpack('>HHHHi', read(12))
        if encoding != 0:
            raise RuntimeError(f'unexpected VNC encoding {encoding}')
        pixels = read(w * h * 4)
        for row in range(h):
            at = ((y + row) * width + x) * 4
            frame[at:at + w * 4] = pixels[row * w * 4:(row + 1) * w * 4]
    sock.close()

    rgb = bytearray(width * height * 3)
    rgb[0::3], rgb[1::3], rgb[2::3] = frame[0::4], frame[1::4], frame[2::4]
    rows = b''.join(b'\x00' + rgb[i:i + width * 3] for i in range(0, len(rgb), width * 3))

    def chunk(name, data):
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', zlib.crc32(name + data))

    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'wb') as png:
        png.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                  + chunk(b'IDAT', zlib.compress(rows, 6)) + chunk(b'IEND', b''))


class Machine:
    def __init__(self, iso):
        self.dir = tempfile.mkdtemp(prefix='kavat-test-')
        shutil.copy(f'{OVMF}/OVMF_VARS.4m.fd', f'{self.dir}/vars.fd')
        render = sorted(glob.glob('/dev/dri/renderD*'))
        if render:
            # the host's GPU draws for the machine: Hyprland needs real OpenGL
            display = ['-device', 'virtio-vga-gl,xres=1920,yres=1080',
                       '-display', f'egl-headless,rendernode={render[0]}']
        else:
            print('no GPU render node on this host: the desktop may not start', file=sys.stderr)
            display = ['-device', 'virtio-vga,xres=1920,yres=1080', '-display', 'none']
        self.log = open(f'{self.dir}/qemu.log', 'w')
        self.proc = subprocess.Popen([
            'qemu-system-x86_64', '-name', 'kavat-test',
            '-machine', 'q35,accel=kvm', '-cpu', 'host', '-smp', '4', '-m', '4G',
            '-drive', f'if=pflash,format=raw,readonly=on,file={OVMF}/OVMF_CODE.4m.fd',
            '-drive', f'if=pflash,format=raw,file={self.dir}/vars.fd',
            '-drive', f'file={iso},media=cdrom,format=raw,readonly=on',
            '-vga', 'none', *display,
            '-device', 'virtio-serial-pci',
            '-chardev', f'socket,id=agent,path={self.dir}/agent.sock,server=on,wait=off',
            '-device', 'virtserialport,chardev=agent,name=org.qemu.guest_agent.0',
            '-vnc', f'unix:{self.dir}/vnc.sock',
            '-nic', 'none', '-serial', 'none', '-parallel', 'none', '-monitor', 'none',
            '-no-reboot',
        ], stdout=self.log, stderr=subprocess.STDOUT)
        self.agent = Channel(f'{self.dir}/agent.sock', 15)

    def wait_for_agent(self, timeout):
        """True once the guest agent answers: the system has booted."""
        deadline = time.time() + timeout
        token = 1
        while time.time() < deadline:
            if self.proc.poll() is not None:
                return False
            token += 1
            self.agent.send('guest-sync', id=token)
            end = time.time() + 2
            while time.time() < end:
                answer = self.agent.read(end - time.time())
                if answer and answer.get('return') == token:
                    return True
        return False

    def run(self, command, timeout=30):
        """Run a shell command inside the machine, as root: (exit status, output)."""
        pid = self.agent.ask('guest-exec', path='/bin/sh', arg=['-c', command],
                             **{'capture-output': True})['pid']
        deadline = time.time() + timeout
        while time.time() < deadline:
            status = self.agent.ask('guest-exec-status', pid=pid)
            if status['exited']:
                out = base64.b64decode(status.get('out-data', '')).decode(errors='replace')
                err = base64.b64decode(status.get('err-data', '')).decode(errors='replace')
                return status.get('exitcode', 1), (out + err).strip()
            time.sleep(0.3)
        return 124, 'timed out'

    def screenshot(self, path):
        grab(f'{self.dir}/vnc.sock', path)

    def stop(self):
        # the machine holds nothing worth saving: it ran from the ISO, in memory
        self.proc.terminate()
        try:
            self.proc.wait(10)
        except subprocess.TimeoutExpired:
            self.proc.kill()
        self.log.close()
        shutil.rmtree(self.dir, ignore_errors=True)


def main():
    if len(sys.argv) > 2 or (len(sys.argv) == 2 and sys.argv[1].startswith('-')):
        sys.exit(__doc__)
    if len(sys.argv) == 2:
        iso = sys.argv[1]
    else:
        isos = sorted(glob.glob(f'{ROOT}/out/*.iso'), key=os.path.getmtime)
        if not isos:
            sys.exit('out/ holds no ISO: build one with tools/build-iso.sh')
        iso = isos[-1]
    print(f'booting {os.path.relpath(iso)}')

    picture = f'{ROOT}/out/test/desktop.png'
    failed = False

    def report(ok, what, detail=''):
        nonlocal failed
        failed = failed or not ok
        print(f"{'ok  ' if ok else 'FAIL'}  {what}")
        if detail and not ok:
            print('      ' + detail.replace('\n', '\n      '))

    machine = Machine(iso)
    try:
        booted = machine.wait_for_agent(BOOT_TIMEOUT)
        report(booted, 'the system boots')
        if booted:
            # each check is a command that succeeds once its part of the desktop is up
            checks = [
                ('Hyprland runs in the live session', 'pgrep -u live -x Hyprland'),
                ('Hyprland answers', f'{HYPRCTL} version'),
                ('Noctalia runs', 'pgrep -u live -x noctalia'),
                ('Noctalia has drawn on the screen', f'{HYPRCTL} layers | grep -i noctalia'),
            ]
            deadline = time.time() + DESKTOP_TIMEOUT
            for what, command in checks:
                status, output = machine.run(command)
                while status != 0 and time.time() < deadline:
                    time.sleep(2)
                    status, output = machine.run(command)
                report(status == 0, what, output)

            status, output = machine.run(f'{HYPRCTL} configerrors')
            report(status == 0 and not output, "Hyprland's configuration loads without errors", output)

            # a pacman that is not Arch's leaves fields of its own in the package database,
            # and Arch's pacman complains about them every time it reads it
            _, output = machine.run("pacman -Q 2>&1 >/dev/null | grep 'local database' | head -n 5")
            report(not output, 'the package database is clean', output)

            if failed:
                _, journal = machine.run('journalctl -b --no-pager -p warning | tail -n 40')
                print('\nlast warnings of the machine:\n' + journal)
            else:
                time.sleep(3)  # let the shell finish drawing before the picture
        machine.screenshot(picture)
        print(f'picture of the screen: {os.path.relpath(picture)}')
    finally:
        machine.stop()
    sys.exit(1 if failed else 0)


if __name__ == '__main__':
    main()
