"""Drive real GTK and egui windows on a private Xvfb display; never use DISPLAY=:0."""
import argparse, contextlib, json, os, pathlib, select, subprocess, time
from PIL import Image, ImageGrab, ImageDraw
HERE = pathlib.Path(__file__).resolve().parent

def wait_for(read, predicate=lambda value: value is not None, seconds=30):
    until = time.monotonic() + seconds
    last = None
    while time.monotonic() < until:
        try:
            last = read()
            if predicate(last):
                return last
        except (OSError, ValueError, IndexError, subprocess.SubprocessError):
            pass
        time.sleep(0.025)
    raise RuntimeError(f'Timed out waiting for fixture: {last!r}')

class Driver:

    def __init__(self, out, runtime, binary):
        self.out, self.runtime, self.binary = (out, runtime, binary)
        self.env = dict(os.environ)
        self.env.pop('WAYLAND_DISPLAY', None)
        self.env.pop('GTK_MODULES', None)
        self.env.pop('GTK_THEME', None)
        self.env['GTK_IM_MODULE'] = 'gtk-im-context-simple'
        self.env['GTK_A11Y'] = 'none'
        self.env['DBUS_SESSION_BUS_ADDRESS'] = 'unix:path=' + str(out / 'no-session-bus')
        self.env.update(GTK_PARITY_OUT=str(out), GDK_BACKEND='x11', GSK_RENDERER='cairo', GDK_SCALE='1', GDK_DPI_SCALE='1', WINIT_X11_SCALE_FACTOR='1', WINIT_UNIX_BACKEND='x11', LIBGL_ALWAYS_SOFTWARE='1', NO_AT_BRIDGE='1', LC_ALL='C.UTF-8', GIO_USE_VFS='local', GTK_USE_PORTAL='0', GSETTINGS_BACKEND='memory', XDG_CONFIG_HOME=str(out / 'config'), XDG_CACHE_HOME=str(out / 'cache'))
        lib = runtime / 'usr/lib/x86_64-linux-gnu'
        self.env['LD_LIBRARY_PATH'] = str(lib)
        self.env['GI_TYPELIB_PATH'] = str(lib / 'girepository-1.0')
        self.xdo = runtime / 'usr/bin/xdotool'
        self.seq = 0
        self.traces = []

    def x(self, *args):
        return subprocess.check_output([str(self.xdo), *map(str, args)], env=self.env, text=True, timeout=4).strip()

    @contextlib.contextmanager
    def display(self):
        with (self.out / 'xvfb.log').open('w') as log:
            server = subprocess.Popen([str(self.runtime / 'usr/bin/Xvfb'), '-displayfd', '1', '-screen', '0', '1100x1000x24', '-dpi', '96', '-nolisten', 'tcp', '-ac', '-noreset'], env=self.env, stdout=subprocess.PIPE, stderr=log, text=True)
            try:
                if not select.select([server.stdout], [], [], 8)[0]:
                    raise RuntimeError('Xvfb did not allocate an isolated display')
                number = server.stdout.readline().strip()
                if not number.isdecimal():
                    raise RuntimeError(f'Invalid Xvfb display: {number!r}')
                self.env['DISPLAY'] = ':' + number
                yield
            finally:
                server.terminate()
                server.wait(timeout=5)

    def state(self, app):
        return wait_for(lambda: json.loads((self.out / (app + '-state.json')).read_text()))

    def command(self, app, **values):
        self.seq += 1
        values['seq'] = self.seq
        temp = self.out / (app + '-command.tmp')
        temp.write_text(json.dumps(values))
        temp.replace(self.out / (app + '-command.json'))
        ack = wait_for(lambda: self.state(app), lambda s: s.get('sequence') == self.seq)
        return wait_for(lambda: self.state(app), lambda s: s['frame'] >= ack['frame'] + 2)

    def key(self, value):
        self.x('key', '--clearmodifiers', value)
        time.sleep(0.12)

    def type(self, value):
        self.x('type', '--clearmodifiers', '--delay', '2', '--', value)
        time.sleep(0.12)

    def click(self, point):
        self.x('mousemove', *point)
        self.x('click', '1')
        time.sleep(0.15)

    @contextlib.contextmanager
    def app(self, app, mode='light', disabled=False):
        for suffix in ['-state.json', '-command.json']:
            (self.out / (app + suffix)).unlink(missing_ok=True)
        command = ['/usr/bin/python3', str(HERE / 'reference.py'), mode] if app == 'gtk' else [str(self.binary), mode]
        env = {**self.env, 'STATE': 'disabled' if disabled else 'normal'}
        with (self.out / f'{app}-{mode}-{('disabled' if disabled else 'normal')}.log').open('w') as log:
            process = subprocess.Popen(command, env=env, stdout=log, stderr=subprocess.STDOUT)
            try:
                title = 'GTK reference' if app == 'gtk' else 'egui comparison'
                window = wait_for(lambda: self.x('search', '--onlyvisible', '--name', '^' + title + '$').splitlines()[-1])
                self.x('windowmove', window, 0, 0)
                self.x('windowfocus', '--sync', window)
                state = self.state(app)
                time.sleep(0.3)
                if app == 'gtk':
                    (self.out / 'metadata.json').write_text(json.dumps({key: state[key] for key in ['gtk_version', 'adwaita_version', 'font', 'theme']}, indent=2))
                yield (window, state)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()

    def snapshot(self, name, rect=None):
        image = ImageGrab.grab(xdisplay=self.env['DISPLAY'])
        image.crop(rect or (0, 0, 900, 900)).save(self.out / (name + '.png'))

    def record(self, app, case, fields, focus_target=None):
        state = self.command(app)
        result = {key: state.get(key) for key in fields}
        result['focus'] = [name for name, w in state['widgets'].items() if w['focus']]
        if focus_target:
            result['entry_focus'] = state['widgets']['entry']['focus']
            result['target_focus'] = state['widgets'][focus_target]['focus'] or (focus_target == 'dropdown' and state['dropdown_open'])
        self.traces.append(dict(app=app, case=case, result=result))
        (self.out / 'traces.json').write_text(json.dumps(self.traces, indent=2, ensure_ascii=False))
        return state

    def semantics(self, app):
        with self.app(app):
            for name, fields in [('check', ['check']), ('switch', ['switch']), ('button', ['button'])]:
                for key in ['space', 'Return', 'KP_Enter', 'ctrl+space', 'shift+space', 'alt+space', 'ctrl+Return', 'shift+Return', 'alt+Return']:
                    self.command(app, reset=True, focus=name)
                    self.key(key)
                    self.record(app, f'{name}.{key}', fields)
            for key in ['Down', 'Up', 'Home', 'End', 'ctrl+Down', 'alt+Down', 'space', 'Return']:
                self.key('Escape')
                self.key('Escape')
                self.command(app, dropdown=1, focus='dropdown')
                self.key(key)
                self.record(app, f'dropdown.closed.{key}', ['dropdown', 'dropdown_open'])
            self.key('Escape')
            self.key('Escape')
            self.command(app, dropdown=0, focus='dropdown')
            self.key('space')
            for key in ['Down', 'End', 'Home', 'Up', 'Return']:
                self.key(key)
                self.record(app, f'dropdown.open.{key}', ['dropdown', 'dropdown_open'])
            self.key('Escape')
            self.key('Escape')
            for index in [0, 2]:
                for key in ['Right', 'Left', 'Down', 'Up', 'Home', 'End', 'space', 'Return']:
                    self.command(app, radio=index)
                    self.command(app, focus='radio')
                    self.key(key)
                    self.record(app, f'radio.{index}.{key}', ['radio'])
            for key in ['Up', 'Down', 'Page_Up', 'Page_Down', 'Home', 'End', 'ctrl+Right', 'ctrl+Home', 'ctrl+End', 'ctrl+Page_Up', 'ctrl+Page_Down']:
                self.command(app, number=4, focus='number')
                self.key(key)
                self.record(app, f'number.{key}', ['number'])
            for text in ['-', 'abc', '', '12.5', '-3.25', '1e1', '1,5', 'NaN', 'inf']:
                self.command(app, focus='entry')
                self.command(app, number=4, focus='number')
                self.key('ctrl+a')
                if text:
                    self.type(text)
                else:
                    self.key('BackSpace')
                self.record(app, f'number.input.{text!r}', ['number'])
                self.key('Return')
                self.record(app, f'number.commit.{text!r}', ['number'])
            for key in ['Right', 'Left', 'Up', 'Down', 'Page_Up', 'Page_Down', 'Home', 'End', 'ctrl+Right', 'ctrl+Home', 'ctrl+End', 'plus', 'minus']:
                self.command(app, slider=40, focus='slider')
                self.key(key)
                self.record(app, f'slider.{key}', ['slider'])
            text = 'Příliš žluťoučký kůň skáče.'
            for keys in [['Home'], ['End'], ['ctrl+Left'], ['ctrl+Home', 'ctrl+Right'], ['Home', 'shift+Right'], ['End', 'ctrl+shift+Left'], ['End', 'ctrl+BackSpace'], ['Home', 'Delete']]:
                self.command(app, text=text, focus='entry')
                self.key('End')
                for key in keys:
                    self.key(key)
                st = self.record(app, 'entry.' + ','.join(keys), ['entry', 'selection', 'cursor'])
                selected = st.get('selection')
                if app == 'egui' and selected:
                    self.traces[-1]['result']['cursor'] = selected[0]
                self.traces[-1]['result']['selection'] = sorted(selected) if selected else []
                if selected and len(set(selected)) == 1:
                    self.traces[-1]['result']['selection'] = []
            for disabled in [True, False]:
                self.command(app, list_selected=0, list_disabled=disabled)
                self.command(app, focus='listbox')
                for index, key in enumerate(['Down', 'Down', 'Page_Down', 'Page_Down', 'Page_Up', 'ctrl+Down', 'space', 'Home', 'Up', 'End', 'Down', 'Return']):
                    self.key(key)
                    self.record(app, f'list.{('disabled' if disabled else 'enabled')}.{index}.{key}', ['list_selected', 'list_activated'])
            for name in ['button', 'switch', 'check', 'dropdown']:
                self.key('Escape')
                self.key('Escape')
                self.command(app, reset=True, focus='entry')
                self.key('Tab')
                self.command(app, focus='entry')
                box = self.state(app)['widgets'][name]
                width = box.get('bounds_width', box['width'])
                self.x('mousemove', round(box['x'] + width / 2), round(box['y'] + box['height'] / 2))
                for phase in ['press', 'release']:
                    self.x('mousedown' if phase == 'press' else 'mouseup', '1')
                    self.record(app, f'pointer.{name}.{phase}', ['button', 'check', 'switch', 'dropdown', 'dropdown_open'], focus_target=name)
            self.key('Escape')
            self.key('Escape')
            self.command(app, focus='menu')
            self.key('space')
            self.key('Right')
            self.key('Down')
            self.key('Return')
            self.record(app, 'menu.Right.Down.Enter', ['menu_selected'])
            self.key('Escape')
            self.key('Escape')
            self.command(app, focus='dialog')
            self.key('space')
            time.sleep(0.25)
            self.key('Return')
            self.record(app, 'dialog.Enter', ['dialog_save'])
            self.command(app, focus='dialog')
            self.key('space')
            time.sleep(0.25)
            self.key('Escape')
            self.record(app, 'dialog.Escape', ['dialog_save'])
            self.x('mousemove', 600, 685)
            self.x('mousedown', '5')
            time.sleep(0.6)
            self.record(app, 'scroll.nested.one-wheel', ['scroll'])
            self.x('mouseup', '5')

    def visuals(self):
        geometry = []
        for mode in ['light', 'dark']:
            for disabled in [False, True]:
                for app in ['gtk', 'egui']:
                    with self.app(app, mode, disabled):
                        self.x('mousemove', 880, 880)
                        time.sleep(0.3)
                        st = self.state(app)
                        tag = f'{app}-{mode}-' + ('disabled' if disabled else 'normal')
                        self.snapshot(tag)
                        geometry.append(dict(app=app, mode=mode, disabled=disabled, widgets=st['widgets']))
                        if not disabled:
                            for name in ['button', 'plain', 'entry', 'number', 'switch', 'check', 'dropdown', 'destructive', 'invalid']:
                                box = st['widgets'][name]
                                width = box.get('bounds_width', box['width'])
                                rect = (int(box['x']) - 4, int(box['y']) - 4, int(box['x'] + width) + 4, int(box['y'] + box['height']) + 4)
                                point = (round(box['x'] + width / 2), round(box['y'] + box['height'] / 2))
                                self.x('mousemove', *point)
                                self.command(app)
                                time.sleep(0.25)
                                self.snapshot(tag + '-' + name + '-hover', rect)
                                self.x('mousedown', '1')
                                self.command(app)
                                time.sleep(0.25)
                                self.snapshot(tag + '-' + name + '-pressed', rect)
                                self.x('mouseup', '1')
                                self.key('Escape')
                                self.command(app)
                                self.snapshot(tag + '-' + name + '-pointer-focus', rect)
                                self.key('Tab')
                                self.command(app, focus=name)
                                self.x('mousemove', 880, 880)
                                time.sleep(0.25)
                                self.snapshot(tag + '-' + name + '-focus', rect)
                                self.command(app, reset=True)
                                self.key('Escape')
                pair = Image.new('RGB', (1800, 900))
                suffix = mode + ('-disabled' if disabled else '-normal')
                pair.paste(Image.open(self.out / ('gtk-' + suffix + '.png')), (0, 0))
                pair.paste(Image.open(self.out / ('egui-' + suffix + '.png')), (900, 0))
                pair.save(self.out / ('comparison-' + suffix + '.png'))
        (self.out / 'geometry.json').write_text(json.dumps(geometry, indent=2))
        for mode in ['light', 'dark']:
            names = ['button', 'plain', 'entry', 'number', 'switch', 'check', 'dropdown', 'destructive', 'invalid']
            phases = ['hover', 'pressed', 'pointer-focus', 'focus']
            sheet = Image.new('RGB', (1064, len(names) * 112), '#888888')
            labels = ImageDraw.Draw(sheet)
            for row, name in enumerate(names):
                for column, phase in enumerate(phases):
                    x, y = (column * 266, row * 112)
                    labels.text((x + 3, y + 2), name + ' ' + phase, fill='white')
                    for index, app in enumerate(['gtk', 'egui']):
                        with Image.open(self.out / f'{app}-{mode}-normal-{name}-{phase}.png') as crop:
                            sheet.paste(crop.convert('RGB'), (x, y + 16 + index * 45))
            sheet.save(self.out / ('states-' + mode + '.png'))
        from metrics import measure
        measure(self.out)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=pathlib.Path, required=True)
    parser.add_argument('--runtime', type=pathlib.Path, required=True)
    parser.add_argument('--binary', type=pathlib.Path, required=True)
    parser.add_argument('--visuals', action='store_true')
    args = parser.parse_args()
    args.out = args.out.resolve()
    args.out.mkdir(parents=True, exist_ok=True)
    driver = Driver(args.out, args.runtime.resolve(), args.binary.resolve())
    with driver.display():
        for app in ['gtk', 'egui']:
            print('Measuring', app, flush=True)
            driver.semantics(app)
        (args.out / 'traces.json').write_text(json.dumps(driver.traces, indent=2, ensure_ascii=False))
        if args.visuals:
            driver.visuals()
    print('Measured', len(driver.traces), 'native interaction states', flush=True)
    from compare import compare
    report = compare(args.out)
    if report['unexpected']:
        raise SystemExit(1)
if __name__ == '__main__':
    main()
