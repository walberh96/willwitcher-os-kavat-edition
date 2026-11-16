#!/usr/bin/python3
"""kavat-greeter: the login screen of Willwitcher-OS Kavat Edition.

greetd starts a small Wayland session for it and waits. This program shows the login page,
a web page in greeter/theme/, on every screen, and stands between the page and greetd: the
page asks for a person, greetd asks for the password, and when it is right the page names
the session to start. Then this program ends, the small session ends with it, and greetd
starts the person's own.

The page reaches this program as the object `greeter`, through Qt's web channel. It can read
no address but files of this machine.
"""
import configparser
import glob
import json
import os
import pwd
import shlex
import socket
import struct
import subprocess
import sys
import threading

from PySide6.QtCore import Property, QObject, QUrl, Signal, Slot
from PySide6.QtGui import QColor
from PySide6.QtWebChannel import QWebChannel
from PySide6.QtWebEngineCore import QWebEngineUrlRequestInterceptor
from PySide6.QtWebEngineWidgets import QWebEngineView
from PySide6.QtWidgets import QApplication

PAGE = os.environ.get('KAVAT_GREETER_PAGE', '/usr/share/kavat-greeter/page')
SESSIONS = '/usr/share/wayland-sessions'
DEFAULT_SESSION = 'kavat'
LAYOUTS = ['us', 'es']  # as the small session's own settings name them, in that order


class Greetd:
    """The conversation with greetd, over its socket: one request, one answer."""

    def __init__(self):
        path = os.environ.get('GREETD_SOCK')
        self.sock = None
        if path:
            self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self.sock.connect(path)

    def ask(self, **request):
        if self.sock is None:
            return {'type': 'error', 'description': 'greetd is not there: this is not a login'}
        body = json.dumps(request).encode()
        self.sock.sendall(struct.pack('=I', len(body)) + body)
        length = struct.unpack('=I', self._read(4))[0]
        return json.loads(self._read(length))

    def _read(self, count):
        data = b''
        while len(data) < count:
            chunk = self.sock.recv(count - len(data))
            if not chunk:
                raise ConnectionError('greetd closed the conversation')
            data += chunk
        return data


def people():
    """Who can log in: the accounts of people, not of services."""
    found = []
    for entry in pwd.getpwall():
        if 1000 <= entry.pw_uid < 60000 and not entry.pw_shell.endswith(('nologin', 'false')):
            name = entry.pw_gecos.split(',')[0] or entry.pw_name
            face = os.path.join(entry.pw_dir, '.face')
            found.append({'username': entry.pw_name, 'display_name': name,
                          'image': face if os.access(face, os.R_OK) else ''})
    return found


def sessions():
    """The desktop sessions installed, each with the command that starts it."""
    found = []
    for path in sorted(glob.glob(f'{SESSIONS}/*.desktop')):
        entry = configparser.ConfigParser(interpolation=None, strict=False)
        try:
            entry.read(path, encoding='utf-8')
            section = entry['Desktop Entry']
            if section.get('Hidden', 'false') == 'true' or section.get('NoDisplay', 'false') == 'true':
                continue
            found.append({'key': os.path.basename(path)[:-len('.desktop')], 'name': section['Name'],
                          'command': section['Exec'], 'desktop': section.get('DesktopNames', '').strip(';')})
        except (configparser.Error, KeyError, OSError):
            continue
    return found


class Greeter(QObject):
    """What the page sees as `greeter`."""

    # greetd asks for the password of the person named with authenticate()
    prompt = Signal(str)
    # something greetd wants shown
    message = Signal(str)
    # how the login went: True, and start_session() can be called
    complete = Signal(bool)
    nothing = Signal()

    def __init__(self, application):
        super().__init__()
        self.application = application
        self.greetd = Greetd()
        self.lock = threading.Lock()
        self._sessions = sessions()
        self._layout = 0

    @Property(list, notify=nothing)
    def users(self):
        return people()

    @Property(list, notify=nothing)
    def sessions(self):
        return [{'key': session['key'], 'name': session['name']} for session in self._sessions]

    @Property(str, notify=nothing)
    def default_session(self):
        keys = [session['key'] for session in self._sessions]
        return DEFAULT_SESSION if DEFAULT_SESSION in keys else (keys[0] if keys else '')

    @Property(list, notify=nothing)
    def layouts(self):
        return LAYOUTS

    @Property(str, notify=nothing)
    def language(self):
        return os.environ.get('LANG', 'en_US.UTF-8')

    def _answer(self, answer):
        """Tells the page what greetd said."""
        kind = answer.get('type')
        if kind == 'success':
            self.complete.emit(True)
        elif kind == 'auth_message':
            text = answer.get('auth_message', '')
            if answer.get('auth_message_type') in ('secret', 'visible'):
                self.prompt.emit(text)
            else:
                # something to read, not to answer: greetd goes on once it is acknowledged
                self.message.emit(text)
                self._answer(self.greetd.ask(type='post_auth_message_response'))
        else:
            self.greetd.ask(type='cancel_session')
            self.complete.emit(False)

    def _talk(self, **request):
        """One step of the conversation, off the thread that draws: a wrong password makes
        greetd wait a moment before it answers."""
        def step():
            with self.lock:
                try:
                    self._answer(self.greetd.ask(**request))
                except (OSError, ValueError):
                    self.complete.emit(False)
        threading.Thread(target=step, daemon=True).start()

    @Slot(str)
    def authenticate(self, username):
        def start():
            with self.lock:
                try:
                    self.greetd.ask(type='cancel_session')
                    self._answer(self.greetd.ask(type='create_session', username=username))
                except (OSError, ValueError):
                    self.complete.emit(False)
        threading.Thread(target=start, daemon=True).start()

    @Slot(str)
    def respond(self, text):
        self._talk(type='post_auth_message_response', response=text)

    @Slot(str)
    def start_session(self, key):
        session = next((session for session in self._sessions if session['key'] == key), None)
        if session is None:
            self.complete.emit(False)
            return
        environment = ['XDG_SESSION_TYPE=wayland', f'XDG_SESSION_DESKTOP={key}']
        if session['desktop']:
            environment.append(f'XDG_CURRENT_DESKTOP={session["desktop"]}')
        # greetd runs the command through a shell that has read /etc/profile first
        answer = self.greetd.ask(type='start_session', env=environment, cmd=shlex.split(session['command']))
        if answer.get('type') == 'success':
            self.application.quit()
        else:
            self.complete.emit(False)

    @Slot(result=str)
    def next_layout(self):
        """Changes the keyboard to the next layout, and says which one it is now."""
        subprocess.run(['hyprctl', 'switchxkblayout', 'all', 'next'], capture_output=True, check=False)
        self._layout = (self._layout + 1) % len(LAYOUTS)
        return LAYOUTS[self._layout]

    @Slot()
    def suspend(self):
        subprocess.run(['systemctl', 'suspend'], check=False)

    @Slot()
    def restart(self):
        subprocess.run(['systemctl', 'reboot'], check=False)

    @Slot()
    def shutdown(self):
        subprocess.run(['systemctl', 'poweroff'], check=False)


class OnlyThisMachine(QWebEngineUrlRequestInterceptor):
    """The page reads files of this machine and nothing else."""

    def interceptRequest(self, request):
        request.block(request.requestUrl().scheme() not in ('file', 'qrc', 'data'))


def main():
    application = QApplication(sys.argv)
    greeter = Greeter(application)
    channel = QWebChannel()
    channel.registerObject('greeter', greeter)
    guard = OnlyThisMachine()
    views = {}

    def show_on(screen):
        """The page on one screen: the main one asks who logs in, the others show the clock."""
        if screen in views:
            return
        view = QWebEngineView()
        view.page().profile().setUrlRequestInterceptor(guard)
        view.page().setBackgroundColor(QColor(0, 0, 0, 0))
        view.page().setWebChannel(channel)
        asks = not any(other.property('asks') for other in views.values())
        view.setProperty('asks', asks)
        view.setUrl(QUrl.fromLocalFile(os.path.join(PAGE, 'index.html' if asks else 'beside.html')))
        view.setGeometry(screen.geometry())
        view.winId()
        view.windowHandle().setScreen(screen)
        view.showFullScreen()
        views[screen] = view

    def gone(screen):
        view = views.pop(screen, None)
        if view is None:
            return
        asked = view.property('asks')
        view.close()
        view.deleteLater()
        # the screen that asked who logs in was unplugged: another one asks
        if asked and views:
            other = next(iter(views.values()))
            other.setProperty('asks', True)
            other.setUrl(QUrl.fromLocalFile(os.path.join(PAGE, 'index.html')))

    # the main screen first, so that it is the one that asks
    show_on(application.primaryScreen())
    for screen in application.screens():
        show_on(screen)
    # a screen may wake up after the session has started, or be plugged in later: it gets
    # its page then, and is not left black
    application.screenAdded.connect(show_on)
    application.screenRemoved.connect(gone)
    sys.exit(application.exec())


if __name__ == '__main__':
    main()
