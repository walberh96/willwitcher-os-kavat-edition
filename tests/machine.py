"""A virtual machine for the tests, and the checks that tests/boot.py and tests/install.py share.

The machine runs in QEMU with UEFI, no network and the host's GPU, as a normal user. The
tests reach it three ways: QEMU's guest agent runs commands inside it, and QEMU's VNC server
gives its screen and takes its keyboard.
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
BOOT_TIMEOUT = 240     # seconds until a system answers
DESKTOP_TIMEOUT = 120  # seconds, after that, until its desktop is up

KEYS = {'enter': 0xff0d, 'tab': 0xff09, 'esc': 0xff1b, 'backspace': 0xff08, 'space': 0x20,
        'shift': 0xffe1, 'ctrl': 0xffe3, 'alt': 0xffe9, 'super': 0xffeb,
        'up': 0xff52, 'down': 0xff54, 'left': 0xff51, 'right': 0xff53}
# characters that need Shift on a US keyboard, and the key each one is on
SHIFTED = dict(zip('~!@#$%^&*()_+{}|:"<>?', "`1234567890-=[]\\;',./"))


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


class Screen:
    """The machine's screen and keyboard, through QEMU's VNC server.

    QEMU's own screendump cannot read a screen that the GPU draws, so this speaks just enough
    of the VNC protocol: no password, uncompressed frames, key presses and clicks. Every call
    makes its own connection, because the screen changes size between the boot menu and the
    desktop.
    """

    def __init__(self, path):
        self.path = path

    def _connect(self):
        sock = socket.socket(socket.AF_UNIX)
        sock.settimeout(20)
        sock.connect(self.path)

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
        read(16)                                    # the server's pixel format
        read(struct.unpack('>I', read(4))[0])       # the machine's name
        return sock, read, width, height

    def _frame(self):
        """What the screen shows: its width, its height, and four bytes for each pixel."""
        sock, read, width, height = self._connect()
        # pixels as red, green, blue and a spare byte; uncompressed; one full frame
        sock.sendall(struct.pack('>B3xBBBBHHHBBB3x', 0, 32, 24, 0, 1, 255, 255, 255, 0, 8, 16))
        sock.sendall(struct.pack('>BxHi', 2, 1, 0))
        sock.sendall(struct.pack('>BBHHHH', 3, 0, 0, 0, width, height))

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
        return width, height, frame

    def pixel(self, x, y):
        """The colour of one point of the screen, as (red, green, blue)."""
        width, _, frame = self._frame()
        return tuple(frame[(y * width + x) * 4:(y * width + x) * 4 + 3])

    def picture(self, path):
        """Save what the screen shows as a PNG."""
        width, height, frame = self._frame()
        rgb = bytearray(width * height * 3)
        rgb[0::3], rgb[1::3], rgb[2::3] = frame[0::4], frame[1::4], frame[2::4]
        rows = b''.join(b'\x00' + rgb[i:i + width * 3] for i in range(0, len(rgb), width * 3))

        def chunk(name, data):
            return struct.pack('>I', len(data)) + name + data + struct.pack('>I', zlib.crc32(name + data))

        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'wb') as png:
            png.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                      + chunk(b'IDAT', zlib.compress(rows, 6)) + chunk(b'IEND', b''))

    def _press(self, sock, keys):
        codes = [KEYS[key] if key in KEYS else ord(key) for key in keys]
        for code in codes:
            sock.sendall(struct.pack('>BBxxI', 4, 1, code))
            time.sleep(0.03)
        for code in reversed(codes):
            sock.sendall(struct.pack('>BBxxI', 4, 0, code))
            time.sleep(0.03)

    def press(self, *keys):
        """Press keys together and let go: press('alt', 'n'), press('enter')."""
        sock = self._connect()[0]
        self._press(sock, keys)
        time.sleep(0.2)
        sock.close()

    def click(self, x, y):
        """Click with the left button at a point of the screen."""
        sock = self._connect()[0]
        # Telling the server which encodings we take is also when QEMU checks that the
        # machine has an absolute pointer; until then it moves the mouse by differences.
        sock.sendall(struct.pack('>BxHi', 2, 1, 0))
        for buttons in (0, 1, 0):                   # move there, press, let go
            sock.sendall(struct.pack('>BBHH', 5, buttons, x, y))
            time.sleep(0.1)
        time.sleep(0.2)
        sock.close()

    def type(self, text):
        """Type text as a US keyboard would."""
        sock = self._connect()[0]
        for char in text:
            if char in SHIFTED:
                self._press(sock, ['shift', SHIFTED[char]])
            elif char.isupper():
                self._press(sock, ['shift', char.lower()])
            else:
                self._press(sock, [char])
        time.sleep(0.2)
        sock.close()


class Machine:
    """A QEMU machine booting from an ISO, from a disk, or from the ISO with the disk attached.

    firmware is where the machine keeps its UEFI settings; give the same file to a later
    machine and it boots as the same computer. tpm is a folder: when given, the machine has a
    TPM2, a software one that keeps its state there, and the same folder is the same chip.
    """

    def __init__(self, iso=None, disk=None, firmware=None, tpm=None):
        self.dir = tempfile.mkdtemp(prefix='kavat-test-')
        self.tpm = None
        chip = []
        if tpm:
            os.makedirs(tpm, exist_ok=True)
            self.tpm = subprocess.Popen(['swtpm', 'socket', '--tpm2', '--tpmstate', f'dir={tpm}',
                                         '--ctrl', f'type=unixio,path={self.dir}/tpm.sock'],
                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            for _ in range(50):
                if os.path.exists(f'{self.dir}/tpm.sock'):
                    break
                time.sleep(0.1)
            chip = ['-chardev', f'socket,id=tpm,path={self.dir}/tpm.sock',
                    '-tpmdev', 'emulator,id=tpm0,chardev=tpm', '-device', 'tpm-tis,tpmdev=tpm0']
        if firmware is None:
            firmware = f'{self.dir}/vars.fd'
        if not os.path.exists(firmware):
            shutil.copy(f'{OVMF}/OVMF_VARS.4m.fd', firmware)
        drives = []
        if iso:
            drives += ['-drive', f'file={iso},media=cdrom,format=raw,readonly=on']
        if disk:
            drives += ['-drive', f'file={disk},format=qcow2,if=virtio']
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
            'qemu-system-x86_64', '-name', 'kavat-test', '-machine', 'q35,accel=kvm',
            '-cpu', 'host', '-smp', '4', '-m', '4G',
            '-drive', f'if=pflash,format=raw,readonly=on,file={OVMF}/OVMF_CODE.4m.fd',
            '-drive', f'if=pflash,format=raw,file={firmware}',
            *drives, *chip,
            '-vga', 'none', *display,
            '-device', 'virtio-tablet-pci',  # a pointer that goes where the tests click
            '-device', 'virtio-serial-pci',
            '-chardev', f'socket,id=agent,path={self.dir}/agent.sock,server=on,wait=off',
            '-device', 'virtserialport,chardev=agent,name=org.qemu.guest_agent.0',
            '-vnc', f'unix:{self.dir}/vnc.sock',
            '-nic', 'none', '-serial', 'none', '-parallel', 'none', '-monitor', 'none',
            '-no-reboot',
        ], stdout=self.log, stderr=subprocess.STDOUT)
        self.agent = Channel(f'{self.dir}/agent.sock', 15)
        self.screen = Screen(f'{self.dir}/vnc.sock')

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

    def wait_until(self, command, timeout):
        """Run the command until it succeeds or time runs out: (exit status, output)."""
        deadline = time.time() + timeout
        status, output = self.run(command)
        while status != 0 and time.time() < deadline:
            time.sleep(2)
            status, output = self.run(command)
        return status, output

    def power_off(self, timeout=90):
        """Shut the system down from inside and wait for the machine to go. True if it did."""
        self.agent.send('guest-exec', path='/bin/sh', arg=['-c', 'systemctl poweroff'])
        try:
            self.proc.wait(timeout)
            return True
        except subprocess.TimeoutExpired:
            return False

    def stop(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(10)
            except subprocess.TimeoutExpired:
                self.proc.kill()
        if self.tpm and self.tpm.poll() is None:
            self.tpm.terminate()
            self.tpm.wait(10)
        self.log.close()
        shutil.rmtree(self.dir, ignore_errors=True)


class Checks:
    """The list of what a test checked. failed turns True with the first check that fails."""

    def __init__(self):
        self.failed = False

    def report(self, ok, what, detail=''):
        self.failed = self.failed or not ok
        print(f"{'ok  ' if ok else 'FAIL'}  {what}", flush=True)
        if detail and not ok:
            print('      ' + detail.replace('\n', '\n      '), flush=True)
        return ok


def over_installed(name, encrypted=False):
    """Machine settings for a test that works on the disk that tests/install.py left.

    The machine gets a copy-on-write layer over that disk, and copies of its firmware settings
    and of its TPM2, all named after the test, so the installed disk does not change. With
    encrypted, the disk is the one from tests/install.py --encrypt. Returns the folder they
    are in and the settings for Machine.
    """
    folder = f'{ROOT}/out/test' + ('/encrypted' if encrypted else '')
    installed = f'{folder}/disk.qcow2'
    if not os.path.exists(installed):
        sys.exit(f'{os.path.relpath(folder)}/ holds no installed disk: run tests/install.py'
                 + (' --encrypt' if encrypted else '') + ' first')
    disk, firmware = f'{folder}/{name}.qcow2', f'{folder}/{name}-firmware.fd'
    if os.path.exists(disk):
        os.remove(disk)
    subprocess.run(['qemu-img', 'create', '-q', '-f', 'qcow2', '-b', installed, '-F', 'qcow2', disk], check=True)
    shutil.copy(f'{folder}/disk-firmware.fd', firmware)
    settings = {'disk': disk, 'firmware': firmware}
    if encrypted:
        settings['tpm'] = f'{folder}/{name}-tpm'
        shutil.rmtree(settings['tpm'], ignore_errors=True)
        shutil.copytree(f'{folder}/tpm', settings['tpm'])
    return folder, settings


def newest_iso():
    isos = sorted(glob.glob(f'{ROOT}/out/*.iso'), key=os.path.getmtime)
    if not isos:
        sys.exit('out/ holds no ISO: build one with tools/build-iso.sh')
    return isos[-1]


def check_desktop(machine, checks, user, timeout=DESKTOP_TIMEOUT):
    """Check that the user's session has Hyprland and Noctalia up and drawing."""
    # hyprctl as that user; the machine runs a single Hyprland
    hyprctl = f'runuser -u {user} -- env XDG_RUNTIME_DIR=/run/user/$(id -u {user}) hyprctl --instance 0'
    deadline = time.time() + timeout
    for what, command in [
        (f"Hyprland runs in {user}'s session", f'pgrep -u {user} -x Hyprland'),
        ('Hyprland answers', f'{hyprctl} version'),
        ('Noctalia runs', f'pgrep -u {user} -x noctalia'),
        ('Noctalia has drawn on the screen', f'{hyprctl} layers | grep -i noctalia'),
    ]:
        status, output = machine.wait_until(command, deadline - time.time())
        checks.report(status == 0, what, output)
    status, output = machine.run(f'{hyprctl} configerrors')
    checks.report(status == 0 and not output, "Hyprland's configuration loads without errors", output)


def check_packages(machine, checks):
    # a pacman that is not Arch's leaves fields of its own in the package database,
    # and Arch's pacman complains about them every time it reads it
    _, output = machine.run("pacman -Q 2>&1 >/dev/null | grep 'local database' | head -n 5")
    checks.report(not output, 'the package database is clean', output)


def print_journal(machine):
    _, journal = machine.run('journalctl -b --no-pager -p warning | tail -n 40')
    print('\nlast warnings of the machine:\n' + journal, flush=True)
